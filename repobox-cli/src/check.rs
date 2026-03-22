//! Pre-receive hook handler for repo.box
//! Called by git hooks to validate pushes before they're accepted

use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Command, ExitCode};

use repobox::config::{Identity, Verb};
use repobox::engine;
use repobox::parser;

const NULL_SHA: &str = "0000000000000000000000000000000000000000";

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut reject_push = false;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                reject_push = true;
                continue;
            }
        };

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            eprintln!("Invalid ref update format: {}", line);
            reject_push = true;
            continue;
        }

        let old_sha = parts[0];
        let new_sha = parts[1];
        let ref_name = parts[2];

        // Deletions don't introduce commits.
        if new_sha == NULL_SHA {
            continue;
        }

        // Extract branch name for branch refs
        let branch_name = ref_name.strip_prefix("refs/heads/");

        // Collect all new commits introduced by this ref update.
        let new_commits = match list_new_commits(Path::new("."), old_sha, new_sha) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("❌ Failed to inspect pushed commits: {}", e);
                reject_push = true;
                continue;
            }
        };

        if new_commits.is_empty() {
            // Fallback to tip commit if rev-list returned nothing unexpectedly.
            if let Ok(Some(_)) = extract_signer_from_commit(new_sha) {
                // ok
            } else {
                eprintln!("❌ Unsigned commit rejected: {}", &new_sha[..8.min(new_sha.len())]);
                reject_push = true;
                continue;
            }
        }

        // Enforce signatures on every newly introduced commit.
        let mut tip_pusher: Option<String> = None;
        for sha in &new_commits {
            match extract_signer_from_commit(sha) {
                Ok(Some(address)) => {
                    if sha == new_sha {
                        tip_pusher = Some(format!("evm:{}", address));
                    }
                }
                Ok(None) => {
                    eprintln!(
                        "❌ Unsigned commit rejected: {} on {}",
                        &sha[..8.min(sha.len())],
                        ref_name
                    );
                    eprintln!("   All commits pushed to repo.box must be EVM-signed.");
                    reject_push = true;
                }
                Err(e) => {
                    eprintln!("❌ Failed to validate commit signature {}: {}", sha, e);
                    reject_push = true;
                }
            }
        }

        let pusher = tip_pusher.unwrap_or_else(|| get_pusher_identity(new_sha));
        if pusher == "unknown" {
            eprintln!("❌ Could not determine pusher identity for {}", ref_name);
            reject_push = true;
            continue;
        }

        // Check push permission for config-enabled repos (branch-aware).
        if let Some(branch) = branch_name {
            match check_push_authorized(&pusher, branch) {
                Ok(true) => {}
                Ok(false) => {
                    eprintln!("❌ Push denied for {} on branch {}", pusher, branch);
                    reject_push = true;
                }
                Err(e) => {
                    eprintln!("❌ Error checking push permissions: {}", e);
                    reject_push = true;
                }
            }

            // Check force-push permission.
            match is_force_push_update(Path::new("."), old_sha, new_sha, ref_name) {
                Ok(true) => {
                    eprintln!("🚨 Force push detected on branch: {}", branch);
                    match check_force_push_authorized(&pusher, branch) {
                        Ok(true) => {
                            eprintln!("✅ Force push authorized for {} on {}", pusher, branch);
                        }
                        Ok(false) => {
                            eprintln!("❌ Force push denied for {} on {}", pusher, branch);
                            reject_push = true;
                        }
                        Err(e) => {
                            eprintln!("❌ Error checking force push permissions: {}", e);
                            reject_push = true;
                        }
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    eprintln!("❌ Error checking for force push: {}", e);
                    reject_push = true;
                }
            }
        }
    }

    if reject_push {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// List commits introduced by this ref update.
fn list_new_commits(repo_dir: &Path, old_sha: &str, new_sha: &str) -> std::io::Result<Vec<String>> {
    let output = if old_sha == NULL_SHA {
        // New branch/tag: only commits not already reachable from any existing ref.
        Command::new("git")
            .current_dir(repo_dir)
            .args(["rev-list", new_sha, "--not", "--all"])
            .output()?
    } else {
        // Existing ref update.
        let range = format!("{old_sha}..{new_sha}");
        Command::new("git")
            .current_dir(repo_dir)
            .args(["rev-list", &range])
            .output()?
    };

    if !output.status.success() {
        return Err(std::io::Error::other("git rev-list failed"));
    }

    let commits = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(commits)
}

/// Check if a ref update represents a force push
fn is_force_push_update(
    repo_dir: &Path,
    old_sha: &str,
    new_sha: &str,
    ref_name: &str,
) -> std::io::Result<bool> {
    // Skip for new branches and deletions
    if old_sha == NULL_SHA || new_sha == NULL_SHA {
        return Ok(false);
    }

    // Skip for non-branch refs (tags, etc.)
    if !ref_name.starts_with("refs/heads/") {
        return Ok(false);
    }

    // Check if new commit is descendant of old commit
    let output = Command::new("git")
        .current_dir(repo_dir)
        .args(["merge-base", "--is-ancestor", old_sha, new_sha])
        .output()?;

    // If git merge-base --is-ancestor succeeds, it's a fast-forward (not force push)
    // If it fails, it's a force push (non-fast-forward)
    Ok(!output.status.success())
}

/// Get pusher identity from environment or git commit
fn get_pusher_identity(new_sha: &str) -> String {
    // Try to get from environment variables (set by git server)
    if let Ok(pusher) = std::env::var("GL_USERNAME") {
        return pusher;
    }
    if let Ok(pusher) = std::env::var("PUSHER") {
        return pusher;
    }
    if let Ok(pusher) = std::env::var("GIT_PUSHER") {
        return pusher;
    }

    // Try to extract from commit signature
    match extract_signer_from_commit(new_sha) {
        Ok(Some(address)) => format!("evm:{}", address),
        Ok(None) => "unknown".to_string(),
        Err(_) => "unknown".to_string(),
    }
}

/// Extract EVM signer address from a commit
fn extract_signer_from_commit(commit_sha: &str) -> std::io::Result<Option<String>> {
    let output = Command::new("git")
        .args(["cat-file", "commit", commit_sha])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let commit_text = String::from_utf8_lossy(&output.stdout);
    extract_signer_from_commit_text(&commit_text)
}

/// Parse a raw commit object and extract the EVM signer address from the gpgsig header
fn extract_signer_from_commit_text(commit_text: &str) -> std::io::Result<Option<String>> {
    // Extract the gpgsig header value
    let sig_hex = match extract_gpgsig(commit_text) {
        Some(s) => s,
        None => return Ok(None),
    };

    let sig_bytes = hex::decode(&sig_hex).map_err(|e| {
        std::io::Error::other(format!("invalid gpgsig hex: {}", e))
    })?;

    if sig_bytes.len() != 65 {
        return Ok(None); // Not a repobox EVM signature
    }

    // Reconstruct the commit content without the gpgsig header (that's what was signed)
    let signed_data = strip_gpgsig(commit_text);

    match repobox::signing::recover_address(signed_data.as_bytes(), &sig_bytes) {
        Ok(address) => Ok(Some(address)),
        Err(_) => Ok(None),
    }
}

/// Extract the gpgsig value from a raw commit object
fn extract_gpgsig(commit_text: &str) -> Option<String> {
    let mut in_gpgsig = false;
    let mut sig_lines = Vec::new();

    for line in commit_text.lines() {
        if line.starts_with("gpgsig ") {
            in_gpgsig = true;
            sig_lines.push(line.strip_prefix("gpgsig ").unwrap().trim());
        } else if in_gpgsig && line.starts_with(' ') {
            sig_lines.push(line.trim());
        } else if in_gpgsig {
            break;
        }
    }

    if sig_lines.is_empty() {
        return None;
    }

    // Join and clean
    let combined: String = sig_lines.join("");
    let combined = combined.trim().to_string();

    // Strip REPOBOX SIGNATURE armor if present
    let combined = combined
        .replace("-----BEGIN REPOBOX SIGNATURE-----", "")
        .replace("-----END REPOBOX SIGNATURE-----", "")
        .trim()
        .to_string();

    if combined.is_empty() {
        None
    } else {
        Some(combined)
    }
}

/// Strip the gpgsig header from a commit to reconstruct the signed content
fn strip_gpgsig(commit_text: &str) -> String {
    let mut result = String::new();
    let mut in_gpgsig = false;

    for line in commit_text.lines() {
        if line.starts_with("gpgsig ") {
            in_gpgsig = true;
            continue;
        }
        if in_gpgsig && line.starts_with(' ') {
            continue;
        }
        in_gpgsig = false;
        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if the original didn't have one
    if !commit_text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Check if push is authorized for the given pusher and branch.
fn check_push_authorized(pusher: &str, branch_name: &str) -> std::io::Result<bool> {
    let config_path = Path::new(".repobox/config.yml");
    if !config_path.exists() {
        // No config = allow (opt-in enforcement).
        return Ok(true);
    }

    let config_content = std::fs::read_to_string(config_path)?;

    // Fail closed on invalid config when enforcement is enabled.
    let config = parser::parse(&config_content)
        .map_err(|e| std::io::Error::other(format!("invalid .repobox/config.yml: {e}")))?;

    let identity = match Identity::parse(pusher) {
        Ok(id) => id,
        Err(_) => return Ok(false),
    };

    let result = engine::check(
        &config,
        &identity,
        Verb::Push,
        Some(branch_name),
        None,
    );

    Ok(result.is_allowed())
}

/// Check if force push is authorized for the given pusher and branch
fn check_force_push_authorized(pusher: &str, branch_name: &str) -> std::io::Result<bool> {
    // Read .repobox/config.yml if it exists
    let config_path = Path::new(".repobox/config.yml");
    if !config_path.exists() {
        // No config = allow (opt-in enforcement)
        return Ok(true);
    }

    let config_content = std::fs::read_to_string(config_path)?;

    // Fail closed on invalid config when enforcement is enabled.
    let config = parser::parse(&config_content)
        .map_err(|e| std::io::Error::other(format!("invalid .repobox/config.yml: {e}")))?;

    // Parse the identity
    let identity = match Identity::parse(pusher) {
        Ok(id) => id,
        Err(_) => {
            // Invalid identity = deny
            return Ok(false);
        }
    };

    // Check force-push permission
    let result = engine::check(
        &config,
        &identity,
        Verb::ForcePush,
        Some(branch_name),
        None,
    );

    Ok(result.is_allowed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_force_push_detection() {
        // Test case: new branch (not a force push)
        let result = is_force_push_update(
            Path::new("."),
            NULL_SHA,
            "abc123",
            "refs/heads/main",
        );
        assert!(!result.unwrap());

        // Test case: deletion (not a force push)
        let result = is_force_push_update(
            Path::new("."),
            "abc123",
            NULL_SHA,
            "refs/heads/main",
        );
        assert!(!result.unwrap());

        // Test case: tag update (skip, not a branch)
        let result = is_force_push_update(
            Path::new("."),
            "abc123",
            "def456",
            "refs/tags/v1.0",
        );
        assert!(!result.unwrap());
    }

    #[test]
    fn test_extract_gpgsig() {
        let commit = "tree abc123\n\
                       author Test <test@test.com> 1234567890 +0000\n\
                       committer Test <test@test.com> 1234567890 +0000\n\
                       gpgsig deadbeef01\n\
                       \n\
                       initial commit\n";
        let sig = extract_gpgsig(commit);
        assert_eq!(sig, Some("deadbeef01".to_string()));
    }

    #[test]
    fn test_extract_gpgsig_none() {
        let commit = "tree abc123\n\
                       author Test <test@test.com> 1234567890 +0000\n\
                       committer Test <test@test.com> 1234567890 +0000\n\
                       \n\
                       unsigned commit\n";
        let sig = extract_gpgsig(commit);
        assert_eq!(sig, None);
    }
}