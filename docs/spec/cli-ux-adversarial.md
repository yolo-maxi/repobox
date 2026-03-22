# CLI UX Adversarial QA Log

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
