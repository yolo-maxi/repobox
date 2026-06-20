Iterative Hardening in Agentic Workflows

2026-06-19 · agents, workflows, infrastructure

Agentic systems should not stay agentic forever. The best workflows use agents to discover the work, then turn repeatable pieces into deterministic code.

Most teams using agents are asking the wrong question.

They ask: how do we make the agent smarter?

The better question is: how do we make the agent necessary for fewer parts of the workflow next time?

That is the difference between agentic automation as a demo and agentic automation as infrastructure. The demo celebrates discretion. Infrastructure removes it.


The failure mode

We have been building Runyard, our workflow runner for agentic software work. It runs jobs, records traces, handles approvals, supervises agents, and keeps links back to the actual work.

In one recent pass, the UI collapsed several different outcomes into the same visual bucket: cancelled, failed, stale. Some runs were real failures. Some had produced artifacts. Some had been recovered by a human operator outside the workflow. Some had been superseded by a later run. From the product's point of view, they all looked like rubble.

That mismatch is dangerous. When the product says "failed" but reality says "shipped," the system is not merely ugly. It is lying about its own state.

Retries alone do not fix this. A retry can produce useful work and still leave the system confused about what happened.

The deeper fix is supervision: a watcher, often another constrained agent, sits above the worker. It checks whether artifacts exist, whether claims match repo state, whether a run is stuck, whether a retry superseded an earlier failure, and what state should be recorded. Atomic steps still matter, but supervision is what makes the atoms accountable. No ghost successes. No half-finished states that depend on someone remembering what happened in a chat.


Agents are scouts

Agents are excellent at discovering the shape of a job. They inspect a repo, infer a test command, write a shell script, patch a file, run checks, read the error, and try again.

That is valuable. It is also evidence.

When an agent writes the same kind of script twice, that script is no longer an improvisation. It is a candidate for the platform. When a watcher keeps applying the same recovery judgment, that judgment should become a policy. When a workflow repeatedly needs the same repo resolution, branch detection, status normalization, or artifact capture, that should move out of the prompt and into code.

The point is not to eliminate agents. The point is to stop making agents rediscover stable facts.


The hardening loop

Our emerging process is simple:

- Run the workflow agentically first.
- Record the full trace, artifacts, logs, scripts, approvals, and final state.
- Review the run for friction, not only failure.
- Delete steps that should not exist.
- Split steps that are too broad to recover cleanly.
- Extract repeated commands and scripts into deterministic code.
- Leave only the genuinely judgment-heavy parts to agents.

This borrows from a useful engineering instinct: do not automate a bad process. Simplify it first. Delete what should not be there. Make the remaining pieces legible. Only then make it faster.

In agent systems, that means the first working workflow is not the product. It is the prototype that teaches you what the product should harden.


Retrospectives need to catch drag

A failed run is not the only useful signal.

Sometimes a run succeeds but takes far too long. Sometimes the agent succeeds by writing a brittle script that should become a real tool. Sometimes a human has to merge from the outside because the workflow cannot safely mutate its own runner. Sometimes the output is technically correct but visually unusable, like raw JSON dumped where a graph should be.

A good retrospective should catch all of that.

Runyard should ask after every run: what obstructed the task? What did the agent have to infer that the system should already know? Which commands were invented during the run? Which decisions were recoverable, and which were vibes? Which failures should be hidden because they have been rerun and recovered?

This is how a workflow compounds. Not by admiring every trace, but by mining traces for things the platform can stop asking an agent to do.


Prompt optimization is not hardening

We also looked at GEPA, a system for evolving prompts and text artifacts from traces. It is interesting. It may belong in the toolbox.

But it is not hardening by itself.

A better prompt can make an agent more likely to do the right thing. Hardening changes the workflow so the agent has fewer chances to do the wrong thing.

That distinction matters. If a workflow fails because repo detection is ambiguous, the hardening move is not a prettier prompt saying "please detect the repo carefully." The hardening move is an explicit input contract, a repo map, validation against allowed roots, and a recorded decision. Once that exists, an agent can still help with ambiguous product judgment, but it no longer has to guess where the filesystem boundary is.

Our shorthand is: hardening reduces discretion. Prompt optimization improves the parts that still need discretion.


The product surface changes too

Hardening is not only backend architecture. It changes what the user sees.

A runs page should not be a pile of opaque IDs. It should show project, branch, title, description, hierarchy, recovery status, and links to logs and artifacts. Failed runs that were successfully rerun should not stay as scary red debris forever. They should become recovered history.

A workflow page should not hide the code behind a confusing modal. If the workflow is the thing being hardened, its definition, agents, approvals, runs, and history need to be inspectable.

That visibility is part of the system. You cannot harden what nobody can see.


Where this goes

The end state is not a fully deterministic build machine and it is not a swarm of agents improvising forever.

The end state is a ratchet.

Each run can use intelligence where intelligence is useful. But intelligence should leave residue. Its job is to compress hard-won judgment into tools, contracts, interfaces, and defaults, so the next encounter with the same problem is routine rather than heroic.

That is iterative hardening: agents explore, workflows remember, infrastructure absorbs the repeatable parts.

The strongest agentic systems will not be the ones with the most elaborate prompts. They will be the ones where every successful run makes the next run less agentic in exactly the right places.
