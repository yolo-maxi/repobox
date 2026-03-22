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

*No tasks currently in backlog*

## 🔨 In Progress

*No tasks currently in progress*

## 🚧 Blocked

### Gmail token expired — can't send emails
- **Blocked by**: Fran re-auth
- **Tags**: infra

## ✅ Done

### Self-lockout + identity matrix adversarial matrix pass
- **Date:** 2026-03-22 | **Agent:** repobox-qa-pipeline
- Executed dedicated scenario covering founder/agent/unknown-signing states with real commit/push/pull/rebase flow:
  - founder initializes and seeds repo
  - agent blocked on `main`, allowed on `feature/**`
  - founder rebase/merge checks against remote `feature/qa`
  - explicit self-lockout check confirmed lockout BLOCK with recovery text
- `git pull` guidance on no-upstream branch remains clear and actionable.

### Self-lockout prevention UX guard for config edits
- **Completed**: 2026-03-22 | **Agent**: repobox-qa-pipeline
Added safety check in shim commit flow to block policy edits that remove the active identity's `.repobox/config.yml` edit permissions.
When attempted, CLI now returns an explicit BLOCK with recovery instructions:
`Keep at least one identity/group with edit rights on ./.repobox/config.yml`.
This was validated in an adversarial scenario with founder->risky config edits + lint output inspection.


### Identity rendering consistency fix (repo header)
- **Completed**: 2026-03-22 | **Agent**: qa-pipeline
Raw repo owner addresses are no longer displayed directly in repository header on explorer pages. Replaced raw 0x value with formatted `AddressDisplay` output for readability while preserving full-address copy behavior.

### Docs — comprehensive, honest, up-to-date
- **Completed**: 2026-03-22 | **Agent**: claude-agent + reviewer-agent
Comprehensive documentation update covering all features including new verbs, x402 payments, ENS resolution, force push policy. Added agent workflows and payment system guides. Marked unimplemented features as "Coming Soon" for clarity.

### Empty state illustrations
- **Completed**: 2026-03-22 | **Agent**: claude-agent + reviewer-agent
Comprehensive empty state illustrations for improved UX. Implemented 6 empty state scenarios with monochromatic SVG illustrations, reusable TypeScript components, and accessibility coverage. Replaced "No recent activity" etc across explorer, profiles, diffs, and search results. Approved for deployment-ready quality.

### Accurate llms.txt — critical for agent judges
- **Completed**: 2026-03-22 | **Agent**: claude-agent + reviewer-agent
Updated `repo.box/llms.txt` to be fully accurate and current for hackathon agent-judges. Reviewed and corrected: verbs (including `branch`, `read`, `own`), config path (`.repobox/config.yml`), credential helper flow, force push policy, x402 payments, ENS resolution, on-chain resolvers, explorer features. Pitch to judges is now machine-readable and current.

### Test ENS resolution end-to-end
- **Completed**: 2026-03-22 | **Agent**: claude-agent + reviewer-agent
Comprehensive ENS testing across all surfaces including real ENS names, repobox.eth subdomains, and repo.box aliases. 839 lines of unit tests plus integration script. Covers explorer routing, permissions, AddressDisplay, and Git URLs. Demo-ready quality assurance.

### ENS names in permissions
- **Completed**: 2026-03-21 | **Agent**: claude-agent + reviewer-agent
Allow ENS names in `.repobox/config.yml` rules, e.g. `vitalik.eth push >main`. Resolution at evaluation time (not parse time) so ownership changes follow the name. Short TTL cache. Comprehensive testing and documentation.

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

### Wrong remote + detached/no-upstream + explicit self-lockout adversarial pass
- **Date:** 2026-03-22 | **Agent:** repobox-qa-pipeline
- Ran one deep end-to-end scenario covering:
  - wrong remote push path
  - founder/agent identity-aware commit gating on main vs feature
  - fetch/rebase/merge lifecycle
  - detached HEAD and no-upstream `git pull` messaging
  - self-lockout edit protection on `.repobox/config.yml`
- Results:
  - Wrong remote produced native git transport error (clear path issue).
  - Agent main edits blocked; feature writes/push allowed per configured policy.
  - Detached HEAD + no-upstream guidance returned explicit, actionable git instructions.
  - Lockout guard returned explicit `BLOCK` + recovery hint.
- No code changes required in CLI logic for this run; no regressions found in tested path.
### x402 private-repo discoverability + read bypass (adversarial)
- **Date:** 2026-03-22
- **Agent:** repobox-qa-pipeline
- **Status:** ✅ Completed
- Story: private repo paid access / x402 preview discovery flow with founder-agent-unknown identity matrix.
- Findings:
  - Read checks previously used `target: None`, causing private rules to be effectively bypassed by branch context and poor unauthorized UX.
  - No-identity and non-grant identities returned inconsistent/misleading access errors.
- Fix:
  - `repobox-server/src/routes.rs`:
    - enforce read checks with `Some(">*")` for smart-http read path,
    - return `402 Payment Required + X-Payment` when `.repobox/x402.yml` exists and read is denied,
    - honor paid access records via `db::has_x402_access` before falling back to config deny.
- Validation:
  - `cargo test -p repobox-server x402`
  - manual clone/pull command checks:
    - unauth clone -> 402 payment required,
    - founder read -> 200,
    - unauthorized signed identities -> 402 with payment metadata.

### x402 paid-read bypass for repos with no read rules (adversarial)
- **Date:** 2026-03-22 | **Agent:** repobox-qa-pipeline
- **Status:** ✅ Completed (fixed in server)
- Scenario: private repo with `.repobox/x402.yml`, no explicit read rules, founder/agent/unknown identities.
- Fix: `repobox-server/src/routes.rs` now checks `db::has_x402_access` when read rules are absent but x402 exists, and allows granted identities before returning payment-required.
- Validation:
  - no-identity `info/refs` returns 402 + x-payment metadata
  - founder identity allowed
  - unsigned/unknown identity still 402
  - grant-access endpoint creates db entry and immediately bypasses read gate for granted identity
- Follow-up: remove duplicated wording in older KANBAN docs entries that described the pre-fix state as already solved if/where present.

### No-identity/no-upstream + malformed-config adversarial pass
- **Date:** 2026-03-22 | **Agent:** repobox-qa-pipeline
- Executed end-to-end fixture using real git shim:
  - founder bootstrap + identity setup
  - agent onboarding checks
  - unknown/no identity commit denial
  - branch-creation + feature branch push via configured rules
  - no-upstream pull diagnostics + rebase success after upstream set
  - malformed config lint probes (`default:`, invalid group YAML, bad rule syntax)
  - explicit self-lockout commit prevention
- UX outcome: no functional gaps found requiring code changes this run.
- Notes:
  - `git repobox check` branch-scoped file permissions require explicit `target >branch` in target argument to reflect branch context.


### Private repo x402 discovery + identity matrix run (private clone lifecycle)
- **Date:** 2026-03-22 | **Agent:** repobox-qa-pipeline
- Ran dedicated one-path scenario:
  - founder/agent/unknown identities,
  - signed clone with auth headers,
  - no-identity and unknown signature outcomes,
  - malformed auth signal,
  - x402 grant-access bypass flow,
  - founder follow-up commit + pull/rebase lifecycle.
- Results:
  - no-identity and non-granted agent clones consistently return HTTP 402 payment UX and blocker copy.
  - founder clone/pull and granted agent clone/pull work.
  - `git repobox check` for founder/agent branch/file permissions matches policy.
  - removing self config-edit rights still blocked by existing lockout text + recovery guidance.
- Known follow-up:
  - malformed `Authorization` in git clone still surfaces a generic git credential error in some clients despite clear server message (`auth token must be signature:timestamp`).
  - consider improving client-facing auth-failure UX in next run.
