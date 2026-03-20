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

### Full E2E demo script
- **Priority**: P0
- **Tags**: hackathon, demo
Script that runs the complete flow: `repobox init` → `keys generate` → signed commit → push → clone → verify on explorer. For the hackathon presentation.

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
