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

### Wire up activity feed from push events
- **Priority**: P1
- **Tags**: explorer, server
Server needs to log pushes to a table, API needs to return them. Explorer already has the UI.

### Add .repobox/config.yml to all studio projects
- **Priority**: P1
- **Tags**: dogfood
Push SSS, Oceangram, and other repos to git.repo.box with signed commits + configs.

### Install script improvements
- **Priority**: P1
- **Tags**: cli, distribution
`curl -sSf https://repo.box/install.sh | sh` needs to actually download a pre-built binary (currently just has the script skeleton).

### Explorer: show signer address per commit (not just owner)
- **Priority**: P1
- **Tags**: explorer
Each commit should show which EVM address signed it. Different agents = different addresses visible.

### Enforce .repobox-config opt-in on server
- **Priority**: P2
- **Tags**: server
Server should check if `.repobox/config.yml` exists in the pushed tree. Repos without config = no permission enforcement.

## 🔨 In Progress

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

## 🚧 Blocked

### Gmail token expired — can't send emails
- **Blocked by**: Fran re-auth
- **Tags**: infra

## ✅ Done

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
