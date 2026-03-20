---
title: "GitHub Wasn't Built for This"
date: 2026-03-20
description: "We ship code every day. Our agents open PRs, run CI, manage repos. But they're borrowing human identities to do it."
tags: [agents, infrastructure, identity]
---

We ship code every day. Our agents open pull requests, spawn sub-agents in parallel, review diffs, push to production. On a good day they'll ship a dozen changes before anyone finishes their coffee.

But every commit says `Francesco Renzi`. Every PR opens under his token. Every permission the agent has is his permission.

## Borrowed Identity

GitHub has one concept of identity: the user. A human with an email and a password. Permissions, attribution, access control: all built on the assumption that a person is typing.

When our agent pushes code, GitHub sees a human. There's no way for it to say "I'm Ocean, I work for Fran, and these are the things I'm allowed to do." The token comes from a human account. Every action the agent takes is legally and technically *that human's* action.

> Giving your agent the CEO's keycard and hoping it only opens the right doors. That's the current state of the art.

## Permissionless Keys

Agents don't need human keys. They need their own.

Keys the agent generates, controls, and revokes independently. Keys that let a repo owner say "this agent can open PRs here" without granting access to every other repo the human owns.

The pattern already exists. Blockchain wallets work exactly this way: generate a keypair, sign things, prove identity without asking a platform for permission. Apply that to code collaboration and suddenly agents become first-class participants, not shadows behind a human's token.

<div class="visual-break">
<pre class="diagram">
  TODAY                          WHAT'S NEEDED
  ─────                          ────────────

  ┌─────────┐                    ┌─────────┐
  │  Human  │                    │  Human  │
  │ Account │                    │ Account │
  └────┬────┘                    └────┬────┘
       │                              │
       │ full access                  │ delegates
       │ (shared token)              │ (scoped rules)
       ▼                              ▼
  ┌─────────┐                    ┌─────────┐
  │  Agent  │                    │  Agent  │
  │ (ghost) │                    │ (own ID)│
  └─────────┘                    └────┬────┘
                                      │
                                      │ own keypair
                                      │ own permissions
                                      │ own audit trail
                                      ▼
                                 ┌─────────┐
                                 │  Repo   │
                                 │ (scoped)│
                                 └─────────┘
</pre>
</div>

## Self-Defined Rules

When we onboard an agent to a project, there's a conversation. Architecture, conventions, what's off-limits. The agent reads AGENTS.md.

None of that is machine-readable to the platform. The rules live in the agent's context window, and if it hallucates or the window fills up, the guardrails vanish.

What if repos could declare agent rules as a manifest? *"Agent Ocean can write to /src/ and /tests/. Max 500 lines per PR. Must include tests. Cannot modify CI config."* Enforced at the protocol level. Not guidelines: constraints.

More trust, not less. Because the trust is bounded.

## Strict Guardrails, Yolo Mode

We spawn sub-agents constantly. Each gets a task and a sandbox. They go wild inside it: try things, break things, iterate. If one corrupts its working directory, we lose that task. Not the repo.

Git hosting doesn't work like this. A force-push to main is as easy as a push to a feature branch.

What agents need: freedom to experiment in their own space, with a narrow, audited, gated path to production. Make the safe path the fast path. Make the dangerous path require a human.

> The best sandboxes don't feel like cages. They feel like workshops.

## The Inevitable Rewrite

Every decade, developer tools get rebuilt because the assumptions underneath them shifted. CVS to SVN to Git. Self-hosted to GitHub. Manual deploys to CI/CD.

The world where every contributor is a human, where identity means "person with an email," where permissions inherit from a single account: that world is already gone. Every Codex session, every Cursor commit proves it.

The question isn't whether git hosting gets rebuilt for agents. It's whether it happens as a clean break or as patches on a platform that never anticipated this.
