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

## 2026-03-22 — Private repo x402 paid-access discovery run (DO fallback)

### Scenario selected
`private repo paid access / x402 preview/discovery flow` with founder/agent/unknown identities and full git lifecycle where possible.

### Environment
- Preferred remote host (`xiko@167.71.5.215`) was reachable, but repo exists locally only:
  - `/home/xiko/repobox` was not present there.
- Ran against local workspace source on this machine:
  - `repobox-server`: `target/debug/repobox-server`
  - `repobox-cli`: `target/release/repobox`
- Data dir: `/tmp/repobox-server-data`
- Repo under test: `http://127.0.0.1:3560/0xc6ff92A3Db57E04AEb3dC7F760e41f6949736266/fresh-repo.git`

### Commands run
- Prepared identities:
  - founder: `/tmp/repobox-home-test` (`founder` key)
  - agent: `/tmp/repobox-home-agent` (`agent` key)
  - unknown: `/tmp/repobox-home-unknown` (`outsider` key)
- Created and signed initial commit with bare repobox auth and pushed:
  - `git init`
  - set `commit.gpgsign` + `gpg.program` (repobox)
  - `git commit -m "feat: initial private repo"`
  - `git push -u origin master:refs/heads/main`
- Added private/x402 policy and pushed:
  - `.repobox/config.yml` (founder-only read/write/push on >*)
  - `.repobox/x402.yml` (USDC read price)
  - commit + push
- Identity lifecycle checks:
  - `repobox identity set <private-key>`
  - `repobox whoami`
  - `repobox alias add founder <founder_address>`
  - `repobox alias list`
  - `repobox check founder read ">main"`
  - `repobox check 0x... read ">main"` (agent/unknown denied)
- Git lifecycle:
  - `git push` founder updates
  - `git pull --rebase origin main` (from authenticated local repo)
- Clone UX checks:
  - unauthenticated clone:
    - `git clone http://127.0.0.1:3560/.../fresh-repo.git unauth-clone`
  - cloned with unauthorized signatures (agent/unknown): `git clone -c http.extraHeader="Authorization: Basic ..." ...`
- x402 grant/discovery checks:
  - `POST /{address}/{repo}.git/x402/grant-access` (agent, tx_hash placeholder)
  - repeat clone/read checks after grant

### Key outputs observed
- Before fix, authenticated reads were inconsistent because read checks used no branch target (`None`) for read rules.
- After changes:
  - No-identity read for private repo now consistently returns:
    - `HTTP/1.1 402 Payment Required`
    - `x-payment: {...}` metadata
  - Founder read check works when rules match (`HTTP/1.1 200 OK`)
  - Unlisted/unauthorized identities show the same paid discovery UX (`402 + x-payment`) rather than opaque auth failure.

### Fix implemented
- Updated `repobox-server/src/routes.rs`:
  - `check_read_access` now treats read checks as branch-scoped (`Some(">*")`) so configured read rules work with smart-http paths.
  - Added shared `payment_required_response()` for x402 errors.
  - No-auth + x402-private now returns 402 + payment hint instead of only generic auth error.
  - When x402 config exists and repo has paid users, paid access in DB (`db::has_x402_access`) short-circuits to allow read access.

### Test results
- `cargo test -p repobox-server x402`:
  - `test x402_grant_access_endpoint ... ok`
  - `test x402_payment_required_response ... ok`
- Manual CLI/server checks:
  - unauth clone: `fatal ... 402`
  - authorized founder read: `HTTP/1.1 200 OK`
  - unauthorized identity read: `HTTP/1.1 402 Payment Required`
  - `git pull --rebase` in private repo: `remote: payment required for read access`

### UX outcome
- Actionable under <30s: users now get explicit paid access metadata for non-auth/private repositories and discoverability is clearer.
- Remaining gap: x402 grant verification is still one-way path for DB grants; paid grant validation remains outside scope for this pass.

## 2026-03-22 — Private repo x402 paid-discovery + paid-access bypass run (post-fix)

### Scenario selected
`private repo paid access / x402 preview/discovery flow` with founder, agent, unknown identities.

### Environment
- Preferred DO host via SSH (`xiko@167.71.5.215`) was reachable, but `/home/xiko/repobox` is not present there.
- Ran locally against current source with real server shim:
  - `repobox-server`: `/home/xiko/repobox/target/debug/repobox-server`
  - `repobox` client: `/home/xiko/repobox/target/debug/repobox`
  - data dir: `/tmp/repobox-qa-data`

### Commands run
- bootstrapped fixture repo `/tmp/repobox-qa/fresh-private-repo2` as founder identity `0x760b...fBcf71`:
  - `git init`, `repobox init`, `repobox identity set`, `repobox alias add`, `repobox whoami`
  - staged `.repobox/config.yml`, `.repobox/x402.yml`, `README.md`
  - `git commit -m "feat: init private repo"`
  - `git push -u origin main`
- exercised self-lockout check by changing config to `founders push >*` only:
  - `repobox` commit returned explicit BLOCK + recovery guidance on `.repobox/config.yml` edit
- identity checks:
  - `repobox check <founder|agent> push >*`
- pull/rebase lifecycle:
  - authenticated founder token via `http.extraheader` pull with `git pull --rebase`
- private-access UX probes:
  - `curl` to `/info/refs?service=git-upload-pack` without auth
  - signed identity checks using EVM auth header
  - malformed token check (`Authorization: Basic badtoken`)
  - `POST /0x.../fresh-private-repo2.git/x402/grant-access` with agent address

### Findings before patch
- For repos with no explicit `read` rules (`default: deny` + x402 config), authenticated identities without paid grant were denied as `402` (good), but paid identities also remained blocked because read access path skipped paid-check bypass.
- Unknown/no-identity clone/pull flow was discoverability-poor on earlier attempts (opaque errors depending on token validity).

### Fix applied
- `repobox-server/src/routes.rs`:
  - In `check_read_access`, for `!has_read_rules` + x402 config, added x402 DB grant bypass before returning `Payment Required`.
  - On grant match, returns read access (`Ok(())`) with debug trace.
  - On DB read failure, returns `500` with explicit log message.

### Validation results
- no-auth read: `HTTP/1.1 402 Payment Required` + `x-payment` metadata
- malformed auth: `HTTP/1.1 401` + `auth error: invalid UTF-8 in Basic auth`
- founder identity: `HTTP/1.1 200 OK` for `/info/refs`
- agent identity before grant: `HTTP/1.1 402 Payment Required`
- after `x402/grant-access`: same agent identity returns `HTTP/1.1 200 OK`
- anonymous clone: `fatal ... returned error: 402`
- self-lockout guard remains explicit and actionable:
  - message includes: `cannot commit this change because it removes your edit access...`
  - includes direct recovery recommendation.

### UX verdict
- This run moved x402 paid access from inconsistent “always blocked” behavior to a clearer discoverability path:
  - private repos show payment intent by 402 + metadata,
  - granted identities can read without config edits,
  - denied identities are not silently blocked by auth-only errors.
