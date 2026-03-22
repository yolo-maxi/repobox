# CLI UX Adversarial QA Log

## 2026-03-22 — Identity matrix + self-lockout verification run

### Scenario selected
`linter self-lockout attempts` plus founder/agent/unknown identity coverage, with pull/rebase/push checks.

### Environment
- Primary host check via SSH to `xiko@167.71.5.215` succeeded, but local fixture repo was prepared on this host because `/home/xiko/repobox` was unavailable on the remote host in this pass.
- Used local shim directly to avoid stale system binary:
  - `REPOBOX_BIN=/home/xiko/repobox/target/debug/repobox`

### Commands run
- Seeded fixture: `git init`, `repobox init`, initial commit before policy file exists, then wrote `.repobox/config.yml` with founder/agent groups and committed it on `main`.
- Created bare remote, set `origin`, pushed `main`.
- Cloned as `codex-agent` identity, exercised denied write on `main`, then created `feature/qa` and confirmed allowed docs edit + push on that branch.
- Founder-side lifecycle: `fetch`, `rebase origin/feature/qa`, attempted `merge` for hygiene (`Already up to date` after rebase).
- Pulled on agent side from tracking branch and then from detached/no-upstream branch to capture lifecycle copy.
- Self-lockout check: modified policy to remove founder config edit rights and attempted commit of modified `.repobox/config.yml`.
- Unknown/no identity-ish check: commit attempt with `HOME=/tmp/repobox-idless` (missing signer key) and unknown alias invocation.

### Key outputs
- Agent write denial on `main`:
  - `permission denied: codex-agent ... cannot append to README.md`
- Agent feature write/push allowed after branch-scoped edit path and push branch rule:
  - `feat: agent updates docs in feature scope` committed and pushed.
- No-upstream pull message:
  - `There is no tracking information for the current branch. Please specify which branch you want to merge with.`
- Self-lockout block (required in this run):
  - `permission denied: founder ... cannot commit this change because it removes your edit access to ./.repobox/config.yml.`
  - followed by recovery hint: `Keep at least one identity/group with edit rights on ./.repobox/config.yml...`
- Missing signer key path:
  - `signer key not found for evm:...` and remediation guidance.
- Unknown alias:
  - `error: unknown alias 'does-not-exist'.`

### Fix/fault status
- No product code change was required in this cycle; existing self-lockout block text is present and usable under ~30s.
- `repobox lint` and lifecycle checks passed for this fixture path.

### UX judgment
- Good: clear BLOCK message + recovery on self-lockout.
- Useful: clear no-upstream guidance for `git pull` on detached tracking state.
- Gritty edge: first-commit-before-policy path on an unborn branch still requires policy to be added after initial commit; acceptable, but worth documenting in onboarding notes.

## 2026-03-22 — Self-lockout prevention / identity-aware commit checks

### Scenario selected
Self-lockout prevention on config edits while exercising founder, agent, and missing-identity paths.

### Environment
- Primary host attempt: `xiko@167.71.5.215` (SSH reachable), but repo was not present there.
- Executed locally in `/tmp/repobox-qa-lockout5e` on this machine.
- Ran with real shim binary via `PATH=$HOME/.repobox/bin:$PATH`.

### Commands run
- `export REPOBOX_BIN=/home/xiko/repobox/target/debug/repobox`
- `git init` and `repobox init/setup` in founder repo.
- `repobox keys generate --alias founder`
- `git commit -m "repo init"`
- `git remote add origin` + `git push -u origin HEAD`
- `git clone origin agent`
- `git commit -m "no identity..."` before setting identity in cloned repo (with signer key removed)
- `repobox keys generate --alias agent`
- `git checkout -b feature/qa`
- `git commit -m "agent feature update"`
- `git push -u origin feature/qa`
- founder `git fetch`, `git checkout feature/qa`, `git rebase origin/feature/qa`, `git checkout master`, `git merge feature/qa`
- founder edited `.repobox/config.yml` to remove founder edit rule and ran:
  - `repobox lint`
  - `git commit -m "risky lockout"`
- validated missing-identity/ bad signer cases with:
  - `HOME=/tmp/repo-home-empty PATH="/home/xiko/.repobox/bin:$PATH" git commit -m ...`

### Observations before fix
- `git commit` of risky config with risky rules was previously only blocked by a generic permission error.
- No explicit recovery message about restoring at least one config editor identity.

### Fix implemented
- Added a safety check in `repobox-core/src/shim.rs` inside `check_commit` when `.repobox/config.yml` is staged.
- New helper checks:
  - whether the active identity can still `edit ./.repobox/config.yml` in proposed config,
  - whether they could edit it before the change,
  - and returns a `ShimAction::Block` when the change would remove their own config-edit right.
- New explicit message now says:
  - lockout is blocked,
  - identifies that the action removes the signer’s edit access,
  - and gives a recovery instruction (`Keep at least one identity/group with edit rights ...`).
- Added regression test:
  - `test_founder_cannot_lock_themselves_out_of_config_edit` in `repobox-core/src/shim.rs`.

### Test/verification results
- `~/clawd` repo build/tests:
  - `cargo test -p repobox-core` (pass)
  - `cargo test -p repobox-cli` (pass)
- Manual command outputs confirm:
  - no-identity path blocks with: `no identity configured`
  - wrong-signing-key path blocks with key-missing guidance
  - risky config edit path blocks with explicit lockout + recovery text

### UX outcome
- This scenario now blocks the footgun with actionable copy under ~30s:
  - explicit BLOCK,
  - root cause, and
  - next recovery step.

## 2026-03-22 — Wrong remote, detached HEAD, no-upstream, and lockout hardening run

### Scenario selected
`wrong remote, detached HEAD, no upstream branch` plus mandatory self-lockout verification.

### Environment
- Remote attempt via SSH to `xiko@167.71.5.215` succeeded only for connectivity check; repo working tree for this run was not present there.
- Ran locally in `/tmp/repobox-qa-20260322T210920` with the local shim binary.
- Multiple identities: `qa-founder`, `qa-agent` (generated via `git repobox keys generate`) and an identity-less context.

### Commands run (highlights)
- Founder bootstrap: `git init`, `git repobox keys generate --alias qa-founder`, `git repobox keys generate --alias qa-agent`, `git repobox use qa-founder`, branch+seed commit.
- Wrong remote test:
  - `git remote add origin /tmp/repobox-qa-does-not-exist/none.git`
  - `git push -u origin main`
- Correct remote test and baseline lifecycle:
  - set remote to local bare path, push `main`, clone as agent, agent branch permissions checks (`agent main edit` denied, `feature/qa` allowed).
- Rebase + merge lifecycle:
  - `git fetch`, `git checkout -b feature/qa origin/feature/qa`, `git rebase origin/feature/qa`
  - `git checkout main`, `git merge --ff-only feature/qa`
- Detached/no-upstream Git lifecycle:
  - `git checkout --detach origin/main && git pull` (agent)
  - `git checkout -b feature/no-upstream` + `git pull`
- Mandatory lockout test:
  - edited `.repobox/config.yml` to remove founder file-edit right and attempted commit.

### Observed outputs
- Wrong remote push failed with native git transport guidance:
  - `fatal: '/tmp/repobox-qa-does-not-exist/none.git' does not appear to be a git repository...`
- Main-branch hardening for agent works as intended:
  - `❌ permission denied: qa-agent ... cannot edit README.md`
- Feature branch self-serve permissions behave as configured:
  - `git checkout -b feature/qa` allowed for agent.
- Detached HEAD `pull` and no-upstream messaging are actionable:
  - `You are not currently on a branch...`
  - `There is no tracking information for the current branch...`
- Self-lockout guard remained strict and explicit:
  - `permission denied: qa-founder ... cannot commit this change because it removes your edit access to ./.repobox/config.yml.`
  - includes recovery text: `Keep at least one identity/group with edit rights on ./.repobox/config.yml ...`

### UX judgment
- Good:
  - Remote failure is clear and standard git error output.
  - Main/feature role split is understandable (main denied, feature allowed) and actionable.
  - Detached and no-upstream diagnostics are within the 30-second guidance target.
  - Lockout block text is explicit and offers immediate remediation.
- Minor cleanup:
  - `git repobox lint` currently does not accept a file path argument; it must be run without args.
    This is non-blocking UX noise but should be documented clearly in command examples.
