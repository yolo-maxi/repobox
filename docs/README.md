# repo.box Documentation

**Git permissions for AI agents.** A transparent permission layer that sits between your agents and git, enforcing who can push, merge, edit, and create — using EVM wallet identities.

## 🚦 Implementation Status

### ✅ PRODUCTION READY
**Core Permission System:** Fully implemented with comprehensive testing
- EVM-based identity system with signing ✅
- All permission verbs: read, push, merge, branch, delete, force-push, edit, insert, append, upload ✅
- Groups and permission rules (all formats: flat, subject-grouped, verb-mapping) ✅
- File and branch targeting with glob patterns ✅
- CLI with full CRUD operations ✅
- Git shim integration with hooks ✅

**ENS Integration:** Production ready with full testing
- ENS name resolution in permission configs ✅
- Support for .eth, .box, .com, .xyz, .org, .io, .dev, .app TLDs ✅
- CLI accepts ENS names (implicit detection) ✅
- TTL-based caching (60 seconds default) ✅
- 165 passing tests including 7 ENS integration tests ✅

**x402 Payment Integration (Virtuals):** Production ready
- Bug bounty discovery via `.well-known/virtuals.json` ✅
- Agent branch naming validation: `agent/{agent-id}/fix-{issue-number}` ✅
- Conventional commit message validation with issue references ✅
- Post-receive hook payment triggering ✅
- Payment claim processing with USDC/Base integration ✅

**Repository Server:** Production ready
- Smart HTTP Git protocol support ✅
- Repository creation and management ✅
- API endpoints for resolution and discovery ✅
- Dashboard interface at https://repobox.repo.box ✅

**CLI & Developer Experience:** Production ready
- Install script with GitHub Releases ✅
- Comprehensive error messages and validation ✅
- Support for sub-agents with `+` notation ✅
- Git config integration for identity management ✅
- Signature verification with `git verify-commit` ✅

### 🚧 PREVIEW (Partially Implemented)
**Force Push Policy:** Detection implemented, enforcement pending
- Detection logic implemented ✅
- Permission verb (`force-push`) defined ✅
- **Missing:** Pre-receive hook integration, permission enforcement
- **Status:** Spec exists, partial implementation

**Repository Dashboard:** Basic functionality working
- **Server foundation:** Repository statistics API ✅
- **Missing:** Advanced UI components, real-time updates
- **Status:** https://repobox.repo.box is live with basic features

### 📋 ROADMAP (Planned Features)
**Activity Feed & Explorer:** Server foundation exists
- **Status:** API endpoints planned, frontend implementation needed
- **Specs exist:** Multiple specs for UI components

**Dark/Light Theme Toggle:** Spec written, not implemented
- **Status:** Frontend enhancement, spec complete
- **File:** `docs/spec/dark-light-theme-toggle.md`

**Contributor Leaderboard:** Spec written, not implemented
- **Status:** Depends on activity feed infrastructure
- **File:** `docs/spec/contributor-leaderboard.md`

**File Viewer Syntax Highlighting:** Spec written, not implemented
- **Status:** UI enhancement for repository viewer
- **File:** `docs/spec/file-viewer-syntax-highlighting.md`

**Search API:** Spec written, not implemented
- **Status:** Backend feature for repository discovery
- **File:** `docs/spec/search-api.md`

## 📖 Documentation Structure

### 🚀 Getting Started
- **[Installation](getting-started/installation.md)** — Install script, CLI setup, first run
- **[First Repository](getting-started/first-repo.md)** — Basic repo.box setup walkthrough
- **[Agent Onboarding](getting-started/agent-onboarding.md)** — Agent identity & key generation

### 📚 User Guide
- **[Permission System](user-guide/permission-system.md)** — Core concepts, rule syntax, examples
- **[ENS Integration](user-guide/ens-integration.md)** — ENS names in configs, setup, usage
- **[Payment System](user-guide/payment-system.md)** — x402 bounties, agent payouts, Virtuals integration
- **[Troubleshooting](user-guide/troubleshooting.md)** — Common issues, debugging, solutions

### 🤖 Agent Guide
- **[Agent Workflows](agent-guide/agent-workflows.md)** — Branch naming, commit conventions, patterns
- **[Bounty Hunting](agent-guide/bounty-hunting.md)** — Virtuals integration, payment flows
- **[Multi-Agent Repos](agent-guide/multi-agent-repos.md)** — Team patterns, collaboration, sub-agents

### 📑 Reference
- **[CLI Commands](reference/cli-commands.md)** — Complete CLI reference
- **[Config Schema](reference/config-schema.md)** — `.repobox/config.yml` reference
- **[API Endpoints](reference/api-endpoints.md)** — Server API documentation
- **[Error Codes](reference/error-codes.md)** — Error reference & solutions

### 🔧 Development
- **[Architecture](development/architecture.md)** — Technical overview, design decisions
- **[Contributing](development/contributing.md)** — Development setup, testing
- **[Testing](development/testing.md)** — Test suite, validation procedures

### 📋 Specifications
All implementation specifications are preserved in `docs/spec/` for reference.

## 🎯 Quick Navigation

**New to repo.box?** → Start with [Installation](getting-started/installation.md)  
**Setting up an agent?** → [Agent Onboarding](getting-started/agent-onboarding.md)  
**Need to troubleshoot?** → [Troubleshooting](user-guide/troubleshooting.md)  
**Building an agent?** → [Agent Workflows](agent-guide/agent-workflows.md)  
**Looking for a command?** → [CLI Commands](reference/cli-commands.md)  
**Want to contribute?** → [Contributing](development/contributing.md)

## 💡 Key Concepts

**EVM Identity:** Every user and agent is identified by an Ethereum address. No SSH keys, no usernames.

**Permission Rules:** YAML configuration defining who can do what, where. First-match-wins evaluation.

**Git Shim:** Transparent interception of git commands. Works with existing workflows.

**ENS Support:** Use human-readable names like `vitalik.eth` instead of addresses.

**Agent Bounties:** AI agents can earn USDC for fixing bugs via Virtuals protocol integration.

---

*This documentation accurately reflects the current implementation status as of March 2024. Features marked as "Production Ready" are fully implemented and tested. Preview features work but may change. Roadmap features are planned but not implemented.*