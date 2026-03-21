# Enforce .repobox-config Opt-in on Server

**Specification ID**: `SPEC-CFGOPT-001`  
**Priority**: P2  
**Tags**: server, permissions, security  
**Author**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b)  
**Date**: 2026-03-21  

## Summary

Change the server's permission enforcement from "all repos" to "opt-in only". Repositories must contain `.repobox/config.yml` in their tree to have permission rules enforced. Repos without this config file will have no permission enforcement (public read/write with signature requirements only).

## Context and Motivation

**Current State:**
- Server enforces permission rules on ALL repositories
- Any repo automatically gets permission enforcement based on `.repobox/config.yml` if present
- Fallback to default policy when config is missing or invalid

**Problem:**
- Many repos don't want/need permission enforcement
- Complex permission rules slow down simple repositories
- Users expect simple git repos to "just work" without configuration overhead
- Permission failures confuse users who just want to push code

**Solution:**
- Make permission enforcement opt-in only
- Repos without `.repobox/config.yml` = no permission rules at all
- Clear distinction between "managed" and "unmanaged" repositories

## Technical Specification

### Current Architecture Analysis

**Permission Check Flow (Current):**
```
1. User pushes to repo
2. Server reads `.repobox/config.yml` via `read_config_from_repo()`
3. If config exists → parse and enforce rules
4. If config missing/invalid → use default policy (allow/deny)
5. For read operations → same flow in `check_read_access()`
```

**Key Functions (Current):**
- `read_config_from_repo()` in `routes.rs` - reads config from git tree
- `check_read_access()` - enforces read permissions for clone/fetch
- `git::check_push_authorized()` - enforces push permissions

### Proposed Changes

#### 1. Modify Permission Check Logic

**File**: `repobox-server/src/routes.rs`

**Function**: `check_read_access()`

**Current Logic**:
```rust
let config_content = match read_config_from_repo(&repo_dir) {
    Some(content) => content,
    None => return Ok(()), // No config = public (default: allow)
};
```

**New Logic**:
```rust
let config_content = match read_config_from_repo(&repo_dir) {
    Some(content) => content,
    None => {
        tracing::debug!(repo = %format!("{}/{}", repo.address, repo.name), 
                       "no .repobox/config.yml found - skipping permission enforcement");
        return Ok(()); // No config = no permission enforcement (public access)
    }
};
```

#### 2. Update Push Authorization Logic

**File**: `repobox-server/src/routes.rs`

**Function**: `receive_pack()` around line 155-175

**Current Logic**:
```rust
if !matches!(
    git::check_push_authorized(&state.data_dir, &repo, &pusher, &record.owner_address),
    Ok(true)
) {
    // Unauthorized — revert would be ideal but for now just log.
    tracing::warn!(
        repo = %format!("{}/{}", repo.address, repo.name),
        pusher = %pusher,
        owner = %record.owner_address,
        "push denied: pusher not authorized"
    );
}
```

**New Logic**:
```rust
// Check if repository has opted into permission enforcement
let repo_dir = git::repo_dir(&state.data_dir, &repo);
if read_config_from_repo(&repo_dir).is_some() {
    // Has config = permission enforcement enabled
    if !matches!(
        git::check_push_authorized(&state.data_dir, &repo, &pusher, &record.owner_address),
        Ok(true)
    ) {
        tracing::warn!(
            repo = %format!("{}/{}", repo.address, repo.name),
            pusher = %pusher,
            owner = %record.owner_address,
            "push denied: pusher not authorized (config-enabled repo)"
        );
        // Future: return 403 error instead of just logging
    }
} else {
    // No config = no permission enforcement
    tracing::debug!(
        repo = %format!("{}/{}", repo.address, repo.name),
        pusher = %pusher,
        "push allowed: no .repobox/config.yml (permission enforcement disabled)"
    );
}
```

#### 3. Update Core Permission Function

**File**: `repobox-server/src/git.rs`

**Function**: `check_push_authorized()`

**Enhancement**: Add early return for repos without config

```rust
pub(crate) fn check_push_authorized(
    data_dir: &Path,
    repo: &RepoPath,
    pusher: &str,
    owner: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let repo_dir = repo_dir(data_dir, repo);
    
    // Check if repo has opted into permission enforcement
    let config_content = match crate::routes::read_config_from_repo(&repo_dir) {
        Some(content) => content,
        None => {
            // No config = no permission rules = allow push (owner-only via signature requirement)
            tracing::debug!(
                repo = %format!("{}/{}", repo.address, repo.name),
                "no config found - allowing push without permission checks"
            );
            return Ok(true);
        }
    };
    
    // Existing permission checking logic continues...
    let config = match repobox::parser::parse(&config_content) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(config_error = %e, "invalid config - allowing push");
            return Ok(true); // Invalid config = don't block
        }
    };
    
    // Rest of existing logic unchanged
    // ...
}
```

#### 4. Addressless Route Consistency

Ensure addressless routes (`/{repo}/git-receive-pack`) have the same opt-in logic as regular routes.

**File**: `repobox-server/src/routes.rs`

**Function**: `addressless_receive_pack()`

Apply the same config-checking logic as in `receive_pack()`.

### Database and Storage

**No database changes required** - the opt-in check is purely based on git tree content via `git show HEAD:.repobox/config.yml`.

### Configuration Schema

**Existing `.repobox/config.yml` format remains unchanged:**

```yaml
groups:
  founders:
    - evm:0x...
    
permissions:
  default: deny
  rules:
    - founders own >*
```

**New behavior**: 
- File exists = permission rules enforced as before
- File missing = no permission enforcement (public access with signature requirements)

## Implementation Plan

### Phase 1: Core Logic Changes (2-3 hours)

1. **Update `check_read_access()`** in `routes.rs`
   - Add debug logging for config presence
   - Early return for missing config

2. **Update `check_push_authorized()`** in `git.rs`  
   - Add config presence check
   - Early return for missing config
   - Update function signature if needed to access `read_config_from_repo`

3. **Update `receive_pack()`** in `routes.rs`
   - Add config presence check before authorization 
   - Consistent logging messages

4. **Update `addressless_receive_pack()`** in `routes.rs`
   - Mirror the changes from regular `receive_pack()`

### Phase 2: Testing and Validation (2-3 hours)

1. **Unit Tests**
   - Test repos with config → permission enforcement enabled
   - Test repos without config → no permission enforcement  
   - Test repos with invalid config → no permission enforcement

2. **Integration Tests**
   - Full push/clone flow with config-enabled repos
   - Full push/clone flow with non-config repos
   - Mixed scenarios (some repos with config, some without)

3. **Manual Testing**
   - Create test repo without `.repobox/config.yml`
   - Verify push succeeds regardless of signer identity
   - Verify clone works without authentication
   - Create test repo with config → verify old behavior

### Phase 3: Documentation and Deployment (1 hour)

1. **Update Server Documentation**
   - Document opt-in behavior in README
   - Update API documentation if needed

2. **Deployment**
   - Deploy to git.repo.box
   - Monitor logs for config presence/absence patterns
   - Verify existing configured repos still work

## Affected Files

### Primary Changes
```
repobox-server/src/routes.rs     # check_read_access(), receive_pack(), addressless_receive_pack()
repobox-server/src/git.rs        # check_push_authorized() enhancement
```

### Secondary Files
```
repobox-server/tests/smart_http.rs    # Update integration tests
repobox-core/tests/permissions.rs    # Update unit tests  
README.md                             # Document opt-in behavior
docs/SKILL.md                         # Update usage documentation
```

## Testing Strategy

### Unit Tests (repobox-server/tests/)

```rust
#[test]
fn test_no_config_allows_push() {
    // Create repo without .repobox/config.yml
    // Verify any EVM-signed push succeeds
    // Verify read access works without auth
}

#[test]  
fn test_with_config_enforces_permissions() {
    // Create repo with .repobox/config.yml
    // Verify permission rules are enforced
    // Verify unauthorized pushes fail
}

#[test]
fn test_invalid_config_allows_push() {
    // Create repo with malformed .repobox/config.yml
    // Verify pushes succeed (fallback behavior)
}
```

### Integration Tests

```bash
# Test 1: Non-configured repo
git init test-no-config
echo "# Simple repo" > README.md
git add . && git commit -S -m "initial commit"
git push https://git.repo.box/test-no-config.git main  # Should succeed

# Test 2: Configured repo  
git init test-with-config
mkdir .repobox
echo "permissions:\n  default: deny" > .repobox/config.yml
git add . && git commit -S -m "with config"
git push https://git.repo.box/test-with-config.git main  # Should enforce rules

# Test 3: Clone without auth
git clone https://git.repo.box/test-no-config.git      # Should succeed
git clone https://git.repo.box/test-with-config.git    # May fail if config denies reads
```

### Load Testing

```bash
# Create 100 repos: 50 with config, 50 without
# Concurrent push operations
# Verify performance doesn't degrade
# Verify config presence detection is efficient
```

## Acceptance Criteria

### ✅ Functional Requirements

1. **Repositories without `.repobox/config.yml`**:
   - ✅ Allow push from any EVM-signed identity (owner verification only)
   - ✅ Allow clone/fetch without authentication  
   - ✅ No permission rule enforcement
   - ✅ Server logs indicate "permission enforcement disabled"

2. **Repositories with `.repobox/config.yml`**:
   - ✅ Existing permission behavior unchanged
   - ✅ Rules enforced as before
   - ✅ Server logs indicate "permission enforcement enabled"

3. **Repositories with invalid `.repobox/config.yml`**:
   - ✅ Fall back to "no enforcement" mode
   - ✅ Log warning about invalid config
   - ✅ Don't block operations due to config errors

### ✅ Non-Functional Requirements

4. **Performance**:
   - ✅ Config presence check adds <10ms to operations
   - ✅ No performance regression for existing configured repos
   - ✅ Efficient git tree scanning

5. **Backward Compatibility**:
   - ✅ All existing repos with configs continue working identically
   - ✅ No breaking changes to API or git protocol
   - ✅ No database migration required

6. **Observability**:
   - ✅ Clear logging distinguishes configured vs non-configured repos
   - ✅ Metrics track adoption of permission configurations
   - ✅ Debug logs show config presence check results

## Security Considerations

### Threat Model

**Attack Vector**: Malicious actor removes `.repobox/config.yml` to bypass permissions

**Mitigation**: 
- Only applies to future pushes after config removal
- Historical commits retain signature verification
- Repository ownership still enforced via signed commits
- Removing config requires existing push permission

**Risk Level**: LOW - attacker would need existing push access to remove config

### Access Control Impact

**Before**: All repos have permission enforcement (with fallback policies)
**After**: Only repos with explicit config have permission enforcement

**Impact**: 
- ✅ Reduces attack surface for permission bypass bugs
- ✅ Clearer security model (opt-in is explicit)
- ✅ Performance improvement for simple repos
- ⚠️ Requires user education about when to add config

## Migration Strategy

### Existing Repositories

**No migration required** - all existing repos with `.repobox/config.yml` continue working identically.

### New Repository Recommendations

**Template: Simple repos**
```bash
git init my-simple-repo
# No .repobox/config.yml needed
# Push succeeds with any EVM signature
```

**Template: Managed repos**  
```bash
git init my-managed-repo
repobox init  # Creates .repobox/config.yml
# Permission rules now enforced
```

### Documentation Updates

1. **README.md**: Add section explaining opt-in behavior
2. **CLI help**: Update `repobox init` to mention permission enforcement
3. **Website**: Update docs to clarify when config is needed
4. **Examples**: Provide both simple and managed repo examples

## Rollback Plan

If issues arise, rollback involves reverting the early return statements:

```rust
// Rollback: change this
None => return Ok(()),

// Back to this  
None => {
    // Apply default policy instead of skipping enforcement
    let default_config = get_default_config(); 
    parse_and_enforce(default_config);
}
```

This restores the previous "always enforce" behavior.

## Metrics and Monitoring

### Server Metrics to Track

1. **Config Presence Rate**:
   - `repos_with_config_total` 
   - `repos_without_config_total`
   - Track adoption over time

2. **Operation Success Rates**:
   - `push_success_rate{config_enabled="true"}`
   - `push_success_rate{config_enabled="false"}`
   - Compare reliability

3. **Performance Metrics**:
   - `config_check_duration_ms` histogram
   - `permission_check_skipped_total` counter

### Log Analysis

**Key log messages to monitor**:
```
"no .repobox/config.yml found - skipping permission enforcement"
"push allowed: no config (permission enforcement disabled)"  
"push denied: pusher not authorized (config-enabled repo)"
```

**Alert conditions**:
- Sudden drop in `repos_with_config_total` (mass config removal)
- Increase in permission errors for previously working repos
- Performance degradation in config checking

## Future Enhancements

### Phase 2 Features (Post-Implementation)

1. **Config Migration Tool**:
   ```bash
   repobox migrate --enable-permissions
   # Adds default .repobox/config.yml to existing repos
   ```

2. **Permission Enforcement Dashboard**:
   - Show which repos have configs enabled
   - Aggregate permission deny/allow statistics
   - Config validation and recommendations

3. **Gradual Migration Support**:
   - Server flag to enable/disable opt-in behavior
   - A/B testing between enforcement modes
   - Per-repository override flags

### Advanced Security Features

1. **Config Tampering Detection**:
   - Alert when `.repobox/config.yml` is removed from protected repos
   - Require additional authorization for config changes

2. **Inheritance and Templates**:
   - Organization-wide default configs
   - Config template inheritance
   - Automatic config generation for new repos

## Implementation Checklist

### ✅ Development Tasks

- [ ] Update `check_read_access()` in `routes.rs`
- [ ] Update `check_push_authorized()` in `git.rs`  
- [ ] Update `receive_pack()` in `routes.rs`
- [ ] Update `addressless_receive_pack()` in `routes.rs`
- [ ] Add/update unit tests in `repobox-server/tests/`
- [ ] Add integration tests for opt-in behavior
- [ ] Update documentation (README, SKILL.md)

### ✅ Testing Tasks

- [ ] Test repos without config (no permission enforcement)
- [ ] Test repos with valid config (existing behavior)  
- [ ] Test repos with invalid config (fallback to no enforcement)
- [ ] Performance testing (config check overhead)
- [ ] Security testing (permission bypass attempts)
- [ ] Load testing (mixed configured/non-configured repos)

### ✅ Deployment Tasks

- [ ] Deploy to git.repo.box staging environment
- [ ] Verify existing configured repos still work
- [ ] Test with sample non-configured repos
- [ ] Monitor server logs for config presence patterns
- [ ] Production deployment with monitoring
- [ ] Update public documentation and examples

---

**Specification approved for implementation**  
**Next steps**: Move to "In Progress" in KANBAN.md and begin Phase 1 development.