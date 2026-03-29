# Native Sandboxing for Codex and Claude Code

This note captures the **native sandboxing features already provided by Codex and Claude Code**, and how repo.box should use them.

## Why this matters

We tried to make direct `git` bypass impossible by replacing the system `git` entrypoint. That was the wrong layer.

The safer architecture is:

- **repo.box** enforces repo policy
- **the coding harness** enforces runtime boundaries
- **the host OS** stays boring and stable

So if we want agents to be unable to casually bypass repo.box, the right move is to run them in a sandboxed execution environment rather than hijacking `/usr/bin/git`.

## Bottom line

Both tools already have native sandbox stories:

- **Codex** has built-in sandbox modes and approval policies
- **Claude Code** has built-in OS-level sandboxing plus approval / auto modes

That means repo.box does **not** need to replace system git in order to get stronger agent isolation.

## Codex native sandboxing

Source checked:
- OpenAI Developers — `https://developers.openai.com/codex/concepts/sandboxing`

### What Codex supports

Codex documents native sandboxing across its app, IDE, and CLI.

Key points from the official docs:

- The sandbox applies to **spawned commands**, not just file edits
- That means commands like `git`, package managers, test runners, and scripts inherit the same boundary
- It supports platform-native enforcement depending on OS
- Defaults are configurable via `config.toml`

### Main sandbox modes

Codex documents these common modes:

- `read-only`
  - inspect files only
  - no edits or commands without approval

- `workspace-write`
  - read files
  - edit within the workspace
  - run routine local commands inside that boundary
  - best default for local coding

- `danger-full-access`
  - no sandbox restrictions
  - should be treated as opt-out / unsafe mode

### Approval policies

Codex also separates sandboxing from approvals.

Officially documented approval policies:

- `untrusted`
- `on-request`
- `never`

Important distinction:

- **sandbox** = technical boundary
- **approval policy** = when the agent must stop and ask

### Useful implication for repo.box

For Codex, the clean model is:

- run Codex in `workspace-write`
- keep approval policy at `on-request` or stricter
- give it only the repo / worktree paths it should edit
- rely on repo.box for final git policy enforcement

## Claude Code native sandboxing

Sources checked:
- Claude Code Docs — `https://code.claude.com/docs/en/sandboxing`
- Anthropic engineering post — `https://www.anthropic.com/engineering/claude-code-auto-mode`

### What Claude Code supports

Claude Code has a native sandboxed bash tool with **OS-level enforcement**:

- **macOS**: Seatbelt
- **Linux / WSL2**: bubblewrap

Official docs explicitly say effective sandboxing needs **both**:

- filesystem isolation
- network isolation

That point matters a lot. Filesystem-only sandboxing is not enough if the agent can still exfiltrate data over the network.

### Main Claude Code sandbox behavior

Official docs describe:

- sandbox enabled for bash commands
- configurable filesystem allow/deny rules
- configurable network/domain allow rules
- sandbox restrictions inherited by subprocesses
- optional hard-fail if sandbox is unavailable

Relevant settings/features mentioned in the docs include:

- `sandbox.failIfUnavailable`
- `sandbox.filesystem.allowWrite`
- `sandbox.filesystem.denyWrite`
- `sandbox.filesystem.denyRead`
- `sandbox.filesystem.allowRead`
- managed-domain restrictions such as `allowManagedDomainsOnly`
- `excludedCommands` for tools that must run outside sandbox

### Claude sandbox modes

The docs describe two operational modes:

- **Auto-allow mode**
  - commands that can run safely inside the sandbox run automatically
  - commands outside the boundary fall back to permission flow

- **Regular permissions mode**
  - commands still go through normal approval flow even when sandboxed

Important nuance:

- the **sandbox boundary stays the same**
- what changes is whether sandbox-safe commands are auto-approved

### Claude auto mode is not the same thing as sandboxing

Anthropic’s auto-mode post adds another layer:

- model-based approval/classifier logic
- prompt-injection screening on tool outputs
- better autonomy with less approval fatigue

That is useful, but for repo.box purposes we should treat it as **supplementary**, not the primary safety boundary.

The hard boundary should still be:

- OS sandbox
- limited filesystem scope
- limited network scope
- repo.box server-side policy

## Recommended repo.box stance

### 1. Never replace system git

Do **not**:

- symlink `/usr/bin/git` to repo.box
- rename system git to force interception
- modify host-level canonical git paths

That approach is too easy to recurse, confuse tooling, or destabilize the machine.

### 2. Prefer native harness sandboxing first

For coding agents, prefer:

- **Codex** in native `workspace-write`
- **Claude Code** with native sandboxing enabled
- hard-fail if sandbox is unavailable for managed runs

### 3. Restrict the writable area aggressively

Good default:

- writable: current repo / worktree only
- maybe a small temp/build directory if needed
- no write access to `$HOME` broadly
- no write access to shell startup files, SSH config, global git config, or system paths

### 4. Restrict network where possible

If the harness supports it, allow only what is needed, such as:

- repo.box / git remote
- GitHub (if required)
- package registries only when installation is part of the task

Default-deny is better than trying to detect bad outbound behavior later.

### 5. Give each agent its own HOME / identity context

Even with native sandboxing, we should isolate agent state:

- separate HOME
- separate git config
- separate repo.box identity / key material
- ephemeral worktree when possible

### 6. Let repo.box be the final enforcement layer

Sandboxing reduces bypasses.
repo.box should still enforce:

- branch permissions
- file/path permissions
- merge restrictions
- force-push restrictions
- signature / identity checks

That way:

- the sandbox constrains what the agent can attempt locally
- repo.box constrains what can actually land remotely

## Practical recommendation by tool

### Codex

Recommended baseline:

- sandbox: `workspace-write`
- approval: `on-request`
- writable roots: repo/worktree only
- no danger-full-access in unattended runs

### Claude Code

Recommended baseline:

- native sandbox enabled
- sandbox hard-fails if unavailable
- auto-allow only if filesystem + network boundaries are configured tightly
- otherwise regular permissions mode
- avoid broad `excludedCommands`

## Decision

For repo.box, the path forward should be:

1. use **native Codex / Claude Code sandboxing**
2. scope the writable workspace tightly
3. isolate agent HOME / credentials
4. keep repo.box as the git policy layer
5. **never again** try to enforce this by replacing `/usr/bin/git`

## References

- Codex sandboxing: `https://developers.openai.com/codex/concepts/sandboxing`
- Claude Code sandboxing: `https://code.claude.com/docs/en/sandboxing`
- Claude Code auto mode: `https://www.anthropic.com/engineering/claude-code-auto-mode`
