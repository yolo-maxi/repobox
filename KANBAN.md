# repo.box — Git Permission Layer for AI Agents

## 💡 Ideas

### ENS support for clone URLs
- **Priority**: P2
- **Tags**: feature, ux
Clone via `git.repo.box/vitalik.eth/myrepo.git` — resolve ENS to address server-side.

### Multi-branch support in explorer
- **Priority**: P2
- **Tags**: feature, explorer
Branch selector dropdown in repo detail page. Currently only shows `main`.

### Diff viewer in explorer
- **Priority**: P3
- **Tags**: feature, explorer
Click a commit → see the diff. Colored additions/deletions.

### Webhook notifications on push
- **Priority**: P3
- **Tags**: feature, server
POST to a URL when someone pushes. Useful for CI/CD integration.

### `repobox verify` command
- **Priority**: P2
- **Tags**: feature, cli
Verify all commits in a repo are properly signed. Show signer address per commit.

### Token-gated repos
- **Priority**: P2
- **Tags**: feature, server
Use on-chain resolver to gate read access. Hold X tokens to clone.

### Activity feed on explorer home
- **Priority**: P1
- **Tags**: feature, explorer
The "Recent Activity" column shows "No recent activity" — wire up the push log.

## 📋 Backlog

### Explorer: show signer address per commit (not just owner)
- **Priority**: P1
- **Tags**: explorer
Each commit should show which EVM address signed it. Different agents = different addresses visible.



## 🔨 In Progress

### Enforce .repobox-config opt-in on server
- **Priority**: P2
- **Tags**: server
- **Assignee**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b)
- **Status**: Specification Complete, Ready for Implementation

Change server permission enforcement from "all repos" to "opt-in only". Repositories must contain `.repobox/config.yml` in their tree to have permission rules enforced. Repos without this config file will have no permission enforcement (public read/write with signature requirements only).

  **DETAILED SPECIFICATION**: [`docs/specs/config-opt-in.md`](./docs/specs/config-opt-in.md)

  #### Implementation Summary

  **Problem**: Currently all repos have permission enforcement, which creates friction for simple repositories that don't need complex access controls.

  **Solution**: Make permission enforcement opt-in by checking for `.repobox/config.yml` presence:
  - **Repos WITH config** → Permission rules enforced as before  
  - **Repos WITHOUT config** → No permission enforcement (public access with EVM signature requirements only)

  #### Architecture Changes

  **Core Logic Updates:**
  1. `check_read_access()` in `routes.rs` - early return when no config found
  2. `check_push_authorized()` in `git.rs` - skip permission checks for non-configured repos  
  3. `receive_pack()` and `addressless_receive_pack()` - consistent opt-in behavior

  **Key Benefits:**
  - ✅ Reduces complexity for simple repositories
  - ✅ Clear distinction between managed vs unmanaged repos
  - ✅ Performance improvement for non-configured repos
  - ✅ No breaking changes to existing configured repos

  #### Files to Modify
  ```
  repobox-server/src/routes.rs     # check_read_access(), receive_pack()
  repobox-server/src/git.rs        # check_push_authorized() enhancement  
  repobox-server/tests/*.rs        # Updated unit/integration tests
  README.md                        # Document opt-in behavior
  ```

  #### Testing Strategy
  - **Unit tests**: repos with/without config, invalid config scenarios
  - **Integration tests**: full push/clone flows for both repo types
  - **Performance tests**: verify no regression for existing repos
  - **Security tests**: verify permission bypass protections

  #### Implementation Phases
  1. **Phase 1** (2-3h): Core logic changes in routes.rs and git.rs
  2. **Phase 2** (2-3h): Comprehensive testing and validation  
  3. **Phase 3** (1h): Documentation and production deployment

  **Specced by**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b) | 2026-03-21

### Add .repobox/config.yml to all studio projects
- **Priority**: P1
- **Tags**: dogfood
Push SSS, Oceangram, and other repos to git.repo.box with signed commits + configs.

### Wire up activity feed from push events
- **Priority**: P1
- **Tags**: explorer, server
Server needs to log pushes to a table, API needs to return them. Explorer already has the UI.

  **DETAILED SPECIFICATION:**

  #### Database Schema Design
  
  The `push_log` table already exists in the SQLite database with the correct schema:
  ```sql
  CREATE TABLE IF NOT EXISTS push_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL,           -- Repository owner EVM address  
    name TEXT NOT NULL,              -- Repository name
    pusher_address TEXT,             -- EVM address who made the push
    commit_hash TEXT,                -- Latest commit hash from the push
    commit_message TEXT,             -- Latest commit message
    pushed_at TEXT NOT NULL          -- ISO timestamp of push event
  );
  ```
  
  **Indexes needed for performance:**
  ```sql
  CREATE INDEX IF NOT EXISTS idx_push_log_timestamp ON push_log(pushed_at DESC);
  CREATE INDEX IF NOT EXISTS idx_push_log_repo ON push_log(address, name);
  ```

  #### Architecture Overview
  
  **Current State:**
  - ✅ UI exists: `web/src/app/explore/page.tsx` shows activity feed
  - ✅ API exists: `web/src/app/api/explorer/activity/route.ts` queries push_log 
  - ✅ Database helper: `web/src/lib/database.ts` with runQuery function
  - ✅ Table schema: push_log table auto-created on first query
  - ❌ **Missing**: Server-side logging when pushes happen
  
  **Integration Points:**
  1. **Server logging**: Add push_log inserts in `repobox-server` 
  2. **API optimization**: Add database indexes for query performance
  3. **UI enhancement**: Activity feed already consuming the API correctly

  #### Implementation Plan

  ##### Phase 1: Database Schema Updates
  **Files to modify:**
  - `repobox-server/src/db.rs`
    - Add `create_push_log_indexes()` function
    - Add `insert_push_log()` function
    - Update `init()` to create indexes
  
  **Database functions needed:**
  ```rust
  pub(crate) fn insert_push_log(
      db_path: &Path,
      address: &str,
      name: &str, 
      pusher_address: Option<&str>,
      commit_hash: Option<&str>,
      commit_message: Option<&str>,
  ) -> std::io::Result<()>
  
  fn create_push_log_indexes(connection: &Connection) -> Result<(), rusqlite::Error>
  ```

  ##### Phase 2: Push Event Detection & Logging
  **Files to modify:**
  - `repobox-server/src/routes.rs`
    - Add logging calls in `receive_pack()` function
    - Add logging calls in `addressless_receive_pack()` function
  - `repobox-server/src/git.rs`  
    - Add helper function `extract_latest_commit_info()` to get commit details
    - Update existing extraction functions to return commit message

  **Integration points:**
  1. **After successful push**: Log when `git receive-pack` completes successfully
  2. **Extract commit details**: Get hash & message from HEAD after push
  3. **Extract pusher**: Use existing `extract_pusher_from_head()` function
  4. **Handle both routes**: Regular `/{address}/{repo}` and addressless `/{repo}`

  #### Detailed Implementation Guide

  ##### Step 1: Database Functions (`repobox-server/src/db.rs`)
  
  **Add push_log table creation to `init()`:**
  ```rust
  pub(crate) fn init(db_path: &Path) -> std::io::Result<()> {
      let connection = Connection::open(db_path).map_err(to_io_error)?;
      
      // Existing repos table
      connection.execute(/* existing repos table SQL */, []).map_err(to_io_error)?;
      
      // Add push_log table
      connection.execute(
          "CREATE TABLE IF NOT EXISTS push_log (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              address TEXT NOT NULL,
              name TEXT NOT NULL,
              pusher_address TEXT,
              commit_hash TEXT,
              commit_message TEXT,
              pushed_at TEXT NOT NULL
          )",
          [],
      ).map_err(to_io_error)?;
      
      // Create indexes for performance  
      create_push_log_indexes(&connection).map_err(to_io_error)?;
      Ok(())
  }
  
  fn create_push_log_indexes(connection: &Connection) -> Result<(), rusqlite::Error> {
      connection.execute(
          "CREATE INDEX IF NOT EXISTS idx_push_log_timestamp ON push_log(pushed_at DESC)",
          [],
      )?;
      connection.execute(
          "CREATE INDEX IF NOT EXISTS idx_push_log_repo ON push_log(address, name)",
          [],
      )?;
      Ok(())
  }
  ```

  **Add push logging function:**
  ```rust
  pub(crate) fn insert_push_log(
      db_path: &Path,
      address: &str,
      name: &str,
      pusher_address: Option<&str>,
      commit_hash: Option<&str>,
      commit_message: Option<&str>,
  ) -> std::io::Result<()> {
      let connection = Connection::open(db_path).map_err(to_io_error)?;
      connection
          .execute(
              "INSERT INTO push_log(address, name, pusher_address, commit_hash, commit_message, pushed_at)
               VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
              params![
                  address,
                  name,
                  pusher_address,
                  commit_hash,
                  commit_message,
                  now_string()
              ],
          )
          .map_err(to_io_error)?;
      Ok(())
  }
  ```

  ##### Step 2: Commit Info Extraction (`repobox-server/src/git.rs`)

  **Add commit info extraction helper:**
  ```rust
  #[derive(Debug, Clone)]
  pub(crate) struct CommitInfo {
      pub hash: String,
      pub message: String,
      pub pusher_address: Option<String>,
  }
  
  /// Extract commit hash, message, and signer from HEAD after a push
  pub(crate) fn extract_latest_commit_info(data_dir: &Path, repo: &RepoPath) -> std::io::Result<Option<CommitInfo>> {
      let repo_dir = repo_dir(data_dir, repo);
      let repo_dir_str = repo_dir.to_string_lossy().to_string();

      // Get HEAD commit hash
      let output = Command::new("git")
          .args(["--git-dir", &repo_dir_str, "rev-parse", "HEAD"])
          .output()?;

      if !output.status.success() {
          return Ok(None);
      }

      let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

      // Get commit message (first line only for activity feed)
      let output = Command::new("git")
          .args(["--git-dir", &repo_dir_str, "log", "--format=%s", "-1", "HEAD"])
          .output()?;

      let message = if output.status.success() {
          String::from_utf8_lossy(&output.stdout).trim().to_string()
      } else {
          String::new()
      };

      // Get pusher (signer) address
      let pusher_address = extract_pusher_from_head(data_dir, repo)?;

      Ok(Some(CommitInfo {
          hash,
          message,
          pusher_address,
      }))
  }
  ```

  ##### Step 3: Push Logging Integration (`repobox-server/src/routes.rs`)

  **Modify `receive_pack()` function:**
  ```rust
  async fn receive_pack(
      State(state): State<Arc<AppState>>,
      Path((address, repo)): Path<(String, String)>,
      headers: HeaderMap,
      body: Bytes,
  ) -> Response {
      let repo = match repo_path(address, repo) {
          Ok(repo) => repo,
          Err(status) => return status.into_response(),
      };

      if let Err(error) = ensure_repo_initialized(&state, &repo) {
          return internal_error(error);
      }

      let response = backend_post(
          &state,
          &repo,
          "/git-receive-pack",
          header_value(&headers, "content-type"),
          body,
      );

      // EXISTING ownership check code...

      // NEW: Log the push event after successful processing
      if response.status() == StatusCode::OK {
          if let Ok(Some(commit_info)) = git::extract_latest_commit_info(&state.data_dir, &repo) {
              let _ = db::insert_push_log(
                  &state.db_path,
                  &repo.address,
                  &repo.name,
                  commit_info.pusher_address.as_deref(),
                  Some(&commit_info.hash),
                  Some(&commit_info.message),
              );
              
              tracing::info!(
                  repo = %format!("{}/{}", repo.address, repo.name),
                  pusher = ?commit_info.pusher_address,
                  commit = %commit_info.hash[..8],
                  "push logged to activity feed"
              );
          }
      }

      response
  }
  ```

  **Similar modifications for `addressless_receive_pack()`:**
  - Add the same logging logic after successful push processing
  - Handle the case where the repo gets moved from staging to final location

  #### Error Handling & Edge Cases

  **Database Errors:**
  - Push logging failures should NOT block the push operation
  - Log database errors but continue normal git operation
  - Use `let _ =` to explicitly ignore logging errors

  **Commit Extraction Failures:**
  - Handle repos with no commits (should not happen after successful push)
  - Handle unsigned commits (pusher_address = NULL)
  - Handle empty/malformed commit messages

  **Concurrent Pushes:**
  - SQLite handles concurrent inserts with AUTOINCREMENT primary key
  - No additional locking needed

  **Performance Considerations:**
  - Database inserts are fast (<1ms typically)
  - Indexes ensure efficient querying
  - Commit info extraction reuses existing git commands

  #### Testing Strategy

  **Unit Tests (`repobox-server/src/db.rs`):**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use tempdir::TempDir;

      #[test]
      fn test_insert_and_query_push_log() {
          let temp_dir = TempDir::new("test_db").unwrap();
          let db_path = temp_dir.path().join("test.db");
          
          init(&db_path).unwrap();
          
          insert_push_log(
              &db_path,
              "0x1234",
              "test-repo",
              Some("0x5678"),
              Some("abcd1234"),
              Some("feat: initial commit"),
          ).unwrap();
          
          // Verify via sqlite3 command or rusqlite query
      }
  }
  ```

  **Integration Tests:**
  - Test full push → log → API query flow
  - Test both regular and addressless push routes  
  - Test with signed and unsigned commits
  - Test database index performance with large datasets

  #### API Enhancements (Future)

  **Current API is sufficient but could be enhanced:**
  - Add filtering by repository: `GET /api/explorer/activity?repo=owner/name`
  - Add filtering by pusher: `GET /api/explorer/activity?pusher=0x1234`
  - Add pagination: `GET /api/explorer/activity?offset=20&limit=10`
  - Add date ranges: `GET /api/explorer/activity?since=2026-01-01`

  #### Acceptance Criteria (Definition of Done)

  **Database Layer:**
  - ✅ `push_log` table exists with proper schema
  - ✅ Database indexes created for performance (timestamp, repo)  
  - ✅ `insert_push_log()` function implemented and tested
  - ✅ Database errors don't block git operations

  **Server Integration:**
  - ✅ Push events logged after successful `receive_pack` operations
  - ✅ Both regular and addressless push routes log activity
  - ✅ Commit hash, message, and pusher extracted correctly
  - ✅ Unsigned commits handled gracefully (pusher_address = NULL)
  - ✅ Logging failures don't break git operations

  **API & UI:**
  - ✅ Activity API returns push_log data correctly (already works)
  - ✅ Explorer page shows "Recent Activity" instead of "No recent activity"  
  - ✅ Activity items link to repository pages correctly
  - ✅ Timestamps display in "X minutes ago" format

  **Performance:**
  - ✅ Push operations complete in same time as before (<500ms typically)
  - ✅ Activity API queries complete in <100ms with indexes
  - ✅ No memory leaks or resource issues under load

  **Testing:**
  - ✅ Unit tests for database functions
  - ✅ Integration tests for full push → activity flow  
  - ✅ Manual testing with multiple repos and users
  - ✅ Load testing with concurrent pushes

  #### Implementation Checklist

  **Phase 1: Database Functions**
  - [ ] Update `repobox-server/src/db.rs` with push_log functions
  - [ ] Add indexes creation to `init()` function  
  - [ ] Write unit tests for database operations
  - [ ] Test database performance with sample data

  **Phase 2: Git Integration** 
  - [ ] Add `extract_latest_commit_info()` to `git.rs`
  - [ ] Update `receive_pack()` in `routes.rs` to log pushes
  - [ ] Update `addressless_receive_pack()` to log pushes  
  - [ ] Handle staging-to-final repo moves correctly

  **Phase 3: Testing & Validation**
  - [ ] Write integration tests for push logging
  - [ ] Manual testing with repobox CLI
  - [ ] Verify activity feed displays correctly
  - [ ] Performance testing under load

  **Phase 4: Documentation & Deployment**
  - [ ] Update server documentation
  - [ ] Deploy to git.repo.box with database migration
  - [ ] Monitor logs for any issues
  - [ ] Verify production activity feed works

  #### Dependencies & Considerations

  **No Breaking Changes:**
  - Database schema is additive (new table + indexes)
  - Server API remains unchanged
  - Git protocol compatibility maintained

  **Deployment Notes:**
  - Database migration automatic (CREATE TABLE IF NOT EXISTS)
  - No downtime required for deployment
  - Activity feed will populate from new pushes only (no historical data)

  **Future Enhancements:**
  - Could backfill historical activity from git log analysis
  - Could add webhook notifications for activity events
  - Could add activity filtering and search
  - Could add RSS feed for public repositories

  **Specced by**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b) | 2026-03-21

### Full E2E demo script
- **Priority**: P0
- **Tags**: hackathon, demo
Script that runs the complete flow: `repobox init` → `keys generate` → signed commit → push → clone → verify on explorer. For the hackathon presentation.

  **DETAILED SPECIFICATION:**
  
  #### Acceptance Criteria (Definition of Done)
  - Single executable shell script (`scripts/demo-e2e.sh`) that runs entire flow
  - Demo succeeds in clean environment (fresh temp directory, no existing git config)
  - Script includes visual progress indicators and clear output messages
  - All steps are fully automated with no manual intervention required
  - Script validates success at each step and exits with clear error if anything fails
  - Final output includes direct links to view results on explorer
  - Demo completes in under 60 seconds on typical hardware
  - Script can be run multiple times without conflicts (creates unique repo names)
  
  #### Files to Create/Modify
  ```
  scripts/demo-e2e.sh           # Main demo script (new file)
  scripts/demo-reset.sh         # Cleanup script for repeated runs (new file)  
  docs/DEMO.md                  # Demo instructions and variations (new file)
  .repobox/config.yml.template  # Template config for demo repos (new file)
  ```
  
  #### Step-by-Step Implementation Guide
  
  **1. Environment Setup (scripts/demo-e2e.sh)**
  - Check prerequisites: `repobox` binary exists at `/home/xiko/repobox/target/release/repobox`
  - Check git.repo.box server is responsive (curl test)
  - Check repo.box explorer is accessible
  - Create unique temp directory with timestamp: `/tmp/repobox-demo-YYYYMMDD-HHMMSS`
  - Export PATH to include repobox binary
  
  **2. Demo Repository Creation**
  - `cd` into temp directory
  - `git init demo-hackathon-$(date +%s)` (unique name with timestamp)
  - Create initial demo content:
    - `README.md` with repo.box explanation and demo timestamp
    - `src/hello.py` with simple Python "Hello repo.box" script
    - `src/agent-example.js` showing mock AI agent code
    - `.gitignore` with common patterns
  
  **3. repo.box Initialization**
  - `repobox init` to create `.repobox/config.yml`
  - Copy from template and customize for demo (show 3 groups: founders, agents, bots)
  - Show config contents with syntax highlighting
  
  **4. Identity Management**
  - `repobox keys generate --alias demo-founder` 
  - `repobox keys generate --alias demo-agent`
  - `repobox identity set demo-founder` (set as primary)
  - `repobox whoami` to confirm identity
  - Update `.repobox/config.yml` groups section with actual generated addresses
  
  **5. Signed Commit & Push Flow**
  - `git add .`
  - `git commit -m "feat: initial demo repository setup [demo-founder]"`
  - Show commit signature verification: `git log --show-signature -1`
  - `git remote add origin https://git.repo.box/demo-hackathon-$(timestamp).git`
  - `git push -u origin main` 
  - Capture and display the repository owner address from push output
  
  **6. Agent Simulation**
  - Switch identity: `repobox identity set demo-agent`
  - Create feature branch: `git checkout -b feature/agent-improvement`
  - Modify `src/agent-example.js` (show agents can edit files on feature branches)
  - `git add . && git commit -m "feat: enhanced agent capabilities [demo-agent]"`
  - `git push origin feature/agent-improvement`
  
  **7. Verification & Clone Test**
  - Clone from server in separate directory: 
    ```bash
    cd /tmp && git clone https://git.repo.box/demo-hackathon-$(timestamp).git verified-clone
    cd verified-clone && git log --show-signature --oneline
    ```
  - Verify all commits show valid EVM signatures
  - Show branch structure: `git branch -a`
  
  **8. Explorer Navigation**
  - Generate direct explorer URLs for demo repo
  - Display QR code for mobile viewing (optional enhancement)
  - Show key explorer features:
    - Repository overview with owner address
    - Commit history with signer addresses
    - File browser showing `.repobox/config.yml`
    - Config tab showing parsed permission rules
  
  #### Test Plan & Verification
  
  **Automated Tests in Script:**
  - Each step must exit 0 or script stops with clear error
  - Verify `repobox whoami` returns expected identity after each switch
  - Verify `git log --show-signature` shows valid signatures  
  - Verify clone succeeds and contains expected files
  - Verify explorer URLs return HTTP 200
  
  **Manual Verification Steps:**
  - Run script 3 times in sequence (test repeatability)
  - Verify each run creates unique repo names
  - Check explorer shows multiple demo repos
  - Verify permission enforcement: try agent push to main (should fail)
  - Test different clone URLs work from external machine
  
  **Performance Benchmarks:**
  - Demo script completes under 60 seconds
  - Clone operations complete under 10 seconds
  - Explorer page loads under 3 seconds
  
  #### Edge Cases & Error Handling
  
  **Network Issues:**
  - Test with git.repo.box server down (should fail gracefully)
  - Test with explorer down (should warn but continue)
  - Test with slow network (include timeouts)
  
  **Permission Issues:**
  - Test agent trying to push to main branch (should be denied)
  - Test invalid signature scenarios
  - Test missing `.repobox/config.yml` (should use defaults)
  
  **File System Issues:**
  - Test insufficient disk space
  - Test permission denied on temp directory creation
  - Test binary not found scenarios
  
  **Git State Issues:**
  - Test in directory with existing git repo
  - Test with existing repobox identity set
  - Test with existing aliases that conflict
  
  **Recovery Scenarios:**
  - Script interrupted mid-execution (cleanup temp files)
  - Multiple concurrent demo runs (unique naming)
  - Demo artifacts left behind from previous runs
  
  #### Demo Variations (docs/DEMO.md)
  
  **Quick Demo (30 seconds):**
  - Skip agent simulation, just show founder flow
  - Pre-generated identities, focus on push/clone/explorer
  
  **Full Demo (60 seconds):**
  - Complete flow as specified above
  - Show permission enforcement in action
  
  **Interactive Demo:**
  - Pause points for explanation
  - Manual verification steps
  - Audience Q&A integration
  
  **Debug Mode:**
  - Verbose output with timing information
  - Intermediate file contents displayed
  - Step-by-step confirmation prompts
  
  #### Output Format & Logging
  
  **Progress Indicators:**
  ```bash
  🔧 Setting up demo environment...
  🔑 Generating demo identities...  
  📦 Creating demo repository...
  ✅ Pushing to git.repo.box...
  🌐 Verifying on explorer...
  ✨ Demo complete!
  ```
  
  **Final Summary:**
  ```
  ===============================================
  repo.box E2E Demo Results
  ===============================================
  Repository: demo-hackathon-1710960180
  Owner: 0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b
  Commits: 2 (all signed)
  Branches: main, feature/agent-improvement
  
  🌐 View on Explorer:
  https://repo.box/explore/0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b/demo-hackathon-1710960180
  
  📋 Config Rules Applied:
  - founders: push/merge to any branch
  - agents: feature branches only  
  - all commits: EVM-signed and verified
  
  ⏱️  Demo completed in 47 seconds
  ===============================================
  ```

  **Specced by**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b) | 2026-03-20

  **IMPLEMENTATION UPDATE** (2026-03-21):
  ✅ **Core script implemented** - `scripts/demo-e2e.sh` created and tested
  ✅ **Both modes working** - Quick (30s) and Full (60s) modes functional
  ✅ **Multi-agent simulation** - Identity switching and feature branches
  ✅ **Push verification** - Both main and feature branch pushes successful
  ✅ **Visual progress indicators** - Colored terminal output with emojis
  ✅ **Error handling** - Graceful failures with cleanup
  ✅ **Unique naming** - Timestamp-based repo names prevent conflicts
  
  **Status**: Ready for review and merge. Clone verification has retry logic but may need server configuration adjustments.
  
  **Implemented by**: claude-agent (0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00) | Branch: feature/e2e-demo

## 🚧 Blocked

### Gmail token expired — can't send emails
- **Blocked by**: Fran re-auth
- **Tags**: infra

## ✅ Done

### Explorer: signer address per commit
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
ECDSA signature extraction from REPOBOX SIGNATURE blocks, address recovery via @noble/curves, owner vs collaborator badges in commit list UI.

### Install script + release pipeline
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
install.sh rewrite with platform detection, SHA256 checksums, sudo fallback, version pinning. Plus tools/release.sh (cross-compile) and tools/deploy-release.sh.

### Activity feed from push events
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
Server-side push logging in db.rs/git.rs/routes.rs (+172 lines). Both push routes covered, non-blocking error handling. Reviewed and approved.

### Rust server compilation + deployment
- **Completed**: 2026-03-20
Axum 0.8 fixes, deployed at git.repo.box:3490. PM2: repobox-git.

### EVM signature verification (ecrecover)
- **Completed**: 2026-03-20
65-byte recoverable signatures, real verify() via ecrecover, recover_address().

### Address-less push with auto-routing
- **Completed**: 2026-03-20
Push to `git.repo.box/myrepo.git`, server derives owner from signed root commit.

### Unsigned push rejection
- **Completed**: 2026-03-20
Server deletes bare repos post-push if no valid EVM signature found.

### Explorer UI (explore pages)
- **Completed**: 2026-03-20
Stats, repos list, repo detail with file tree, commits, README (rendered markdown), Config tab.

### git commit -S support (gpg.program)
- **Completed**: 2026-03-20
CLI acts as gpg.program. REPOBOX SIGNATURE armor format. init sets gpg.program + commit.gpgsign.

### Self-hosting (dogfooding)
- **Completed**: 2026-03-20
repo.box hosts itself at git.repo.box. Owner: 0xDbbA...2048.

### Permission config (.repobox/config.yml)
- **Completed**: 2026-03-20
3 groups (founders, agents, reviewers), default deny, 7 rules. Live on explorer Config tab.

### Sub-agent workflow with EVM identities
- **Completed**: 2026-03-20
Spawned claude-agent on feature/mobile-landing, signed with 0xAAc0...4a00, merged by founder.

### Mobile-responsive landing page
- **Completed**: 2026-03-20 | **Agent**: claude-agent (0xAAc0...4a00)
Conditional canvas rendering (CSS gradient on mobile), responsive typography with clamp(), media queries.

### Unified Next.js web app
- **Completed**: 2026-03-20
Consolidated landing, dashboard, blog, API, explorer, docs into one app at web/.

### Remote group resolvers
- **Completed**: 2026-03-20
HTTP + on-chain resolvers with caching. Server proxy for eth_call to Alchemy.

### 150 Rust tests passing
- **Completed**: 2026-03-20
135 core + 15 server (7 unit + 8 integration).
