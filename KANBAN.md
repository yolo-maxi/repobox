# repo.box — Git Permission Layer for AI Agents

## 💡 Ideas

### ENS support for clone URLs
- **Priority**: P2
- **Tags**: feature, ux
Clone via `git.repo.box/vitalik.eth/myrepo.git` — resolve ENS to address server-side.

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

## 📋 Backlog

### Address component with ENS/subdomain resolution + human-readable URLs
- **Priority**: P1
- **Tags**: feature, explorer, ui, ens
Reusable `<AddressDisplay>` component. Resolution: ENS → repo.box subdomain → truncated hex. Hover: full address. Click: copy. All addresses clickable. Human-readable URL routing: `/explore/{ens-name}/` and `/explore/{subdomain}/` resolve and show repos. Add `/api/explorer/resolve/{name}` endpoint.

### ENS names in permissions
- **Priority**: P1
- **Tags**: feature, permissions, ens
Allow ENS names in `.repobox/config.yml` rules, e.g. `vitalik.eth push >main`. Resolution at evaluation time (not parse time) so ownership changes follow the name. Short TTL cache.

### Playground refresh — visual, accuracy, speed
- **Priority**: P1
- **Tags**: playground, ui, ai
Visual refresh, load system prompt from canonical source so it never drifts (currently missing `own`, `read`, `branch`), switch to faster Venice model, test both English→Config and Config→English modes.

### Virtuals integration — bug report to payment flow
- **Priority**: P1
- **Tags**: feature, x402, virtuals, docs
Ensure the full "virtuals" flow is implemented end-to-end: agent files bug report → creates branch → pushes fix → PR reviewed → merged → x402 payment triggered. Verify each step works with real EVM-signed commits and actual x402 micropayments. Then document it prominently: landing page case study, docs section, llms.txt, playground example. This is a key hackathon demo story — autonomous agent work with crypto-native payments. Make sure it's not just specced but actually functional and well-presented.

### Accurate llms.txt — critical for agent judges
- **Priority**: P1
- **Tags**: docs, agents
`repo.box/llms.txt` must be fully accurate and current — hackathon agent-judges will read this to understand what repo.box does. Review and update: correct verbs (including `branch`, `read`, `own`), config path (`.repobox/config.yml`), credential helper flow, force push policy, x402 payments, ENS resolution, on-chain resolvers, explorer features. Remove anything that doesn't work yet or mark it clearly. This is our pitch to the judges in machine-readable form.

### Empty state illustrations
- **Priority**: P2
- **Tags**: explorer, ui
Replace "No recent activity" etc with SVG illustrations + helpful text.

### Docs — comprehensive, honest, up-to-date
- **Priority**: P3
- **Tags**: docs
Cover all features (new verbs, x402, ENS, force push policy). Mark unimplemented features as "Coming Soon" / "Roadmap". Only run when no P0/P1/P2 tasks remain.

## 🔨 In Progress

### Fix repo detail page (explorer) — comprehensive overhaul
- **Priority**: P0
- **Tags**: explorer, ui, bugs
Major overhaul of `/explore/[address]/[name]` page:
1. **Sidebar layout**: Match explore home — left sidebar + constrained main column. No full-width.
2. **Broken tabs**: Fix Files, Commits, Config, Contributors tabs — data seems correct but UI is broken/bad.
3. **Remove SSH clone URL**: SSH not implemented. Only HTTPS with EVM-authed credential helper.
4. **Fix contributor count inconsistency**: Explore list shows different count than repo detail.
5. **Contribution chart**: Visual chart on Contributors tab.
6. **Fix language bar "Other" duplication**: Shows "Other" twice. Exclude binary/data/blob files.
7. **GitHub-style URL schema**: `/tree/{branch}/{path}`, `/blob/{branch}/{path}`, `/commits/{branch}`.

## 🚧 Blocked

### Gmail token expired — can't send emails
- **Blocked by**: Fran re-auth
- **Tags**: infra

## ✅ Done

### Force push handling
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Force push detection via pre-receive hook. Denied by default (`receive.denyNonFastForwards`). New `force-push` permission verb required to allow it. `--force-with-lease` also blocked without explicit permission. Comprehensive tests.

### Verb refactor: create → new files, branch → new branches
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Separated `create` (new files) from `branch` (new branches). Updated parser, engine, docs. All 184 tests pass. Breaking change with migration path.

### Dark/light theme toggle
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Theme toggle in explorer header, CSS variables, localStorage persistence.

### Activity feed from push events
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Server-side push logging + explorer API + UI. Auto-refresh every 30s.

### Explorer sidebar layout
- **Completed**: 2026-03-21 | **Agent**: Ocean
Sidebar with stats, sort, activity feed. Constrained main content column. GitHub-style flat repo rows with hover states.

### Contributor count pill on repo rows
- **Completed**: 2026-03-21 | **Agent**: Ocean
`getContributorCount()` API, compact emoji+number pills on explore page.

### Security: dashboard token moved to env var
- **Completed**: 2026-03-21 | **Agent**: Ocean
Removed hardcoded auth token from source (was exposed in public repo). Rotated token, moved to DASHBOARD_TOKEN env var.

### Production mode for web app
- **Completed**: 2026-03-21 | **Agent**: Ocean
Switched from `pnpm run dev` to `pnpm start`. Removed Vercel/Next.js dev toolbar.

### Dual-push to GitHub + git.repo.box
- **Completed**: 2026-03-21 | **Agent**: Ocean
Origin remote now pushes to both remotes. Explorer stays current with pipeline work.

### Clone URL copy widget
- **Completed**: 2026-03-21 | **Agent**: claude-agent
One-click copy, credential helper instructions, responsive design.

### Contributor identity cards
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Contributors tab with EVM address cards, push count, owner badges, responsive grid.

### Repo stats cards (languages, LOC, contributors)
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Language breakdown bar, lines of code, contributor count, repo age.

### Mobile-responsive explorer
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Responsive breakpoints at 375px/768px. Touch targets, horizontal scroll tabs, stacked layouts.

### Full E2E demo script
- **Completed**: 2026-03-21 | **Agent**: claude-agent
demo-e2e.sh (724 lines), quick/full modes, multi-agent simulation, demo-reset.sh, docs/DEMO.md.

### Search across all repos
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Server-side search API + debounced UI with grouped results.

### Branch selector in repo detail
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Dropdown with search, all APIs accept ?branch= param. Input sanitization (security fix).

### File viewer with syntax highlighting
- **Completed**: 2026-03-21 | **Agent**: claude-agent
25+ languages, line numbers, copy/download, binary/large file handling.

### README rendering polish
- **Completed**: 2026-03-21 | **Agent**: claude-agent
GitHub-style markdown, syntax highlighting, copy buttons, table styling, heading anchors.

### Commit detail page with diff viewer
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Unified diff, syntax highlighting, keyboard navigation, 20+ languages.

### Config opt-in enforcement
- **Completed**: 2026-03-21 | **Agent**: claude-agent
Repos without .repobox/config.yml skip permission checks. EVM signatures still required.

### Signer address per commit
- **Completed**: 2026-03-21 | **Agent**: claude-agent
ECDSA signature extraction, address recovery, owner vs collaborator badges.

### Install script + release pipeline
- **Completed**: 2026-03-21 | **Agent**: claude-agent
install.sh with platform detection, SHA256 checksums, version pinning. release.sh + deploy-release.sh.

### Rust server compilation + deployment
- **Completed**: 2026-03-20
Axum 0.8, deployed at git.repo.box:3490.

### EVM signature verification (ecrecover)
- **Completed**: 2026-03-20
65-byte recoverable signatures, real verify() via ecrecover.

### Address-less push with auto-routing
- **Completed**: 2026-03-20
Push to `git.repo.box/myrepo.git`, server derives owner from signed root commit.

### Unsigned push rejection
- **Completed**: 2026-03-20
Server deletes bare repos if no valid EVM signature.

### Explorer UI (explore pages)
- **Completed**: 2026-03-20
Stats, repos list, repo detail with file tree, commits, README, Config tab.

### git commit -S support (gpg.program)
- **Completed**: 2026-03-20
REPOBOX SIGNATURE armor format. init sets gpg.program + commit.gpgsign.

### Self-hosting (dogfooding)
- **Completed**: 2026-03-20
repo.box hosts itself at git.repo.box.

### Permission config (.repobox/config.yml)
- **Completed**: 2026-03-20
Groups, rules, default deny. Live on explorer Config tab.

### Sub-agent workflow with EVM identities
- **Completed**: 2026-03-20
claude-agent on feature branches, signed commits, founder merges.

### Mobile-responsive landing page
- **Completed**: 2026-03-20 | **Agent**: claude-agent
Responsive typography, CSS gradients, media queries.

### Unified Next.js web app
- **Completed**: 2026-03-20
Landing, dashboard, blog, API, explorer, docs in one app.

### Remote group resolvers
- **Completed**: 2026-03-20
HTTP + on-chain resolvers with caching. Alchemy proxy.

### 150+ Rust tests passing
- **Completed**: 2026-03-20
135 core + 15 server tests. Now 184+ with verb refactor and force push.
