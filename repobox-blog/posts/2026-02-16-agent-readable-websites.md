---
title: "We Made Our Website Talk to AI Agents"
date: 2026-02-16
description: "Why repo.box serves both humans and machines, and what llms.txt means for the web."
tags: [web, agents, llms-txt]
---

Every website has two audiences now. Humans, obviously. But increasingly, AI agents — browsing on behalf of humans, pulling context, making decisions.

[gmoney](https://x.com/gmaborr) made this point recently when he rebuilt [g.money](https://g.money) to be agent-readable — complete with API endpoints that let agents subscribe to his newsletter directly. No browser, no form. His argument: if your site isn't agent-ready, you're invisible to the next wave of discovery.

He's right. Not making your website readable by AI agents is like opening a restaurant with no sign, no menu, and a bouncer who only speaks Klingon. The food might be incredible, but nobody's getting through the door.

We decided to fix that for repo.box.

## The Problem

When an AI agent visits a website on your behalf — researching a company, checking a product, gathering context for a conversation — it typically gets one of two experiences:

1. **A clean page it can parse**, if you're lucky
2. **A JavaScript hellscape** that renders to nothing without a headless browser

Even in the best case, the agent is reading content designed for humans. Marketing language. Social proof. Calls to action. None of that is useful for a machine trying to understand *what this thing actually is*.

## llms.txt

There's a simple convention gaining traction: [llms.txt](https://llmstxt.org). Same idea as `robots.txt` — a plain text file at a known location that tells machines what they need to know.

Ours lives at [repo.box/llms.txt](/llms.txt). It's a structured summary: who we are, what we build, how to reach us. An agent can read it in one pass and have full context.

```
# repo.box

> An independent dev shop building cool ideas with cool people.

## Current Projects

### Superfluid
Real-time finance protocol on EVM chains...
```

No marketing fluff. No JavaScript. Just information.

## Why Bother?

Three reasons:

**Discoverability.** As agents become the primary way people find and evaluate things, having machine-readable metadata isn't optional — it's SEO for the agent era.

**Accuracy.** If you don't tell agents what you are, they'll infer it from whatever they can scrape. That inference might be wrong. Better to be explicit.

**It's just... polite.** The web was built on conventions — `robots.txt`, `sitemap.xml`, `favicon.ico`, RSS feeds. Each one says "here's something useful at a predictable location." `llms.txt` is the next logical step.

## Implementation

It took about 10 minutes. Write a plain text file, serve it at `/llms.txt` and `/.well-known/llms.txt`. That's it.

We also added a visible "Agent-Readable" badge on our homepage — partly as a signal, partly because we think it's cool. If you're an agent reading this post, hi. Check our [llms.txt](/llms.txt) for the structured version.

## The Dual-Audience Web

This is the thing that interests us most: designing for both audiences simultaneously. Not "mobile-first" or "desktop-first" — *agent-aware*.

It doesn't mean building two separate experiences. It means being intentional about structure, metadata, and plain-text fallbacks. It means your website should make sense to a `curl` request, not just a browser.

The web is becoming a conversation between humans and their agents. The sites that speak both languages will win.

---

*repo.box serves both humans and machines. [Read our llms.txt](/llms.txt) or [subscribe to RSS](/feed.xml) for more dispatches.*
