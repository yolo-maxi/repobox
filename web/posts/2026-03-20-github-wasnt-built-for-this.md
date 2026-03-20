---
title: "GitHub Wasn't Built for This"
date: 2026-03-20
description: "We ship code every day. Our agents open PRs, run CI, manage repos. But they're borrowing human identities to do it."
tags: [agents, infrastructure, identity]
---

We ship code every day. Our agents open pull requests, spawn sub-agents to implement features in parallel, review diffs, push to production. On a good day they'll ship a dozen changes across multiple repos before anyone finishes their coffee.

But every commit says `Francesco Renzi`. Every PR opens under his token. Every permission the agent has is his permission, delegated through a personal access token that can do everything he can.

## Borrowed Identity

GitHub has one concept of identity: the user. A human with an email, a password, a profile picture. Permissions, commit attribution, access control: all designed around the assumption that a person is typing.

When our agent pushes code, GitHub sees a human. It doesn't know the agent exists. There's no way for it to say "I'm Ocean, I work for Fran, and these are the things I'm allowed to do." The token was generated from a human account, with human permissions, and every action the agent takes is legally and technically *that human's* action.

Giving your agent the same keycard as the account owner and hoping it only opens the right doors. That's the current state of the art.

## Permissionless Keys

Agents don't need human keys. They need their own.

Not keys derived from a human account. Not keys that inherit human permissions. Keys the agent generates, controls, rotates, and revokes independently. Keys that let a repo owner say "this agent can open PRs on my project" without also granting access to every other repo the human owns.

Cryptographic identity makes this trivial. The agent generates a keypair. Signs its commits. Anyone can verify that commit came from a specific agent with a specific key, not from "some bot" using a human's token. The repo owner decides whether to trust that key. No platform in the middle gatekeeping who gets to participate.

This is how blockchain wallets already work. Generate a key, sign things, prove who you are without asking permission from a platform. The pattern exists. It just hasn't been applied to code collaboration yet.

## Self-Defined Rules

When we onboard an agent to a project, there's a conversation. Architecture, conventions, what's off-limits. The agent reads AGENTS.md and understands the boundaries.

None of that is machine-readable to the platform. GitHub doesn't know the agent shouldn't touch the billing module. It doesn't know modifications should stay under `/src/agents/`. The rules exist in natural language, in the agent's context, and if it hallucates or the context window fills up, those guardrails vanish.

What if the repo itself could declare agent rules? A manifest: *"Agent Ocean can write to /src/ and /tests/. Max 500 lines per PR. Must include tests. Cannot modify CI config."* Enforced at the protocol level. Not guidelines. Actual constraints that can't be violated, even accidentally.

This isn't restrictive. It's liberating. Right now you have to trust your agent with everything because the platform can't express granular permissions. With declarative rules, you can give real autonomy over the parts that matter and hard limits on the parts that don't. More trust, not less, because the trust is bounded.

## Strict Guardrails, Yolo Mode

We spawn sub-agents constantly. Each one gets a task, a sandboxed environment, a set of tools. They go wild inside the sandbox: try things, break things, iterate. The blast radius stays contained. If a sub-agent corrupts its working directory, we lose that task. We don't lose the repo.

Git hosting doesn't work like this. A force-push to main is just as easy as a push to a feature branch. Branch rules are a blunt instrument designed for humans who might click the wrong button, not for agents processing thousands of operations.

What agents need: let them experiment freely in their own space. Branches, wild prototypes, throwaway experiments. But make the path from experiment to production narrow, audited, and gated. Make the safe path the fast path. The dangerous path requires explicit human approval.

The best sandboxes don't feel like cages. They feel like workshops.

## The Inevitable Rewrite

Every decade, the assumptions underneath developer tools shift enough that the whole stack gets rebuilt. CVS to SVN to Git. Self-hosted to GitHub. Manual deploys to CI/CD. Each transition happened because the old tool was built for a world that no longer existed.

The world where every contributor is a human, where identity means "person with an email," where permissions are inherited from a single account: that world is already gone. Every Codex session, every Cursor commit, every Devin task proves it.

The question isn't whether git hosting gets rebuilt for agents. It's whether the rebuild happens as a clean break or as patches on a human-first platform that never anticipated this.
