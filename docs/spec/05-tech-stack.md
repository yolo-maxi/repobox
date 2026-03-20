# repo.box Spec: Tech Stack

## Language: Rust

Chosen for type safety (security-critical permission engine), crypto ecosystem alignment (alloy, foundry, reth), and prior art (Radicle Heartwood proves Rust + git works).

## Core Libraries

### Git
- **gitoxide** (`gix`) — Pure Rust git implementation. Handles object storage, pack protocol, refs, diffs. Actively maintained by Sebastian Thiel (also maintainer of `git2` Rust bindings). Used for all local git operations.
- **Git Smart HTTP Protocol** — Implement `info/refs`, `git-upload-pack`, `git-receive-pack` endpoints using gitoxide for pack negotiation and object transfer.

### Ethereum / Crypto
- **alloy** — Modern Rust Ethereum library (by Paradigm). For ERC-8128 signature verification (`ecrecover`), ERC-1271 smart contract calls, and onchain group resolver `eth_call`s.
- **alloy-signer** — For client-side request signing (server auth, post-hackathon).
- **alloy-provider** — JSON-RPC provider for server-side onchain calls.

### HTTP
- **axum** — Async HTTP framework (by Tokio team). For the git smart HTTP endpoints, REST API, and ERC-8128 middleware.
- **tower** — Middleware layer. ERC-8128 verification as a tower middleware that extracts identity before hitting route handlers.

### Config
- **serde + serde_yaml** — Parse `.repobox.yml` YAML.
- Custom permission parser for the free-form permission syntax.

### Storage
- **Filesystem** — Bare git repos on disk (standard git storage).
- **redb** or **SQLite (rusqlite)** — Nonce dedup store, resolver cache, metadata indexes.

### CLI
- **clap** — Argument parsing for the binary.
- Single binary that operates in two modes: **git shim** (invoked as `git`, intercepts commands) and **server** (`repobox serve`).

## Architecture

```
┌─────────────────────────────────────────────┐
│              Git Shim (local)                │
│  (intercepts: commit, push, merge, branch)   │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Identity │  │Permission│  │  Real Git  │  │
│  │ (local)  │  │ Engine   │  │ (delegate) │  │
│  └──────────┘  └──────────┘  └───────────┘  │
│        │              │             │        │
│  ┌─────────────────────────────────────────┐ │
│  │    .repobox.yml parser + gitoxide diffs   │ │
│  └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                      │
                  ERC-8128
                  signed HTTP
                  (post-MVP)
                      │
┌─────────────────────────────────────────────┐
│              reporepobox serve (server)               │
│                                              │
│  ┌──────────────────────────────────────┐    │
│  │  axum HTTP server                    │    │
│  │                                      │    │
│  │  ┌────────────┐  ┌───────────────┐   │    │
│  │  │ ERC-8128   │  │ Git Smart     │   │    │
│  │  │ middleware  │  │ HTTP protocol │   │    │
│  │  └────────────┘  └───────────────┘   │    │
│  └──────────────────────────────────────┘    │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Identity │  │Permission│  │ Workflow   │  │
│  │ (verify) │  │ Engine   │  │ Engine     │  │
│  └──────────┘  └──────────┘  └───────────┘  │
│        │              │             │        │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │  Group   │  │  Config  │  │  Nonce /   │  │
│  │ Resolvers│  │  Parser  │  │  Cache DB  │  │
│  │(onchain/ │  │ (.repobox    │  │ (redb/     │  │
│  │ http)    │  │  config) │  │  sqlite)   │  │
│  └──────────┘  └──────────┘  └───────────┘  │
│        │                                     │
│  ┌─────────────────────────────────────────┐ │
│  │          gitoxide (repo storage)        │ │
│  └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## Key Design Decisions

### Single Binary
Single binary, two modes:

**Git shim mode** (invoked as `git` via PATH/symlink):
- Intercepts `git commit`, `git merge`, `git push`, `git checkout -b`, `git branch`
- Checks permissions against `.repobox.yml`, then delegates to real git
- Read-only commands pass through unchanged
- `git repobox init` — set up a repo with `.repobox.yml`
- `git repobox lint`, `git repobox check` — config tooling

**Server mode:**
- `repobox serve` — start the HTTP git server with ERC-8128 auth (post-hackathon)

### Shared Engine
The permission engine and workflow engine are library crates shared between CLI and server. Same code, same rules, enforced in both places.

### ERC-8128 as axum Middleware
```rust
// Pseudocode
async fn erc8128_middleware(req: Request, next: Next) -> Response {
    let result = verify_erc8128(&req).await;
    match result {
        Ok(identity) => {
            req.extensions_mut().insert(identity);
            next.run(req).await
        }
        Err(e) => Response::unauthorized(e),
    }
}
```

Every request hits the middleware first. By the time a route handler runs, the identity is already verified and available.

### Async Runtime
Tokio. Required by axum, alloy, and beneficial for concurrent onchain resolver calls.

## Crate Structure

```
repobox/
  crates/
    reporepobox-core/          # Identity types, group interface, permission engine
    repobox-config/        # .repobox.yml YAML parser, permission syntax parser
    reporepobox-git/           # gitoxide wrappers, smart HTTP protocol
    reporepobox-auth/          # ERC-8128 verification, alloy integration
    reporepobox-groups/        # Group resolvers (static, onchain, http)
    reporepobox-workflows/     # Workflow engine, state machine, JSONL
    reporepobox-server/        # axum HTTP server, middleware, API routes
    reporepobox-shim/          # git shim, command interception, local enforcement
  src/
    main.rs            # Binary entry point (dispatches to CLI or server)
```

## Prior Art We're Building On

| Project | What We Take | Language |
|---------|-------------|----------|
| Radicle Heartwood | "Everything in git" philosophy, Rust + gitoxide viability proof | Rust |
| Gitea/Forgejo | Git smart HTTP protocol implementation patterns | Go |
| Soft Serve | Simplicity mindset, single binary, minimal config | Go |
| ERC-8128 (@slicekit) | Auth protocol spec, reference implementation patterns | TypeScript |

## What We Don't Build (v1)

- **Web UI** — CLI and API first. Web UI is a future layer on top of the API.
- **Federation/P2P** — Single server. Not Radicle's gossip model.
- **CI/CD** — Webhook actions trigger external CI. We don't run builds.
- **Package registry** — Out of scope.
- **Git LFS** — Future addition.
