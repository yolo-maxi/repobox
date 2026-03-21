# repo.box — Git Permission Layer for AI Agents

EVM-signed commits. On-chain group resolvers. Append-only enforcement. Agent-native git.

## What is repo.box?

repo.box replaces SSH keys and GitHub accounts with EVM wallets. Every commit is signed with an Ethereum key. Permissions are defined per-repo in `.repobox/config.yml` and enforced server-side. Groups can resolve on-chain (token holders, NFT owners, DAO members).

**Platform**: https://repo.box
**Explorer**: https://repo.box/explore
**Git server**: https://git.repo.box

## Installing the CLI

```bash
curl -sSf https://repo.box/install.sh | sh
```

This installs the `repobox` binary. No global git hooks, no PATH changes — it only affects repos where you run `repobox init`.

**Optional:** To enforce permissions locally (before pushing), run `repobox setup`. This shims your git command, so that in repos where you've run `repobox init`, permission rules are checked client-side — unauthorized changes get blocked before they leave your machine. With the shim active, all commands become git subcommands: `git repobox init`, `git repobox keys generate`, etc.

## Setting Up Your Identity

```bash
# Generate a new EVM key
repobox keys generate

# Or import an existing private key
repobox keys import <private-key>

# Check your identity
repobox whoami
```

Keys are stored locally at `~/.repobox/keys/<address>.key`.

## Using repo.box in a Repo

```bash
# Clone any repo.box repo
git clone https://git.repo.box/<owner-address>/<repo>.git
cd <repo>

# Initialize repo.box (sets gpg.program locally, creates .repobox/config.yml)
repobox init

# Commits are now EVM-signed automatically (commit.gpgsign = true)
git add . && git commit -m "signed with my EVM key"
git push origin main
```

`repobox init` configures `gpg.program` to point to the repobox binary and enables `commit.gpgsign` — both local to the repo. Your other repos are unaffected.

## Permissions

Each repo has `.repobox/config.yml`:

```yaml
groups:
  maintainers:
    - evm:0xAlice...
    - evm:0xBob...
  tokenholders:
    - resolver:https://api.example.com/holders

permissions:
  default: deny
  rules:
    - "* read ./**"              # anyone can clone
    - "* append ./data.jsonl"    # anyone can append
    - "* push main"              # anyone can push to main
    - "maintainers edit ./**"    # maintainers can edit anything
```

**Verbs**: `read` (clone/fetch), `write` (create new files), `append` (add lines, no deletions), `edit` (modify/delete — superset of write and append), `push`, `merge`

The server enforces these rules on push. Append-only means the server inspects the diff — additions only, zero deletions. With `repobox setup`, these same rules are also enforced locally before push.

## On-Chain Group Resolvers

Groups can be resolved dynamically via HTTP:

```yaml
groups:
  nft-holders:
    - resolver:https://your-api.com/holders?contract=0x...
```

The resolver returns a JSON array of addresses. Checked at push time.
