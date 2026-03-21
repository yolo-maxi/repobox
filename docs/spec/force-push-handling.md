# repo.box Spec: Force Push Handling

## Overview

Force push (`git push --force`) rewrites Git history by moving branch heads to new positions, potentially discarding commits. In a signed-commit model where every commit has EVM provenance and cryptographic integrity, force pushes pose significant security and audit risks.

This specification defines how repo.box detects, controls, and handles force push operations to maintain repository integrity while preserving legitimate use cases.

## Background & Threat Model

### What is Force Push?

Force push allows pushing commits that rewrite history:

```bash
# Normal push (fast-forward only)
git push origin main

# Force push (rewrites history) 
git push --force origin main
git push -f origin main

# Safer variant (checks remote state first)
git push --force-with-lease origin main
```

### Security Implications for EVM-Signed Commits

1. **Audit Trail Destruction**: Force pushes can completely erase signed commits from the remote repository, breaking the cryptographic audit chain.

2. **Identity Spoofing**: An attacker with force-push access could replace legitimate EVM-signed commits with malicious ones, potentially impersonating other contributors.

3. **History Manipulation**: Previous decisions, code reviews, and approvals embedded in commit messages become invisible, undermining governance.

4. **Signature Invalidation**: If a force push moves a branch to a different tree, previously valid EVM signatures might appear to sign different content.

### Attack Scenarios

- **Malicious Agent**: A compromised AI agent force-pushes to erase evidence of malicious commits
- **Accidental Override**: Developer accidentally destroys days of team work with `git push -f`
- **Privilege Escalation**: Agent with limited permissions uses force-push to bypass file editing restrictions

## Technical Analysis

### Force Push Detection in Pre-Receive Hooks

Git's pre-receive hook receives ref updates in this format:
```
<old-sha> <new-sha> <ref-name>
```

A force push is detectable when:
1. `old-sha != new-sha` (branch is moving)
2. `new-sha` is not a descendant of `old-sha` (not fast-forward)
3. `old-sha != 0000000000000000000000000000000000000000` (not a new branch)

**Algorithm:**
```rust
fn is_force_push(old_sha: &str, new_sha: &str) -> bool {
    if old_sha == NULL_SHA || old_sha == new_sha {
        return false; // New branch or no change
    }
    
    // Check if new_sha is descendant of old_sha
    let output = Command::new("git")
        .args(["merge-base", "--is-ancestor", old_sha, new_sha])
        .output();
        
    !output.map(|o| o.status.success()).unwrap_or(false)
}
```

### --force vs --force-with-lease

- **`--force`**: Unconditionally overwrites remote branch
- **`--force-with-lease`**: Only overwrites if remote branch matches expected state

Both are force pushes from a history perspective, but `--force-with-lease` is safer as it prevents accidental overwrites of others' work.

### Current repobox-server Implementation

The existing pre-receive hook in `git.rs` (`install_pre_receive_hook`) only calls `repobox-check` for general permission validation. It does not specifically analyze ref updates for force push detection.

## Industry Analysis

### GitHub
- **Default**: Force pushes allowed on all branches
- **Branch Protection**: Can disable force pushes via "Restrict pushes that create files"
- **Force Push Protection**: Available for default branches in repository settings

### GitLab
- **Default**: Force pushes allowed
- **Push Rules** (Premium): Can disable force pushes globally or per-branch
- **Protected Branches**: Force push restrictions available

### Best Practices
- Most organizations disable force pushes on protected branches (main, develop)
- Allow force pushes on feature branches for developer workflow
- Require special permissions for force pushes in production environments

## Recommendation: Conditional Allow with Permission Gating

Based on the signed-commit security model and industry practices, **we recommend allowing force pushes but requiring explicit permission**.

### Rationale

**Against complete disallow:**
- Legitimate developer workflows need force push (rebase, amend, squash)
- Feature branches often require history cleanup before merge
- Emergency fixes sometimes need rapid deployment over proper history

**For permission-gated approach:**
- Preserves legitimate use cases while adding security controls
- Aligns with repo.box's permission model (everything is configurable)
- Allows different policies for different branches
- Maintains audit trail of who performed force pushes

## Implementation Plan

### 1. Add `force-push` Verb to Permission System

Extend the `Verb` enum in `repobox-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verb {
    Push,
    Merge,
    Create,
    Delete,
    ForcePush, // NEW
    Edit,
    Write,
    Append,
}
```

### 2. Update Pre-Receive Hook Logic

Modify `repobox-server/src/git.rs` to detect force pushes in the pre-receive hook:

```rust
/// Check if a ref update represents a force push
pub(crate) fn is_force_push_update(
    repo_dir: &Path,
    old_sha: &str,
    new_sha: &str,
    ref_name: &str,
) -> std::io::Result<bool> {
    // Skip for new branches and deletions
    if old_sha == NULL_SHA || new_sha == NULL_SHA {
        return Ok(false);
    }
    
    // Skip for non-branch refs
    if !ref_name.starts_with("refs/heads/") {
        return Ok(false);
    }
    
    // Check if new commit is descendant of old commit
    let output = Command::new("git")
        .current_dir(repo_dir)
        .args(["merge-base", "--is-ancestor", old_sha, new_sha])
        .output()?;
        
    Ok(!output.status.success())
}
```

### 3. Enhanced Permission Checking

Create new function in `repobox-server/src/git.rs`:

```rust
pub(crate) fn check_force_push_authorized(
    data_dir: &Path,
    repo: &RepoPath,
    pusher_address: &str,
    branch_name: &str,
    is_force_with_lease: bool, // Future enhancement
) -> std::io::Result<bool> {
    // Read .repobox/config.yml and check force-push permission
    let config = match read_config_from_repo(&repo_dir(data_dir, repo)) {
        Some(content) => match repobox::parser::parse(&content) {
            Ok(cfg) => cfg,
            Err(_) => return Ok(true), // Invalid config = allow (opt-in)
        },
        None => return Ok(true), // No config = allow (opt-in)
    };

    let identity = repobox::config::Identity::parse(&format!("evm:{pusher_address}"))
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    
    let result = repobox::engine::check(
        &config,
        &identity,
        repobox::config::Verb::ForcePush,
        Some(branch_name),
        None,
    );
    
    Ok(result.is_allowed())
}
```

### 4. Create `repobox-check` Binary

Add new binary target in `repobox-cli/Cargo.toml`:

```toml
[[bin]]
name = "repobox-check"
path = "src/check.rs"
```

Create `repobox-cli/src/check.rs`:

```rust
//! Pre-receive hook handler for repo.box
//! Called by git hooks to validate pushes before they're accepted

use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let stdin = io::stdin();
    
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        
        let old_sha = parts[0];
        let new_sha = parts[1];
        let ref_name = parts[2];
        
        // Extract branch name
        let branch_name = match ref_name.strip_prefix("refs/heads/") {
            Some(name) => name,
            None => continue, // Not a branch
        };
        
        // Check if this is a force push
        if let Ok(true) = is_force_push_update(Path::new("."), old_sha, new_sha, ref_name) {
            eprintln!("🚨 Force push detected on {}", branch_name);
            
            // Get pusher identity from git env or extract from commit
            let pusher = std::env::var("GL_USERNAME")
                .or_else(|_| std::env::var("PUSHER"))
                .unwrap_or_else(|| "unknown".to_string());
                
            // For now, just log and allow - actual permission check happens in server
            eprintln!("👤 Pusher: {}", pusher);
        }
    }
    
    // Exit 0 = allow, exit 1 = reject
    std::process::exit(0);
}
```

### 5. Update Default Configuration Template

Modify `repobox-cli/src/main.rs` to include force-push examples in the config template:

```yaml
permissions:
  default: allow
  rules:
    # Branch operations
    #   - founders push >*
    #   - founders merge >*
    #   - founders force-push >*          # Allow force push everywhere
    #   - agents push >feature/**
    #   - agents force-push >feature/**   # Allow force push on feature branches
    #   - agents not force-push >main     # Explicitly deny force push to main
    
    # File operations  
    #   - founders edit ./.repobox/config.yml
```

## Configuration Options

### Repository-Level Settings

Add new configuration section in `.repobox/config.yml`:

```yaml
# Force push policy (optional)
force_push:
  # Global setting - if not specified, uses permission rules
  enabled: true  # or false to disable entirely
  
  # Require additional confirmation for force pushes (future)
  require_confirmation: true
  
  # Log all force pushes to audit trail (future)
  audit_log: true
  
  # Safer alternative recommendation
  prefer_force_with_lease: true

permissions:
  rules:
    - founders force-push >*
    - agents force-push >feature/**
    - agents not force-push >main
    - agents not force-push >develop
```

### Git Configuration Integration

Add git config option for users to set preference:

```bash
git config repobox.allowForcePush true
git config repobox.preferForceWithLease true
```

## Test Cases

### 1. Force Push Detection

```bash
# Setup test repository
git init test-repo && cd test-repo
echo "initial" > file.txt
git add . && git commit -m "initial commit"

# Create divergent history
git reset --hard HEAD~1
echo "different" > file.txt  
git add . && git commit -m "divergent commit"

# This should be detected as force push
git push --force origin main
```

**Expected**: Pre-receive hook detects force push, checks permissions

### 2. Permission Scenarios

#### Test Case A: Allowed Force Push
```yaml
permissions:
  rules:
    - alice force-push >feature/test
```
**Action**: Alice force pushes to `feature/test`
**Expected**: Allowed

#### Test Case B: Denied Force Push
```yaml  
permissions:
  rules:
    - alice not force-push >main
```
**Action**: Alice force pushes to `main`
**Expected**: Denied with clear error message

#### Test Case C: No Force Push Rules
```yaml
permissions:
  default: allow
  rules:
    - alice push >*
```
**Action**: Alice force pushes to any branch
**Expected**: Allowed (no force-push rules = unrestricted)

### 3. Edge Cases

- Empty repository (no commits)
- New branch creation (not a force push)
- Branch deletion (different operation)
- Force push with no changes (same SHA)
- Multiple ref updates in single push
- Invalid SHAs in hook input

## Backwards Compatibility

- **Existing repositories**: No force-push rules means force pushes remain allowed
- **Existing configurations**: Continue to work unchanged
- **CLI commands**: `git push --force` works transparently with permission checks
- **Hook installation**: Automatic via existing `install_pre_receive_hook`

## Security Considerations

1. **Hook Bypass**: Users cannot bypass the pre-receive hook since it runs server-side
2. **Permission Escalation**: Force-push permissions are separate from regular push
3. **Audit Trail**: All force pushes should be logged with identity and timestamp
4. **Configuration Tampering**: Force-push rules themselves are protected by file edit permissions

## Monitoring & Observability

### Audit Logging (Future Enhancement)

```json
{
  "timestamp": "2024-03-21T18:27:00Z",
  "event": "force_push",
  "repository": "alice/my-project",
  "pusher": "evm:0xABC123...",
  "branch": "feature/cleanup",
  "old_sha": "abc123...",
  "new_sha": "def456...",
  "commits_lost": 3,
  "allowed": true,
  "rule_matched": "alice force-push >feature/**"
}
```

### Metrics
- Force push frequency by repository
- Force push denials by user/agent
- Most common branches for force pushes
- Impact analysis (commits overwritten)

## Migration Strategy

### Phase 1: Detection Only (Current Sprint)
- Add force push detection to pre-receive hooks
- Log force pushes without blocking
- Add `force-push` verb to permission engine

### Phase 2: Permission Enforcement (Next Sprint)  
- Implement permission checking for force pushes
- Update CLI with new verb support
- Add configuration examples and documentation

### Phase 3: Advanced Features (Future)
- Audit logging and metrics
- `--force-with-lease` distinction
- Interactive confirmation prompts
- Integration with external approval systems

## Conclusion

Force push handling in repo.box should follow the principle of **secure by default, flexible by configuration**. By adding a dedicated `force-push` permission verb and leveraging Git's pre-receive hooks, we can:

1. Maintain the integrity of EVM-signed commit histories
2. Prevent accidental or malicious history rewrites
3. Preserve legitimate developer workflows
4. Provide granular control over force push permissions
5. Maintain full audit trails of history modifications

This approach aligns with industry best practices while respecting repo.box's core philosophy of cryptographically-verified, agent-friendly Git workflows.