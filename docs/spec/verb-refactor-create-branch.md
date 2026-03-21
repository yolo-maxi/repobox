# Spec: Verb Refactor - create → new files, branch → new branches

**Priority:** P1  
**Tags:** feature, permissions, breaking  
**Created:** 2026-03-21  
**Author:** PM Agent  

## Overview

This is a breaking change to repo.box verb semantics. Currently `create` means "create new branch". We're changing this to:

1. **`create`** = create new files (useful for append-only patterns where agents can add files but not modify existing ones)
2. **`branch`** = new verb for creating branches (replacing current `create` semantics)

This refactor enables more granular permissions where agents can be granted file creation rights without branch creation rights, supporting safer append-only workflows.

## Current State Analysis

### Current `create` Implementation

From `repobox-core/src/config.rs`:
```rust
pub enum Verb {
    // Access verbs
    Read,
    // Branch verbs
    Push,
    Merge,
    Create,    // ← Currently means "create branch"
    Delete,
    ForcePush,
    // File verbs
    Edit,
    Write,
    Append,
}
```

From `repobox-core/src/shim.rs`:
- `check_checkout()`: Uses `Verb::Create` for `git checkout -b`
- `check_branch()`: Uses `Verb::Create` for `git branch <name>`

From `repobox-core/src/parser.rs`:
- `OWN_WRITE_VERBS` includes `Verb::Create` (expansion of `own` keyword)

### Current Usage in Documentation

From `repobox-landing/content/docs/guide/configuration.md`:
```yaml
# Branch Verbs
- create    — create a new branch
```

From examples and SKILL.md:
```yaml
- agents create >feature/**  # agents create feature branches
```

## New Verb Semantics Design

### Post-Refactor Verb Categories

```rust
pub enum Verb {
    // Access verbs
    Read,
    // Branch verbs
    Push,
    Merge,
    Branch,     // ← NEW: create new branch (replaces current `create`)
    Delete,
    ForcePush,
    // File verbs
    Edit,
    Write,
    Append,
    Create,     // ← MOVED: create new files (new semantic)
}
```

### Verb Classification Updates

1. **`create` moves from branch verb to file verb**
2. **`branch` added as new branch verb**
3. **`is_branch_verb()` updated**
4. **`is_file_verb()` updated**

### Semantic Mapping

| Old Config | New Config | Meaning |
|------------|------------|---------|
| `agents create >feature/**` | `agents branch >feature/**` | Create branches matching `>feature/**` |
| N/A | `agents create src/**` | Create new files in `src/` directory |
| `agents own >feature/**` | `agents own >feature/**` | Full ownership (read + branch + push + merge + delete + force-push + edit + write + append + **create**) |

### Use Case Examples

#### Before (current):
```yaml
permissions:
  rules:
    - agents create >feature/**  # Can create feature branches
    - agents edit * >feature/**  # Can edit any files on feature branches
```

#### After (new):
```yaml
permissions:
  rules:
    - agents branch >feature/**  # Can create feature branches  
    - agents edit * >feature/**  # Can edit existing files on feature branches
    - agents create src/** >feature/**  # Can create new files in src/ on feature branches
```

#### New Append-Only Pattern:
```yaml
permissions:
  rules:
    - agents create logs/**      # Can create new log files
    - agents append logs/**      # Can append to existing log files
    # Note: no `edit` permission = can't modify existing files
```

## Implementation Plan

### Phase 1: Core Verb Changes

#### 1.1 Update Verb Enum (repobox-core/src/config.rs)

**Before:**
```rust
pub enum Verb {
    Read,
    Push, Merge, Create, Delete, ForcePush,  // Branch verbs
    Edit, Write, Append,                     // File verbs
}
```

**After:**
```rust
pub enum Verb {
    Read,
    Push, Merge, Branch, Delete, ForcePush,  // Branch verbs
    Edit, Write, Append, Create,             // File verbs
}
```

#### 1.2 Update Verb Parsing (repobox-core/src/config.rs)

**In `Verb::parse()`:**
- Add `"branch" => Ok(Verb::Branch)`
- Change `"create" => Ok(Verb::Create)` (now means create files)

#### 1.3 Update Verb Classification Methods

**In `is_branch_verb()`:**
```rust
pub fn is_branch_verb(self) -> bool {
    matches!(
        self,
        Verb::Push | Verb::Merge | Verb::Branch | Verb::Delete | Verb::ForcePush
    )
}
```

**In `is_file_verb()`:**
```rust  
pub fn is_file_verb(self) -> bool {
    matches!(self, Verb::Edit | Verb::Write | Verb::Append | Verb::Create)
}
```

#### 1.4 Update Display Implementation

**In `Display for Verb`:**
```rust
Verb::Branch => write!(f, "branch"),
Verb::Create => write!(f, "create"),
```

### Phase 2: Shim Updates

#### 2.1 Update Branch Creation Checks (repobox-core/src/shim.rs)

**In `check_checkout()`:**
```rust
// OLD:
let result = engine::check_with_resolver(config, identity, Verb::Create, Some(&branch_name), None, resolver);

// NEW:
let result = engine::check_with_resolver(config, identity, Verb::Branch, Some(&branch_name), None, resolver);
```

**In `check_branch()`:**
```rust
// OLD:
let result = engine::check_with_resolver(config, identity, Verb::Create, Some(branch), None, resolver);

// NEW:
let result = engine::check_with_resolver(config, identity, Verb::Branch, Some(branch), None, resolver);
```

#### 2.2 Add File Creation Checks

**In `check_commit()`:**
- Update `classify_staged_file()` logic
- When new file detected (currently returns `Verb::Write`), check `Verb::Create` permission
- If `Create` denied but `Write` allowed → block
- If `Create` allowed → proceed

**Proposed logic:**
```rust
// For newly created files
let create_result = engine::check_with_resolver(config, identity, Verb::Create, current_branch, Some(file), resolver);
let write_result = engine::check_with_resolver(config, identity, Verb::Write, current_branch, Some(file), resolver);
let edit_result = engine::check_with_resolver(config, identity, Verb::Edit, current_branch, Some(file), resolver);

// Hierarchy: Create > Write > Edit for new files
if create_result.is_allowed() || write_result.is_allowed() || edit_result.is_allowed() {
    // Check explicit denies
    if matches!(edit_result, engine::CheckResult::Deny { .. }) {
        return ShimAction::Block(format!("permission denied: {} cannot edit {file}", identity));
    }
    continue;
}
```

### Phase 3: Parser Updates

#### 3.1 Update OWN_WRITE_VERBS

**In `repobox-core/src/parser.rs`:**

**Before:**
```rust
const OWN_WRITE_VERBS: &[Verb] = &[
    Verb::Push, Verb::Merge, Verb::Create, Verb::Delete, Verb::ForcePush,
    Verb::Edit, Verb::Write, Verb::Append,
];
```

**After:**
```rust
const OWN_WRITE_VERBS: &[Verb] = &[
    Verb::Push, Verb::Merge, Verb::Branch, Verb::Delete, Verb::ForcePush,
    Verb::Edit, Verb::Write, Verb::Append, Verb::Create,
];
```

This ensures `own` keyword expands to include both `branch` (create branches) and `create` (create files).

### Phase 4: Documentation Updates

#### 4.1 Configuration Guide (repobox-landing/content/docs/guide/configuration.md)

**Update verb sections:**

```markdown
### Branch Verbs

- `push` — push commits to a branch
- `merge` — merge into a branch  
- `branch` — create a new branch
- `delete` — delete a branch
- `force-push` — rewrite history

### File Verbs

- `edit` — full modification (add, change, remove lines)
- `write` — add lines only, no deletions
- `append` — add lines at end of file only
- `create` — create new files
```

#### 4.2 Examples (repobox-landing/content/docs/guide/examples.md)

**Update all instances of:**
- `create >feature/**` → `branch >feature/**`
- Add examples showing new `create` verb for files

**Example new config:**
```yaml
agents:
  branch:
    - ">feature/**"     # Can create feature branches
  create:
    - "src/**"          # Can create files in src/
    - "tests/**"        # Can create test files
  append:
    - "logs/**"         # Can append to logs
```

#### 4.3 SKILL.md (repobox-landing/public/SKILL.md)

**Update verb table:**
```markdown
| `branch` | Create a new branch |
| `create` | Create new files |
```

**Update examples:**
```yaml
- founders branch >*                    # founders create any branch  
- agents:
    branch:
      - ">feature/**"                   # agents create feature branches
    create:
      - "src/**"                        # agents create source files
```

### Phase 5: Tests

#### 5.1 Update Existing Tests

**In `repobox-core/src/parser.rs` tests:**
- Update tests using `create >branch` → `branch >branch`
- Add tests for `create file` semantics

**In `repobox-core/src/shim.rs` tests:**
- Update `test_checkout_create_feature_allowed()` → verify `Verb::Branch`
- Update `test_agent_create_feature_allowed()` → verify `Verb::Branch`
- Add tests for file creation permissions

#### 5.2 Add New Tests

**Parser tests:**
```rust
#[test]
fn test_create_parses_as_file_verb() {
    let verb = Verb::parse("create").unwrap();
    assert_eq!(verb, Verb::Create);
    assert!(verb.is_file_verb());
    assert!(!verb.is_branch_verb());
}

#[test]  
fn test_branch_parses_as_branch_verb() {
    let verb = Verb::parse("branch").unwrap();
    assert_eq!(verb, Verb::Branch);
    assert!(verb.is_branch_verb());
    assert!(!verb.is_file_verb());
}

#[test]
fn test_own_expands_to_include_both_create_and_branch() {
    let yaml = r#"
groups:
  owners: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - owners own >main
"#;
    let config = parse(yaml).unwrap();
    // Should have 10 rules total (read + 9 write verbs including both branch and create)
    assert_eq!(config.permissions.rules.len(), 10);
    
    let verbs: Vec<_> = config.permissions.rules.iter().map(|r| r.verb).collect();
    assert!(verbs.contains(&Verb::Branch));
    assert!(verbs.contains(&Verb::Create));
}
```

**Shim tests:**
```rust
#[test]
fn test_file_creation_requires_create_permission() {
    // Test that creating new files checks Verb::Create
}

#[test]
fn test_branch_creation_uses_branch_verb() {
    // Test that git checkout -b uses Verb::Branch
}
```

**Engine tests:**
```rust
#[test]
fn test_create_file_vs_branch_permissions() {
    let yaml = r#"
groups:
  developers: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - developers create src/**     # Can create files in src/
    - developers not branch >*     # Cannot create branches
"#;
    let config = parse(yaml).unwrap();
    let dev = Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap();
    
    // Can create files
    let result = engine::check(&config, &dev, Verb::Create, None, Some("src/new.rs"));
    assert!(result.is_allowed());
    
    // Cannot create branches  
    let result = engine::check(&config, &dev, Verb::Branch, Some("feature/new"), None);
    assert!(!result.is_allowed());
}
```

### Phase 6: Migration Strategy

#### 6.1 Breaking Change Communication

**This is a BREAKING CHANGE** that requires config migration. Existing configurations using `create` for branch creation will break.

#### 6.2 Migration Tool

Create `repobox migrate-config` subcommand:

```rust
// In CLI
pub fn migrate_config(config_path: &Path) -> Result<(), MigrationError> {
    let content = fs::read_to_string(config_path)?;
    let migrated = migrate_config_content(&content)?;
    
    // Backup original
    fs::write(config_path.with_extension("yml.backup"), &content)?;
    
    // Write migrated version
    fs::write(config_path, migrated)?;
    
    Ok(())
}

fn migrate_config_content(yaml: &str) -> Result<String, MigrationError> {
    // Parse as YAML, walk the tree, replace:
    // - "create >" → "branch >"
    // - "not create >" → "not branch >"  
    // - create: [">feature/**"] → branch: [">feature/**"]
    // etc.
}
```

#### 6.3 Migration Detection

Add lint warning for pre-migration configs:

```rust
pub fn lint(config: &Config) -> Vec<String> {
    let mut warnings = Vec::new();
    
    for rule in &config.permissions.rules {
        if rule.verb == Verb::Create && rule.target.branch.is_some() {
            warnings.push(format!(
                "Line {}: 'create' with branch target '{}' should be migrated to 'branch' verb. Run: repobox migrate-config",
                rule.line,
                rule.target.branch.as_ref().unwrap()
            ));
        }
    }
    
    warnings
}
```

#### 6.4 Version Detection

Add version marker to detect old configs:

```yaml
# Old configs (no version marker) use legacy create = branch semantics
# New configs explicitly declare version  
version: 2
groups:
  agents: [...]
permissions:
  rules:
    - agents branch >feature/**  # New syntax
```

If no `version` field detected, warn about migration needed.

### Phase 7: Error Messages

#### 7.1 Update Error Messages

**Branch creation errors:**
```rust
// OLD:
"permission denied: {} cannot create branch {branch_name}"

// NEW:  
"permission denied: {} cannot branch {branch_name}"
```

**New file creation errors:**
```rust
"permission denied: {} cannot create {file}"
```

#### 7.2 Migration Error Messages

```rust
"Error: This config uses legacy 'create' verb for branches. Run 'repobox migrate-config' to update to new syntax."
```

## Files Requiring Changes

### Core Library (`repobox-core/`)
1. **`src/config.rs`**
   - Update `Verb` enum
   - Update `Verb::parse()`  
   - Update `is_branch_verb()` and `is_file_verb()`
   - Update `Display for Verb`

2. **`src/parser.rs`**
   - Update `OWN_WRITE_VERBS`
   - Add migration detection logic
   - Update test cases

3. **`src/shim.rs`**
   - Update `check_checkout()` to use `Verb::Branch`
   - Update `check_branch()` to use `Verb::Branch`
   - Update `check_commit()` to check `Verb::Create` for new files
   - Update error messages
   - Update test cases

4. **`src/engine.rs`**
   - Add test cases for new verb semantics

5. **`src/lint.rs`**
   - Add migration warnings

### CLI (`repobox-cli/`)
6. **CLI commands**
   - Add `repobox migrate-config` subcommand

### Documentation
7. **`repobox-landing/content/docs/guide/configuration.md`**
   - Update verb documentation

8. **`repobox-landing/content/docs/guide/examples.md`**
   - Update all examples using old `create` syntax

9. **`repobox-landing/public/SKILL.md`**
   - Update verb table
   - Update examples

### Landing Page
10. **Playground examples**
    - Update any hardcoded examples using `create >branch`

## Testing Strategy

### 1. Unit Tests
- All existing parser tests pass with updated expectations
- All existing shim tests pass with updated verb usage
- New tests for create vs branch verb semantics

### 2. Integration Tests  
- End-to-end git workflows with new verb semantics
- Migration tool testing

### 3. Breaking Change Tests
- Verify old configs with `create >branch` are detected as needing migration
- Verify migrated configs work correctly

### 4. Regression Tests
- Verify `own` verb still works correctly (includes both `branch` and `create`)
- Verify all non-`create` verbs unchanged

## Rollout Plan

### Phase 1: Implementation
1. Implement core verb changes
2. Update tests  
3. Update documentation

### Phase 2: Migration Tools
1. Implement `migrate-config` command
2. Add lint warnings for old configs
3. Test migration on known configs

### Phase 3: Release
1. Document breaking change prominently
2. Provide migration guide
3. Update examples and tutorials

### Phase 4: Adoption
1. Migrate repo.box's own configs
2. Help early adopters migrate
3. Monitor for issues

## Risk Assessment

### High Risk
- **Breaking change** - all existing configs using `create` for branches will break
- **Semantic confusion** - users may expect old behavior

### Mitigation
- Clear migration documentation
- Automated migration tool
- Lint warnings for old configs
- Version detection for backwards compatibility detection

### Medium Risk
- **Test coverage gaps** - new file permission logic is complex
- **Edge cases** - interaction between create/write/edit permissions

### Mitigation  
- Comprehensive test suite
- Gradual rollout
- Clear permission hierarchy documentation

## Success Metrics

1. **Migration tool works**: Can successfully migrate existing configs
2. **No regressions**: All existing functionality works with new verb names
3. **New functionality works**: File creation permissions work as designed  
4. **Documentation accurate**: All examples and docs reflect new syntax
5. **User adoption**: Users successfully migrate without issues

## Timeline Estimate

- **Core implementation**: 2-3 days
- **Testing**: 1-2 days  
- **Documentation updates**: 1 day
- **Migration tooling**: 1-2 days
- **Total**: 5-8 days

---

**End of Specification**

This specification provides a comprehensive roadmap for refactoring the `create` verb semantics while maintaining system integrity and providing a clear migration path for existing users.