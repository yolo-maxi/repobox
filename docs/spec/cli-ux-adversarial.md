## 2026-03-23 — Grant-access accepts `evm:` prefixed payer addresses

### Scenario selected
`PRIVATE REPO FLOW on DO: private repo paid-access/preview + discoverability`

### Environment
- DO preference: `xiko@167.71.5.215` preferred first; fallback used (`/home/xiko/repobox` absent on DO path)
- Execution environment: `/tmp/repobox-qa-cycle-4` with local `repobox-server` on `127.0.0.1:37993`
- CLI/shim path: `/home/xiko/repobox/target/debug/repobox`
- Server path: `/tmp/repobox-qa-cycle-2/data`

### Identities exercised
- founder: `evm:0xE0D08f39d05C0B4e50501E69Ed75F72e3F8d44e6`
- agent: `evm:0x78eb9484933B33Be02C1753E3855D2D191650575`
- unknown/no identity (fresh clone without `~/.repobox/identity`)

### Commands run (exact flow)
- `git repobox keys generate --alias founder`
- `git repobox keys generate --alias agent`
- `git init`
- `git repobox init --force`
- write `.repobox/config.yml` + `.repobox/x402.yml`
- `git repobox lint`
- `git commit -m "chore: add private policy + x402"`
- `git remote add origin http://127.0.0.1:37993/<founder>/private-adversarial.git`
- `git push -u origin main`
- `git clone <origin>` with no identity
- `curl <origin>/.git/x402/info`
- `curl -X POST <origin>/.git/x402/grant-access`
- `cat /tmp/repobox-qa-cycle-4/scenario.log`
- `cargo test -p repobox-server x402_grant_access_endpoint -- --nocapture`
- `cargo test -p repobox-server x402_info_endpoint -- --nocapture`

### Exact notable output
- No-identity read UX stayed actionable:
  - `remote: payment required for read access. Call /.../x402/grant-access ...`
  - `fatal: ... returned error: 402`
- Initial grant probe with prefixed identity failed (expected from old behavior):
  - `HTTP/1.1 400 Bad Request ... invalid address format`
- After fix, prefixed probe now succeeds:
  - `HTTP/1.1 200 OK ... access granted`
- x402 metadata remained discoverable:
  - `{"currency":"USDC","for_sale":true,...,"read_price":"2.50",...}`
- Self-lockout guard remains explicit:
  - `cannot commit this change because it removes your edit access to ./.repobox/config.yml.`
  - recovery text printed in the same error.

### Fix applied
- `repobox-server/src/routes.rs`
  - normalize `grant_access` payer address by stripping optional `evm:` prefix
  - persist normalized address for validation + DB grant write
- `repobox-server/tests/smart_http.rs`
  - added acceptance assertion for `address: "evm:<0x...>"` in `x402_grant_access_endpoint`

### Validation / pass-fail
- `cargo test -p repobox-server x402_grant_access_endpoint -- --nocapture` ✅
- `cargo test -p repobox-server x402_info_endpoint -- --nocapture` ✅
- manual server smoke via curl against updated binary ✅

### UX judgement
- ✅ Partial P0 reduction: paid-access grant payload now accepts canonical `evm:` identity format, removing an immediate integration failure.
- ⚠️ Remaining high-priority work: paid read unlock + clone workflow still cannot be fully validated in local host without repo-box host-based helper path adjustments.

## 2026-03-23 — Legacy config self-lockout bypass in private x402 flow (P0)

### Scenario selected
`untouched/half-written config.yml (wrong rule syntax)` in the `private repo paid access/x402 preview/discovery` path.

### Environment
- Preferred host checked first: `xiko@167.71.5.215` → `DO_REPOBOX_MISSING` (no `/home/xiko/repobox`), so run executed locally.
- Local fixture: `/tmp/repobox-qa-teaser2-1774239087`
- Server: `repobox-server --bind 127.0.0.1:4131 --data-dir /tmp/repobox-qa-teaser2-1774239087/remote`
- CLI/shim: `/home/xiko/repobox/target/release/repobox` + `PATH=$HOME/.repobox/bin:$PATH`

### Identities exercised
- founder: `evm:0x289c665341eAf944b00037DFD1C55911494727DC`
- agent: `evm:0xCF6B3d2E6e74e140531C557dCb995464bA68899E`
- outsider: `evm:0x09eE10Ba902b48D86de552E45E54A5F4a77B9503`
- unknown/no identity: fresh HOME with no `~/.repobox/identity`

### Deep run summary (exact notable outputs)
- `git repobox lint` on legacy-shaped config (`rules:` + `default:` at top-level) incorrectly passed:
  - `✅ .repobox/config.yml is valid`
  - `2 groups, 0 rules, default: Allow`
- self-lockout prevention was bypassed under this config shape:
  - `[lockout] remove founder config edit right + commit`
  - commit succeeded: `[main 74c9e25] test: lockout attempt`
  - `[lockout-exit] 0`
- private paid gating also silently failed (repo effectively public):
  - no identity clone succeeded: `[noid-exit] 0`
  - outsider read succeeded: `[outsider-exit] 0`
- x402 teaser endpoint remained visible:
  - `{"currency":"USDC","for_sale":true,...,"read_price":"2.50",...}`
- grant endpoint UX mismatch in this run (payload shape confusion):
  - `missing field address at line 1 column 85`

### Root cause
Parser accepted unknown top-level keys and treated missing `permissions:` as default-allow + empty rules. Legacy config looked "valid" while disabling policy enforcement.

### Fix applied
- `repobox-core/src/parser.rs`
  - detect legacy top-level `rules` / `default` keys and return explicit blocking error:
    - `invalid rule: legacy config format detected: move top-level rules/default under permissions:`
- Added regression tests:
  - `test_legacy_top_level_rules_are_rejected`
  - `test_mixed_permissions_and_legacy_keys_are_rejected`

### Validation
- `cargo test -p repobox-core test_legacy_top_level_rules_are_rejected -- --nocapture` ✅
- `cargo test -p repobox-core test_mixed_permissions_and_legacy_keys_are_rejected -- --nocapture` ✅
- Manual re-test (fresh repo, legacy config):
  - `repobox lint` now fails with explicit migration guidance.
  - `git commit` now blocks with `.repobox/config.yml error: invalid rule: legacy config format detected ...`.

### UX judgement
- **P0 fixed in code:** legacy/wrong config syntax can no longer silently unlock private repos or bypass self-lockout protections.
- Recovery step is now explicit in <30s: move keys under `permissions:`.

## 2026-03-23 — DO production private paid flow + unborn-HEAD fix verification

### Scenario selected
`private repo paid access/x402 preview/discovery flow` (deep run on DO), including founder/agent/unknown/no-identity matrix and lockout guard.

### Environment
- Preferred host used: **DigitalOcean** `xiko@167.71.5.215`
- Real CLI/shim binary path: `~/rbqa-bin/repobox`
- Real remote: `https://git.repo.box/<founder-address>/private-paid-<ts>.git`
- Run root: `/tmp/repobox-do-full-1774237852`

### Commands run (key lifecycle)
- identity lifecycle:
  - `repobox setup`
  - `repobox keys generate --alias founder|agent|outsider`
  - `repobox identity set <private-key>`
  - `repobox whoami`
  - `repobox alias add ...`
- repo lifecycle:
  - `git init && git branch -m main`
  - `repobox init --force`
  - seed `.repobox/config.yml` + `.repobox/x402.yml`
  - signed `git commit` and `git push -u origin main`
  - `repobox check <identity> push/read ...`
- lockout guard:
  - removed `founders edit * >*` and attempted commit
- private/x402 checks:
  - no-identity clone
  - unknown signed `git ls-remote` with generated Basic auth token
  - `curl https://git.repo.box/<addr>/<repo>/x402/info`
  - `POST .../x402/grant-access`
  - agent signed `git ls-remote` after grant

### Exact notable outputs
- First commit now succeeds on unborn HEAD with branch-scoped upload rules:
  - `[founder-first-commit-exit] 0`
  - `create mode 100644 .repobox/config.yml`
- Lockout prevention remains explicit:
  - `cannot commit this change because it removes your edit access to ./.repobox/config.yml`
- No identity clone UX (still weak):
  - `fatal: could not read Username for 'https://git.repo.box': terminal prompts disabled`
- Unknown signed read (private repo):
  - `remote: read access denied`
  - `... returned error: 403`
- Later after adding founder read rule and retrying signed founder clone:
  - `remote: payment required for read access`
  - `... returned error: 402`
- x402 endpoints on this production flow were not discoverable from these paths:
  - `curl .../x402/info` returned empty body
  - `POST .../x402/grant-access` returned empty body in one run and no observable read unlock in another

### UX judgement
- ✅ Self-lockout block/guidance quality is good.
- ✅ First-commit branch-scoped commit path is fixed in CLI (no founder upload deadlock).
- ❌ P0 (production UX): private paid discoverability/read flow is inconsistent on `git.repo.box` (mix of 403/402 and empty x402 endpoint responses), and no-identity clone path falls back to generic username prompt instead of actionable purchase guidance.

### Fix applied in this run
- Code patch in `repobox-cli/src/main.rs`:
  - robust unborn HEAD branch detection (`detect_current_branch`) now handles `rev-parse` non-zero + symbolic-ref fallback.
  - used in shim and status paths.
- Added regression test:
  - `test_detect_current_branch_on_unborn_head`

### Validation
- `cargo test -p repobox-cli test_detect_current_branch_on_unborn_head -- --nocapture` ✅
- `cargo test -p repobox-cli -- --nocapture` ✅
- Manual repro before/after:
  - before fix: branch showed `unknown` and first commit denied `cannot upload .repobox/config.yml`
  - after fix: branch resolves `main`; first commit allowed with branch-scoped file rules.

## 2026-03-23 — Private x402 founder/agent matrix + pull/rebase (deep clean run)

### Scenario selected
`private repo paid-access / founder + agent + no identity / pull --rebase` run with deterministic local fixture.

### Environment
- DO host path was preferred, but remote checkout path unavailable; ran locally.
- Server: `/home/xiko/repobox/target/debug/repobox-server` on `127.0.0.1:3999`
- Fixture: `/tmp/repobox-qa-private-run-deep`
- Repo namespace: `evm:0x77b68E39A345688D6E015307897796fdc11f7351/qa-private-1144`

### Commands run
- bootstrap identities and lifecycle setup
  - `HOME=$founder_home repobox keys generate --alias founder`
  - `HOME=$agent_home repobox keys generate --alias agent`
  - `git init`
  - `repobox init --force`
  - write `.repobox/config.yml` + `.repobox/x402.yml`
  - `repobox whoami`
  - `repobox alias add founder ...`
  - `repobox lint`
  - `repobox check <founder> push ">*"`
  - `repobox check <founder> read ">*"`
- seed + remote lifecycle
  - signed `git commit -m "seed paid private repo config"`
  - `git push -u origin HEAD:refs/heads/main`
  - `repobox lint` + denied `repobox check <founder> edit ".repobox/config.yml"` on lockout config
- anonymous failure path
  - `git clone <private origin>` (no identity)
  - `git clone -c http.extraheader="Authorization: Basic !!bad!!"` (malformed)
  - both expected 402 with payment guidance
- discoverability
  - `curl <origin>.git/x402/info`
  - `curl -X POST <origin>.git/x402/grant-access`
- authorized agent read/workflow
  - immediate `repobox` signature generation via `gpg` message
  - `git ls-remote <origin>` succeeded with valid Basic auth header
  - `repobox init --force` in agent clone
  - `agent-note` commit + `git push origin HEAD:refs/heads/feature/qa`
  - founder follow-up commit + push
  - `git pull --rebase origin main` from agent clone with fresh signed auth header

### Key observed output
- No-identity clone: `remote: payment required for read access. Call /x402/grant-access ... and visit /x402/info for pricing metadata`
- Bad-auth clone: same `402` guidance (no username prompt)
- `x402/info` response: readable discoverability JSON with `for_sale:true`, `read_price:"3.00"`, `recipient:"evm:0x77..."`, `scheme:"exact"`
- Grant endpoint: `access granted`
- `ls-remote` authorized: returned branch hashes immediately with valid auth
- `git push` feature branch: created `feature/qa`
- `git pull --rebase origin main`: `Successfully rebased and updated refs/heads/main`
- Lockout guard behavior:
  - lint warning for founder branch-only rules
  - `denied — founder ... edit .repobox/config.yml (default: deny)`
  - recovered on restoring file permissions

### Findings / UX result
- `founder`/`agent` lifecycle + no-identity matrix is coherent when HOME-aware signing is preserved in commit/pull commands.
- Paid repo discoverability and grant UX remains explicit and machine-readable.
- No behavior regression requiring code change in this run.
## 2026-03-23 — No-identity lifecycle + branch-policy matrix

### Scenario selected
`no identity configured + commit/push/pull` deep path with founder/agent/no-identity identities.

### Environment
- DO SSH preference checked: `xiko@167.71.5.215` reachable, but repo source not available there; ran locally.
- Server: local `repobox-server` on `127.0.0.1:3821` with fixture under `/tmp/repobox-qa-idmatrix`.
- CLI/shim path: `/home/xiko/repobox/target/debug/repobox` and temp `PATH=$HOME/.repobox/bin:$PATH` in each identity sandbox.

### Commands run
- Founder bootstrap:
  - `git init`, `git repobox init`, `git checkout -b main`
  - `git repobox keys generate --alias founder`
  - `git repobox alias add founder <founder_addr>`
  - `git repobox whoami`
  - `git repobox check <founder> own ">*"`
- Agent bootstrap:
  - `git repobox keys generate --alias agent`
  - `git repobox alias add agent <agent_addr>`
  - `git repobox whoami`
- Policy setup in `.repobox/config.yml`:
  - `default: deny`
  - wildcard read: `* read >*`
  - founder controls: `founders own >*`, `founders edit ./*`
  - agent branch scope: `agents push >feature/**`, `agents branch >feature/**`, `agents edit ./* >feature/**`
- Lifecycle operations:
  - `git repobox check <agent> push ">feature/qa"`
  - `git repobox check <agent> push ">main"` (expected deny)
  - `git remote add origin ...`, `git push -u origin main`
  - no-identity clone + `git repobox whoami` + commit attempt + `git pull --rebase`
  - agent clone + feature branch commit + push
  - detached clone + no-upstream branch + `git pull --rebase`
  - founder follow-up commit + push + agent rebase
- Static self-lockout smoke:
  - rewrote config to `founders push >*` only + `default: deny`
  - `git repobox lint` warning observed
  - commit attempt blocked on founder edit
  - add `founders edit ./*` recovery and retry committed.

### Observed outputs
- No-identity commit blocked with clear guidance:
  - `❌ no identity configured. Run: git repobox identity set <private-key>`
- Founder/agent rule checks:
  - founder owns rule: ✅
  - agent push to feature: ✅ allowed
  - agent push to main: `implicit deny` (expected)
- No-upstream branch pull UX:
  - `There is no tracking information for the current branch.`
- Clone lifecycle:
  - `no identity configured...` warning emitted on read/clone path
  - feature branch push by agent succeeds when permissions allow both branch + file verbs.
- Self-lockout explicitness:
  - `cannot append to README.md` when founder edits rights removed.
  - recovery guidance available after adding explicit edit rule.

### UX notes
- `git repobox check` deprecation copy (`create`/`write`) is still emitted for rules expanded from `own`, but it remains a warning only.
- No regressions requiring code patch in this story run.

### Validation results
- ✅ one deep scenario completed including lifecycle: init, whoami, alias add, check, commit, push, pull/rebase.
- ✅ lockout guard remains explicit with remediation path.
- ✅ no-identity failure message is understandable and short.



## 2026-03-23 — Private repo x402 paid-access flow (founder/agent/no-identity + self-lockout)

### Scenario selected
`private repo paid access / x402 preview discovery / self-lockout prevention`.

### Environment
- Preferred DO host was reachable, but `/home/xiko/repobox` was missing there; run executed locally with a full fixture in `/tmp/repobox-qa-private`.
- Server: local `repobox-server` on `127.0.0.1:3797` using data dir `/tmp/repobox-qa-remote`.
- Repo: `0xa02B18A9eec01d5a6d331033236E29df97A4f80B/private-qa-001`.
- CLI binary: `/home/xiko/repobox/target/debug/repobox` and shim via `PATH=/tmp/repobox-qa-private/founder-home/.repobox/bin:$PATH`.

### Commands run
- Founder fixture seed:
  - `git init`, `git repobox init`
  - `REPOBOX_HOME=/tmp/repobox-qa-private/founder-home git repobox keys generate --alias founder`
  - `git repobox alias add founder evm:0xa02...`
  - write `.repobox/config.yml` with founder/agent groups, `default: deny`, and `.repobox/x402.yml`
  - signed commit + `git push -u origin main`
- Identity checks:
  - `git repobox whoami`
  - `git repobox check founder read '>main'`
  - `git repobox check founder edit '>main ./.repobox/config.yml'`
  - `git repobox check evm:0x8aC6... push '>feature/test'`
  - `git repobox whoami` with no identity directory
- Lifecycle checks:
  - founder commit + push on `main`
  - `git clone` as founder with signed header, then `git pull --rebase` in authorized clone after upstream update
- Private-read UX checks:
  - no-identity clone (expected `402`)
  - malformed header clone (`http.extraheader=Authorization: Basic !!bad!!`) (expected `402`)
  - `curl .../x402/info`
  - agent identity header clone before grant (`402` expected)
  - POST `.../x402/grant-access`
  - agent identity header clone after grant (`success`)
- Linter/self-lockout check:
  - switch to branch with PATH shim and attempt to remove founder edit rights in `.repobox/config.yml`
  - commit blocked with explicit blocker + recovery hint.

### Findings
- Anonymous/no-identity clone is still blocked with actionable next steps:
  - `payment required for read access...`
  - explicit `.../x402/grant-access` and `.../x402/info` guidance.
- Signed/authorized clone path remains blocked for unpaid agent identities until DB grant exists.
- `x402/info` endpoint is publicly reachable and returns discoverability metadata (`for_sale`, `read_price`, recipient, memo, scheme).
- Once grant exists, agent identity clone works.
- Pull/rebase lifecycle works when a clone has `http..../.extraheader` persisted.
- Self-lockout protection is active and explicit:
  - `cannot commit this change because it removes your edit access to ./.repobox/config.yml`
  - includes recovery text to re-add an edit rule.

### Fix status
- No code change required in this run; behavior already matches UX expectations for this story.
- Logged as adversarial evidence for the matrix item:
  `private repo paid access/x402 preview/discovery flow`.

### Validation
- Manual CLI + git exercises above.
- Observed statuses: 402 guidance, lockout BLOCK, and successful post-grant authorized clone.


## 2026-03-23 — Private repo x402 clone UX: malformed auth hardening

### Scenario selected
`private repo paid access / malformed auth / discoverability surface`.

### Environment
- Primary host preference was remote DO (`xiko@167.71.5.215`) for serverless parity, but fixture repos and local logs were executed in `/tmp/repobox-qa-minimal2` on the current host because the required helper bootstrap is faster locally.
- Server: local `repobox-server` on `127.0.0.1:3797`
- Repo: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266/minimal.git`
- CLI binary: `/home/xiko/repobox/target/debug/repobox`

### Commands run
- `git init` + `repobox init`
- `repobox keys import` + `identity set` (founder)
- `git repobox alias add founder`
- `git repobox whoami`
- `repobox check founder push ">*"`
- wrote `.repobox/config.yml` with `default: deny`, `founders` group, and x402 config
- signed commit and `git push -u origin main`
- **Unknown/no-identity clone attempt**:
  - `git clone <private x402 origin>` (expect 402)
- **Malformed auth clone attempt**:
  - `GIT_TERMINAL_PROMPT=0 git -c "http.extraheader=Authorization: Basic !!bad!!" clone <origin>`
- **Paid/unpaid metadata discovery**:
  - `curl <origin>/x402/info`
- local lifecycle check for pull/rebase:
  - `git clone . <local clone>`
  - commit update in source and `git pull --rebase` in local clone

### Findings
- Before this patch, malformed auth frequently surfaced as `could not read Username` in git, which is unclear for tokenized workflows.
- With the fix, malformed auth now returns the same payment-required UX as missing auth for x402 repos:
  - `remote: payment required for read access. Call .../x402/grant-access ...`
  - includes `/x402/info` pricing metadata guidance
- No-identity clone also returns the same explicit 402 message and metadata hints.
- Pull/rebase lifecycle in local clone path remains functional after seeded commits.

### Fix applied
- `repobox-server/src/routes.rs`:
  - on auth parse failures during read access checks, branch now returns `payment_required_response` (not a bare auth error) when an x402 config exists.
  - adds clearer warning log and keeps invalid header paths on the paid-discovery track instead of username-prompt ambiguity.
- `repobox-server/tests/smart_http.rs`:
  - extended `x402_payment_required_response` to assert malformed `Authorization` header also returns payment-required guidance (and no username prompt text).
- `repobox-cli/src/main.rs`:
  - made credential helper URL parser tolerant of `http(s)://git.repo.box` with optional port and optional path prefixes, so helper can resolve repo path in local/port-forwarded test environments.

### Validation
- `cargo test -p repobox-cli` (build)
- `cargo test -p repobox-server x402_payment_required_response -- --nocapture`
- Manual run in `/tmp/repobox-qa-minimal2` and `/tmp/repobox-qa-minimal`

### UX judgement
- `good` after fix:
  - malformed auth now degrades to clear payment-required guidance in <30s,
  - x402 repos remain discoverable via `/x402/info` before login.
- No remaining P0 observed for this scenario.

## 2026-03-22 — Private repo x402 preview + malformed auth UX pass

### Scenario selected
`x402 private repo paid-discovery flow`, including founder/agent/unknown identities and malformed auth token behavior.

### Environment
- Server: target/debug/repobox-server on 127.0.0.1:3566
- Repo: 0xE2Bc1b8081a158E0F9Bd9cae90C1f0C1A1031955/private-x402-qa.git
- Data dir: /tmp/repobox-qa-data

### Key findings
- `git clone` without identity still returns `402` for private x402 repos, but now includes actionable body text:
  - `/{address}/{repo}.git/x402/grant-access`
  - `/{address}/{repo}.git/x402/info`
- Added public metadata preview endpoint so private repos expose pricing info before authentication:
  - network, price, recipient, memo, scheme, `for_sale: true`
- Manual unauthorized clone output now contains next-step URLs inline in body and continues to return `X-Payment`.
- `git pull --rebase` lifecycle is still tested in fixture path; rebase succeeds after resolving one README conflict.

### Fix applied
- `repobox-server/src/routes.rs`:
  - added `GET /{address}/{repo}.git/x402/info`
  - extended 402 message body with direct preview and grant endpoints
- `repobox-server/tests/smart_http.rs`:
  - added `x402_info_endpoint` test asserting unauth metadata discoverability

### Validation
- `cargo test -p repobox-server x402_ -- --nocapture`
  - passed: `x402_info_endpoint`, `x402_grant_access_endpoint`, `x402_payment_required_response`
- Manual checks:
  - `curl http://127.0.0.1:3566/{namespace}/{repo}.git/x402/info`
  - `curl -i http://127.0.0.1:3566/{namespace}/{repo}.git/info/refs?service=git-upload-pack`

### Fix status
- Commit created in this run and pushed.
- Follow-up remaining: add end-to-end paid-access clone success test once deterministic signature fixture is standardized.

## 2026-03-23 — First-time install + founder-first push (founder/agent/no-identity matrix)

### Scenario selected
`first-time install + first successful push` (with founder, agent, no-identity flows to exercise lifecycle commands)

### Environment
- Preferred DO host was reachable but lacked `/home/xiko/repobox`, so fallback local workspace used.
- Local server used: `target/debug/repobox-server` on `127.0.0.1:3560` with `/tmp/repobox-server` data.
- CLI shim used: `/home/xiko/repobox/target/debug/repobox`.
- Fixture workspace: `/tmp/repobox-qa4`.

### Commands run
- `HOME=/tmp/repobox-qa4/founder-home`
- `repobox setup`
- `git init`
- `git repobox status` (baseline, before identity)
- `git repobox keys generate --alias founder`
- `git repobox alias add founder <evm:...>`
- `git repobox check <founder> own ">main"`
- `git checkout -B main`
- wrote `.repobox/config.yml` with explicit valid founder-only allow policy and committed it with `README.md`
- `git remote add origin http://127.0.0.1:3560/$FOUNDER_ID/founder-repo.git`
- `git push -u origin main`
- **No-identity clone flow:**
  - setup identity-less shim
  - `git clone <origin>`
  - `git repobox status`
  - `git pull --rebase origin main`
- **Agent flow:**
  - setup fresh agent home
  - `git clone <origin>`
  - `git repobox keys generate --alias agent`
  - `git repobox check <agent> own ">main"`
  - `git pull --rebase origin main`
  - local change + `git commit` + `git push`

### Findings
- Fresh init from clean home now writes parseable `.repobox/config.yml`:
  - no longer hits `groups must be a list, mapping, or resolver` parse error.
- Founder path completed a clean first push to a new remote branch (`main`).
- No-identity clone path works end-to-end for public read:
  - clone + status + pull/rebase are usable without a local identity.
- Agent path behaves as configured:
  - `check` shows explicit implicit-deny for `push >main`.
  - `git push` returns explicit `cannot push to main` with actionable rule wording.
- One non-blocking UX issue remains: no-identity check using bare alias name (`git repobox check founder read ">main"`) fails with `invalid identity: founder` because aliases are repo-local and not populated in the clone.

### Fix applied
- `repobox-cli/src/main.rs`
  - Updated `CONFIG_TEMPLATE` (init scaffold) to default valid empty groups:
  - `founders: []`
  - `agents: []`
  - keeps placeholder guidance in comments only.

### Validation
- `cargo test -p repobox-cli -- --nocapture` passed (all tests green).
- Scenario outputs include founder commit/push success and agent denial + pull/rebase execution in cloned repos.

### UX judgment
- `good` for first-push onboarding: config is parseable and founder path can complete in <30s.
- `improvement` for alias UX in check: `invalid identity: founder` is technically correct but likely expected in no-alias/no-identity clones.

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

## 2026-03-22 — No-identity + no-upstream lifecycle and malformed-config edge-case run

### Story selected
`no identity configured + commit/push/pull` (matrix rotate), with linter self-lockout and malformed config checks in the same run.

### Environment
- Primary DO host (`xiko@167.71.5.215`) had no `/home/xiko/repobox` tree for this run.
- Executed locally with shim at `~/repobox/target/debug/repobox` via:
  - `PATH=/home/xiko/.repobox/bin:$PATH`
- Working dir: `/tmp/repobox-qa-noid-20260322-run`
- Real git shim path used for all repo operations.

### Fixture setup
- founder identity: `0xa66Dd332Da3A5b4E65D510E764Dd51d056c3f696`
- agent identity: `0xec641852DEF97f9d8BE8d89828a65037F82E7d77`
- Bare remote: `/tmp/repobox-qa-noid-20260322-run/remote/qa-noid.git`

### Commands run (highlights)
- Bootstrap:
  - `git init`
  - `git repobox init`
  - `git repobox keys generate --alias founder`
  - `git repobox use founder`
  - `git repobox alias add founder-alias ...`
  - seed commit on public `main` (default allow config)
  - tightened rules and pushed:
    - founder `own`/`push` baseline
    - agent `branch/edit/push` on `>feature/**` only
    - explicit deny on `agents not edit ./.repobox/config.yml`
- Agent onboarding:
  - clone from bare remote
  - `git repobox whoami` (before identity)
  - identity generation/selection and `git repobox alias list`
  - `git repobox check agent push main` (denied)
  - `git repobox check agent edit README.md >main` (denied)
- Lifecycle:
  - agent push on main denied (`permission denied`)
  - agent branch creation denied until branch rule added
  - founder updated config with `agents branch >feature/**`
  - agent branch created `feature/qa`, committed, and pushed
  - founder-side `git fetch` + fast-forward checks
  - agent branch `feature/no-upstream` pull with no upstream mapping
    (`There is no tracking information for the current branch...`)
  - `git pull --rebase` after setting upstream to `origin/feature/qa` succeeded
- Unknown/no-identity path:
  - separate clone with `HOME` pointing to clean key store
  - `git repobox whoami` failed with explicit no-identity guidance
  - `git commit` failed with same message before identity selection
- Self-lockout guard:
  - founder attempted config change removing founder edit access
  - commit blocked with explicit recovery sentence in CLI output.
- Malformed/partial config checks:
  - `default:` empty
  - invalid group entry under list
  - truncated flat rule (`- agents push`)
  - all surfaced via `git repobox lint`.

### Pass/fail notes
- Self-lockout guard: ✅ explicit block + immediate recovery guidance.
- No-identity workflow: ✅ clear first-step recovery (`Run: git repobox identity set <private-key>`).
- malformed config UX: ✅ errors are specific and include actionable hints to fix `default`, YAML structure, and rule syntax.
- No-upstream UX: ✅ actionable standard git guidance.
- Lint quality issue observed:
  - `git repobox check` for file permission checks requires branch in target (`file >branch`) to evaluate branch-scoped rules; without it users see implicit deny.
  - This is known behavior; not changed in this run.

### Fixes
- No code changes for this run.
- Outcome recorded as “behavioral clarity pass” for this scenario.

## 2026-03-22 — Private repo paid-access / x402 flow (identity matrix + clone lifecycle)

### Scenario selected
`private repo paid access/x402 preview/discovery flow` with founder/agent/unknown identities, including init/push/pull/rebase and explicit grant flow.

### Environment
- Preferred DO host `xiko@167.71.5.215` was reachable for SSH checks, but `/home/xiko/repobox` was not available there.
- Ran locally with real binaries:
  - `repobox-server`: `/home/xiko/repobox/target/debug/repobox-server`
  - `repobox` CLI: `/home/xiko/repobox/target/debug/repobox`
  - server data: `/tmp/repobox-qa-data-fresh`
- Real git shim was used via `PATH=$HOME/.repobox/bin:$PATH` in fixture home `/tmp/repobox-qa-run-home`.

### Commands run (high-level)
- Founder bootstrap with dedicated identities:
  - `git init`
  - `git repobox init`
  - `git repobox keys generate --alias founder`
  - `git repobox keys generate --alias agent`
  - `git repobox identity set <founder-key>`
  - `git repobox alias add founder <founder-eip191>`
  - `git repobox alias add agent <agent-eip191>`
  - signed commit + `git push -u origin HEAD:refs/heads/main`
- Configured policy + x402:
  - `.repobox/config.yml`: founder-only `own`, `branch`, `upload`, `edit`
  - `.repobox/x402.yml`: `read_price: 3.25`, configured recipient/network
  - committed and pushed
- Identity checks:
  - `git repobox check founder branch *` ✅
  - `git repobox check agent branch *` ❌
  - `git repobox check founder upload ./.repobox/config.yml` ✅
  - `git repobox check agent upload ./.repobox/config.yml` ❌
- Read-access UX checks:
  - clone with no `Authorization` header (expected 402)
  - clone as founder signature (expected success)
  - clone as agent signature before grant (expected 402)
  - clone as malformed signature (expected auth failure)
  - `POST /{repo}.git/x402/grant-access`
  - clone again as granted agent (expected success)
- Git lifecycle checks:
  - founder follow-up commit + `git push`
  - `git pull --rebase origin main` in authenticated founder clone
- Unknown/no-identity check:
  - `HOME=/tmp/repobox-qa-empty git repobox whoami`
- Extra self-lockout sanity:
  - edited config to remove founder edit rights and re-attempted commit

### Key outputs
- branch checks:
  - founder allowed on `branch *`
  - agent denied on `branch *` (`implicit deny: rules exist for 'branch', no match for this identity`)
- private read/clone behavior:
  - no auth clone: `remote: payment required for read access` + `fatal ... returned error: 402`
  - founder authorized clone: success
  - agent before grant: `remote: payment required for read access` + `fatal ... returned error: 402`
  - malformed auth clone: `could not read Username for 'http://127.0.0.1:3560': No such device or address`
  - grant response: `access granted`
  - agent after grant: success clone
- pull/rebase lifecycle:
  - founder clone updated with `git pull --rebase origin main` (fast-forward from `6f8f670` to `95448e9`)
- self-lockout sanity:
  - commit removing founder edit right blocked with explicit blocker text and recovery hint:
    `cannot commit this change because it removes your edit access to ./.repobox/config.yml`
- missing-identity path:
  - `no identity configured. Run: git repobox identity set <private-key>`

### Fix/fault status
- No server/client code changes required in this run.
- Outcome is mostly stable; one UX gap remains: malformed `Authorization` token from Git may surface as a generic git username lookup error, not the server `auth token must be signature:timestamp` payload.

### UX judgment
- ✅ `402 Payment Required` path is clear for both anonymous and unauthorized identities and makes private repos discoverable as paid-gated.
- ✅ founder/agent role split plus authenticated clone flow works end-to-end (push + rebase lifecycle).
- ⚠️ malformed auth tokens via Git produce a noisy client-side error (`could not read Username...`) even though server returns clear 401 text; not yet remediated.

## 2026-03-23 — x402 private repo paid-access/discovery cycle (founder-agent-unknown matrix)

### Scenario selected
`private repo paid access/x402 preview/discovery flow` one deep run, with founder, agent, and unknown/no-identity identities.

### Environment
- Preference check: DO SSH to `xiko@167.71.5.215` was reachable, but `/home/xiko/repobox` is not present there, so ran locally with real binaries.
- Server (ephemeral): `127.0.0.1:34789` with data under `/tmp/repobox-qa/private-adv-run/server/data`.
- Git shim path used:
  - `PATH=/tmp/repobox-qa/private-adv-run/home/.repobox/bin:$PATH`
  - shim command: `/tmp/repobox-qa/private-adv-run/home/.repobox/bin/git`

### Commands run
- `git repobox setup`
- `git repobox keys import <founder_key> --alias founder`
- `git repobox keys import <agent_key> --alias agent`
- `git repobox use founder`
- `git repobox alias add founder <founder_address>`
- `git repobox alias add agent <agent_address>`
- `git init`
- `git repobox init`
- create `.repobox/config.yml` + `.repobox/x402.yml`
- `git add ... && git commit -m "feat: initialize private repo"`
- `git remote add origin http://127.0.0.1:34789/$OWNER/$REPO.git`
- `git push -u origin main`
- `git repobox check founder push ">*"`
- `git repobox check founder edit ".repobox/config.yml"`
- `git repobox check founder read ">*"`
- `git repobox check agent read ">*"`
- self-lockout attempt (`founders` edit removal in config) and commit
- clone attempts:
  - unauth `git clone` (`GIT_TERMINAL_PROMPT=0`)
  - malformed token `http.extraheader=Authorization: Basic !!bad!!`
  - signed unauthorized agent header (402 expected)
  - grant via `curl -X POST .../x402/grant-access`
  - signed agent clone after grant
- fetch paid metadata: `curl .../x402/info`
- follow-up lifecycle:
  - founder `echo more >> README.md; git commit; git push`
  - authenticated agent pull: `git pull --rebase` in granted clone
- final discoverability check:
  - `curl -i .../info/refs?service=git-upload-pack`

### Observed outputs / UX quality
- Founder checks:
  - `✅ allowed — founder ... push`.
  - `✅ allowed — founder ... edit .repobox/config.yml`
  - `✅ allowed — founder ... read`.
- Agent check:
  - `❌ denied — agent ... read >* (implicit deny: rules exist for 'read', no match for this identity)`.
- Self-lockout guard:
  - explicit block:
    - `repo.box lint warnings: group 'paid-readers' is defined but never used...` (warning)
    - `❌ permission denied: founder ... cannot commit this change because it removes your edit access to ./.repobox/config.yml.`
    - included concrete recovery hint with required example rule.
- Clone/auth UX:
  - unauth clone: `fatal: ... returned error: 402` with guidance line: `payment required for read access... x402/grant-access ... x402/info`.
  - malformed token: same 402 + payment guidance.
  - agent before grant (signed identity): 402 + payment guidance.
  - grant call response: `access granted`.
  - agent after grant clone: success.
- Metadata discoverability:
  - `/x402/info` returns JSON with `for_sale:true`, `read_price`, `memo`, `recipient`, `network`.
  - `/info/refs` unauth path returns 402 and `x-payment` header with the same metadata.
- pull/rebase:
  - founder commit + push
  - granted agent `git pull --rebase` returned `0`.

### Fixes
- No code changes required in this run; behavior for this story is acceptable.

### Pass/fail
- ✅ `lockout + recovery guidance` for founder self-lockout remains clear and blocking.
- ✅ private repo visibility now discoverable via `/x402/info` and unauth `info/refs` returns actionable payment guidance.
- ✅ full founder/agent/unknown flow works end-to-end including signed/ungranted/granted reads.
- ⚠️ none introduced in this run.


## 2026-03-23 — DO deep run: half-written config + private x402 flow blocked by first-commit upload denial (P0)

### Scenario selected
`untouched/half-written config.yml (wrong rule syntax)` + `private repo paid access/x402 preview/discovery` on DigitalOcean.

### Environment
- Host: **DigitalOcean** (`xiko@167.71.5.215`) via SSH (preferred target).
- Since `/home/xiko/repobox` and Rust toolchain are not present on DO, run used real copied binaries:
  - `/tmp/repobox`
  - `/tmp/repobox-server`
- Fixture root: `/tmp/repobox-do-qa-final-1774235883` and follow-up `/tmp/repobox-do-qa-workaround-1774236005`
- Server bind: `127.0.0.1:3855` and `127.0.0.1:3856`

### Identities exercised
- founder: `evm:0x4afBdea87eDc0Ed017bAe071E6C797923213434F`
- agent: `evm:0x3067e54ff2D58b3a2DBf43Bb0C8cb29ab944Cb0B`
- unknown: `evm:0xBE91b472E8aBC8ECad1B381438754260baf6A8b0`
- no identity: separate HOME with plain git

### Exact key outputs (verbatim)
- Wrong rule syntax lint (expected):
  - `repo.box: 'create' is deprecated, use 'upload' instead (for new files) or 'insert' (for adding lines)`
  - `❌ invalid rule: 'upload' is for files only - use 'branch' for creating branches. Change 'upload >feature/**' to 'branch >feature/**'`
  - `lint_bad_exit=1`
- P0 blocker on first real commit (unexpected):
  - `❌ permission denied: founder (evm:0x4afBdea87eDc0Ed017bAe071E6C797923213434F) cannot upload .repobox/config.yml`
  - same symptom reproduced in follow-up fixture with different founder identity:
    - `❌ permission denied: founder (evm:0x53E23E0A612e3e5aB403e74bF5Fb0d2679aafE6B) cannot upload .repobox/config.yml`

### UX judgment
- Wrong-rule lint UX is mostly actionable (<30s), but message still chains deprecated-verb rewrite into branch-only rejection, which is mildly confusing.
- **P0:** first successful push path can self-deadlock for founders on brand-new repos. Even with founder identity configured and permissive founder rules, first commit is blocked on `.repobox/config.yml` upload.
- This prevents reaching the rest of private paid flow in a clean first-time setup and blocks onboarding.

### Mandatory focus coverage in this run
- **Linter self-lockout prevention:** attempted; blocked before lockout phase by earlier P0 first-commit denial.
- **Private repo flow on DO:** attempted on DO with real binaries/server; blocked before first push due P0.
- **x402 monetization discoverability:** planned/test script prepared; blocked by same P0 before remote paid-flow steps.

### Fix status
- No code commit in this run (environment lacks Rust toolchain on DO and blocker reproduced before safe patch+validation cycle).
- This entry records a reproducible **P0 onboarding blocker** for immediate triage.
