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

### Branch selector in repo detail
- **Priority**: P1
- **Tags**: explorer, ui
Dropdown to switch between branches in the repo detail page. Currently hardcoded to main. File tree, commits, and README should all update when switching branches. Show branch count badge.

### Search across all repos
- **Priority**: P1
- **Tags**: explorer, ui
Global search bar on explore home that searches repo names, owner addresses, and commit messages. Client-side filtering is already there for repos — extend to server-side full-text search with a dedicated API endpoint.

### Mobile-responsive explorer
- **Priority**: P1
- **Tags**: explorer, ui
Explorer pages need responsive layouts for mobile. File tree should collapse to a breadcrumb navigator, commit list should stack vertically, stats should wrap to 2x2 grid. Test on 375px and 768px viewports.

### Repo stats cards (lines of code, languages, contributors)
- **Priority**: P2
- **Tags**: explorer, ui
Show language breakdown bar (like GitHub), total lines of code, number of unique signers (contributors), and repo age. Use `git log` + file extension analysis server-side. Display as colored bar + stat cards on repo detail page.

### Contributor graph / identity cards
- **Priority**: P2
- **Tags**: explorer, ui
Show all unique EVM signers for a repo as identity cards: address (with alias if known), commit count, first/last commit date, and a mini contribution heatmap. Link each card to the address page.

### Dark/light theme toggle
- **Priority**: P2
- **Tags**: explorer, ui
Add theme toggle to explorer header. Persist preference in localStorage. Current explorer is dark — add a clean light theme option. Use CSS variables for all colors so theming is trivial.

### Clone URL copy widget + credential helper instructions
- **Priority**: P2
- **Tags**: explorer, ui
On repo detail page, prominent clone URL box with one-click copy (HTTPS + SSH variants). Below it, expandable section showing how to set up the repobox credential helper for authenticated clones. Include `curl | sh` install snippet.

### Empty state illustrations
- **Priority**: P2
- **Tags**: explorer, ui
Replace "No recent activity", "No repositories found", etc with illustrated empty states. Use simple SVG illustrations + helpful text ("Push your first repo to see it here"). Makes the explorer feel alive even when empty.

## 🔨 In Progress

### File viewer with syntax highlighting - SPEC COMPLETE
- **Priority**: P1
- **Tags**: explorer, ui  
- **Spec**: `docs/spec/file-viewer-syntax-highlighting.md`
- **Agent**: pm-agent (0x9aBA...234b)
When clicking a file in the tree, show it with proper syntax highlighting (Rust, JS, YAML, etc). Add line numbers, copy button, raw view toggle, and file size/line count in header. Currently shows plain text. Comprehensive implementation spec ready for development.

## 🚧 Blocked

### Gmail token expired — can't send emails
- **Blocked by**: Fran re-auth
- **Tags**: infra

## ✅ Done

### Full E2E demo script
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
Complete hackathon demo: `repobox init` → `keys generate` → signed commit → push → clone → verify. Scripts: demo-e2e.sh (724 lines, quick/full modes), demo-reset.sh, docs/DEMO.md. Multi-agent simulation, visual progress indicators, error handling with cleanup. See detailed spec in In Progress section.

### README rendering polish
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
GitHub-style markdown with syntax highlighting, copy buttons, table styling, heading anchors, image zoom, external link indicators.

### Commit detail page with diff viewer
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
Clickable commit hashes → detail page with unified diff, syntax highlighting, keyboard navigation, 20+ language support. +3069 lines.

### Config opt-in enforcement
- **Completed**: 2026-03-21 | **Agent**: claude-agent (0xAAc0...4a00)
Permission enforcement now opt-in only. Repos without .repobox/config.yml skip permission checks, just require EVM signatures.

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
