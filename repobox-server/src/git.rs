use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use axum::body::{Body, Bytes};
use http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepoPath {
    pub address: String,
    pub name: String,
}

#[derive(Debug)]
pub(crate) struct BackendRequest<'a> {
    pub method: &'a str,
    pub path_info: String,
    pub query_string: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub body: Bytes,
}

pub(crate) fn parse_repo(repo: &str) -> Result<String, StatusCode> {
    let Some(name) = repo.strip_suffix(".git") else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !is_safe_segment(name) || name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(name.to_string())
}

pub(crate) fn validate_address(address: &str) -> Result<(), StatusCode> {
    if is_safe_segment(address) && !address.is_empty() {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub(crate) fn repo_dir(data_dir: &Path, repo: &RepoPath) -> PathBuf {
    data_dir.join(&repo.address).join(format!("{}.git", repo.name))
}

pub(crate) fn staging_repo_dir(data_dir: &Path, repo_name: &str) -> PathBuf {
    data_dir.join("_staging").join(format!("{}.git", repo_name))
}

pub(crate) fn ensure_repo_exists(data_dir: &Path, repo: &RepoPath) -> std::io::Result<bool> {
    let repo_dir = repo_dir(data_dir, repo);
    if repo_dir.exists() {
        return Ok(false);
    }

    let parent = repo_dir
        .parent()
        .ok_or_else(|| std::io::Error::other("invalid repository path"))?;
    std::fs::create_dir_all(parent)?;

    run_git(&["init", "--bare", repo_dir.to_string_lossy().as_ref()])?;
    run_git(&[
        "--git-dir",
        repo_dir.to_string_lossy().as_ref(),
        "config",
        "http.receivepack",
        "true",
    ])?;
    // Default HEAD to main (most repos use main now)
    run_git(&[
        "--git-dir",
        repo_dir.to_string_lossy().as_ref(),
        "symbolic-ref",
        "HEAD",
        "refs/heads/main",
    ])?;

    install_pre_receive_hook(&repo_dir)?;
    Ok(true)
}

pub(crate) fn ensure_staging_repo_exists(data_dir: &Path, repo_name: &str) -> std::io::Result<bool> {
    let staging_dir = staging_repo_dir(data_dir, repo_name);
    if staging_dir.exists() {
        return Ok(false);
    }

    let parent = staging_dir
        .parent()
        .ok_or_else(|| std::io::Error::other("invalid staging path"))?;
    std::fs::create_dir_all(parent)?;

    run_git(&["init", "--bare", staging_dir.to_string_lossy().as_ref()])?;
    run_git(&[
        "--git-dir",
        staging_dir.to_string_lossy().as_ref(),
        "config",
        "http.receivepack",
        "true",
    ])?;
    run_git(&[
        "--git-dir",
        staging_dir.to_string_lossy().as_ref(),
        "symbolic-ref",
        "HEAD",
        "refs/heads/main",
    ])?;

    install_pre_receive_hook(&staging_dir)?;
    Ok(true)
}

pub(crate) fn move_repo_from_staging(data_dir: &Path, repo_name: &str, target_address: &str) -> std::io::Result<()> {
    let staging_dir = staging_repo_dir(data_dir, repo_name);
    let target_repo = RepoPath {
        address: target_address.to_string(),
        name: repo_name.to_string(),
    };
    let target_dir = repo_dir(data_dir, &target_repo);

    if !staging_dir.exists() {
        return Err(std::io::Error::other("staging repo does not exist"));
    }

    // Create the target directory parent
    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Move the staging repo to the final location
    std::fs::rename(&staging_dir, &target_dir)?;
    
    Ok(())
}

pub(crate) fn clean_staging_repo(data_dir: &Path, repo_name: &str) -> std::io::Result<()> {
    let staging_dir = staging_repo_dir(data_dir, repo_name);
    if staging_dir.exists() {
        std::fs::remove_dir_all(&staging_dir)?;
    }
    Ok(())
}

pub(crate) fn read_head(data_dir: &Path, repo: &RepoPath) -> std::io::Result<String> {
    std::fs::read_to_string(repo_dir(data_dir, repo).join("HEAD"))
}

/// Extract the EVM signer address from the first signed commit in a repo.
/// Walks the default branch (HEAD) and finds the root commit.
/// If the root commit has an EVM signature (65-byte gpgsig), recovers the signer address.
/// Returns None if no signed commits are found.
pub(crate) fn extract_owner_from_first_commit(data_dir: &Path, repo: &RepoPath) -> std::io::Result<Option<String>> {
    let repo_dir = repo_dir(data_dir, repo);
    extract_owner_from_repo_dir(&repo_dir)
}

/// Extract the EVM signer address from the first signed commit in a staging repo.
pub(crate) fn extract_owner_from_staging_repo(data_dir: &Path, repo_name: &str) -> std::io::Result<Option<String>> {
    let staging_dir = staging_repo_dir(data_dir, repo_name);
    extract_owner_from_repo_dir(&staging_dir)
}

fn extract_owner_from_repo_dir(repo_dir: &Path) -> std::io::Result<Option<String>> {
    let repo_dir_str = repo_dir.to_string_lossy().to_string();

    // Get the root commit (first commit in history)
    let output = Command::new("git")
        .args(["--git-dir", &repo_dir_str, "rev-list", "--max-parents=0", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let root_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root_hash.is_empty() {
        return Ok(None);
    }

    // Get the raw commit object to extract the gpgsig
    let output = Command::new("git")
        .args(["--git-dir", &repo_dir_str, "cat-file", "commit", &root_hash])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let commit_text = String::from_utf8_lossy(&output.stdout);
    extract_signer_from_commit_text(&commit_text)
}

/// Parse a raw commit object and extract the EVM signer address from the gpgsig header.
/// The gpgsig is a hex-encoded 65-byte recoverable ECDSA signature.
/// The signed data is the commit content WITHOUT the gpgsig header.
fn extract_signer_from_commit_text(commit_text: &str) -> std::io::Result<Option<String>> {
    // Extract the gpgsig header value
    let sig_hex = match extract_gpgsig(commit_text) {
        Some(s) => s,
        None => return Ok(None),
    };

    let sig_bytes = hex::decode(&sig_hex).map_err(|e| {
        std::io::Error::other(format!("invalid gpgsig hex: {e}"))
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

/// Extract the gpgsig value from a raw commit object.
/// Git stores it as a multi-line header with continuation lines starting with a space.
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

    // Join and clean — for our EVM sigs this should be a single hex string
    let combined: String = sig_lines.join("");
    Some(combined.trim().to_string())
}

/// Strip the gpgsig header from a commit to reconstruct the signed content.
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

pub(crate) fn run_backend(
    data_dir: &Path,
    request: BackendRequest<'_>,
) -> std::io::Result<Response<Body>> {
    let mut command = Command::new("git");
    command.arg("http-backend");
    command.env("GIT_PROJECT_ROOT", data_dir);
    command.env("GIT_HTTP_EXPORT_ALL", "1");
    command.env("REQUEST_METHOD", request.method);
    command.env("PATH_INFO", &request.path_info);
    command.env("QUERY_STRING", request.query_string.unwrap_or_default());
    command.env("CONTENT_TYPE", request.content_type.unwrap_or_default());
    command.env("CONTENT_LENGTH", request.body.len().to_string());
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(&request.body)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "git http-backend failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    parse_backend_response(&output.stdout)
}

fn parse_backend_response(output: &[u8]) -> std::io::Result<Response<Body>> {
    let (raw_headers, body) = split_cgi_response(output)?;
    let headers_text = String::from_utf8_lossy(raw_headers);
    let mut status = StatusCode::OK;
    let mut headers = HeaderMap::new();

    for line in headers_text.lines() {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() {
            continue;
        }

        let Some((name, value)) = trimmed.split_once(':') else {
            continue;
        };

        if name.eq_ignore_ascii_case("Status") {
            let code = value
                .trim()
                .split_whitespace()
                .next()
                .and_then(|part| part.parse::<u16>().ok())
                .ok_or_else(|| std::io::Error::other("invalid CGI status line"))?;
            status = StatusCode::from_u16(code)
                .map_err(|_| std::io::Error::other("invalid HTTP status code"))?;
            continue;
        }

        let header_name = HeaderName::from_bytes(name.trim().as_bytes())
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let header_value = HeaderValue::from_str(value.trim())
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        headers.append(header_name, header_value);
    }

    let mut builder = Response::builder().status(status);
    let response_headers = builder
        .headers_mut()
        .ok_or_else(|| std::io::Error::other("failed to build response headers"))?;
    response_headers.extend(headers);
    builder
        .body(Body::from(body.to_vec()))
        .map_err(|error| std::io::Error::other(error.to_string()))
}

fn split_cgi_response(output: &[u8]) -> std::io::Result<(&[u8], &[u8])> {
    if let Some(index) = output.windows(4).position(|window| window == b"\r\n\r\n") {
        return Ok((&output[..index], &output[index + 4..]));
    }
    if let Some(index) = output.windows(2).position(|window| window == b"\n\n") {
        return Ok((&output[..index], &output[index + 2..]));
    }
    Err(std::io::Error::other("malformed CGI response"))
}

fn install_pre_receive_hook(repo_dir: &Path) -> std::io::Result<()> {
    let hook_path = repo_dir.join("hooks").join("pre-receive");
    let script = "#!/bin/sh\nif command -v repobox-check >/dev/null 2>&1; then\n    exec repobox-check \"$@\"\nfi\nexit 0\n";
    std::fs::write(&hook_path, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, permissions)?;
    }

    Ok(())
}

fn run_git(args: &[&str]) -> std::io::Result<()> {
    let output = Command::new("git").args(args).output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(std::io::Error::other(format!(
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    )))
}

fn is_safe_segment(segment: &str) -> bool {
    !segment.contains("..")
        && !segment.contains('/')
        && !segment.contains('\\')
        && !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b':' ))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_extract_gpgsig_multiline() {
        let commit = "tree abc123\n\
                       author Test <test@test.com> 1234567890 +0000\n\
                       committer Test <test@test.com> 1234567890 +0000\n\
                       gpgsig aabb\n \
                       ccdd\n\
                       \n\
                       initial commit\n";
        let sig = extract_gpgsig(commit);
        assert_eq!(sig, Some("aabbccdd".to_string()));
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

    #[test]
    fn test_strip_gpgsig() {
        let commit = "tree abc123\n\
                       author Test <test@test.com> 1234567890 +0000\n\
                       committer Test <test@test.com> 1234567890 +0000\n\
                       gpgsig deadbeef01\n\
                       \n\
                       initial commit\n";
        let stripped = strip_gpgsig(commit);
        assert!(!stripped.contains("gpgsig"));
        assert!(stripped.contains("tree abc123"));
        assert!(stripped.contains("initial commit"));
    }
}
