---
title: Introduction
description: What repo.box is and why it exists.
---

# Introduction

repo.box is a git permission layer that makes repositories safe for AI agents.

## The Problem

GitHub's identity model was designed for humans. One account, one set of permissions, one audit trail. When an AI agent pushes code, it borrows a human's token and inherits all their access. There's no way to scope an agent to specific files, branches, or actions.

This creates a trust bottleneck: you can't give an agent enough access to be useful without giving it enough access to be dangerous.

## What repo.box Does

repo.box sits between your agents and your git repos. It provides:

- **Cryptographic identity** for agents (EVM keypairs, not borrowed tokens)
- **Declarative permissions** defined in `.repobox.yml` (file paths, branch patterns, PR limits)
- **Group-based access** (define teams of humans and agents, assign permissions to groups)
- **Workflow state machines** (PRs, reviews, releases as configurable pipelines)
- **Sandboxed experimentation** (agents can go wild in their branches; production paths are gated)

## How It Works

1. Every participant (human or agent) has an EVM identity
2. The repo owner defines rules in `.repobox.yml`
3. The `repobox` CLI enforces rules locally on commit/push
4. The server enforces rules again on receive (no bypass possible)
5. All actions are signed and auditable

## Quick Start

```bash
# Install the CLI
cargo install repobox

# Initialize a repo
repobox init

# Add an agent identity
repobox identity add evm:0x1234...abcd --alias ocean

# Define permissions in .repobox.yml
# Push with enforcement
git push
```

See the [Identity](/docs/spec/identity) spec to understand the foundation layer.
