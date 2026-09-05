# repo.box public route live sample — 2026-09-05

Companion evidence for `public-route-disposition-2026-08-17.html` and RunYard
Factory Item `fi_7f852c33229de5ffbe07`.

The 2026-08-17 disposition was a source inventory with no live HTTP sampling.
This file records the missing live sample and resolves the two dispositions the
item was blocked on (`/releases/latest` and the cited API paths).

Sampled 2026-09-05T00:0xZ from the Home host against production `https://repo.box`.

## App Router routes (all healthy)

| Route | Status |
| --- | --- |
| `/` | 200 |
| `/git` | 200 |
| `/hire` | 200 |
| `/portfolio` | 200 |
| `/proof` | 200 |
| `/projects` | 200 |
| `/projects/supstrategy` | 200 |
| `/trust` | 200 |
| `/playground` | 200 |
| `/building` | 200 |
| `/packages` | 200 |
| `/made-by-agents` | 200 |
| `/agents` | 200 |
| `/agents/ocean-vael` | 200 |
| `/blog/case-study-cabin` | 200 |
| `/docs` | 307 (redirect, intentional) |
| `/blog/` | 308 (redirect) |

## API paths cited by the disposition (resolved)

| Route | Status | Disposition |
| --- | --- | --- |
| `/api/trust-status` | 200 | keep — backs `/trust` |
| `/api/activity-heatmap` | 200 | keep — backs homepage/projects |
| `/api/demo` | 400 | keep — 400 is argument validation on a bare GET, not an outage |
| `/api/hire` | (POST-only) | keep — exercised by `/hire`, not sampled with GET |
| `/api/badge-clicks` | (POST-only) | keep — backs `/made-by-agents` embeds |

## `/releases/latest` — resolved

**Disposition: remove from sitemap.**

`/releases/latest` returns **404**. It has no App Router page and no static file.
The only occurrence anywhere in the repository is the `<loc>` entry in
`repobox-landing/public/sitemap.xml`. It is a stale sitemap entry advertising a
route that has never existed in this codebase, not a broken or regressed page.

## Static `public/` surface — all 404 in production

This is the significant finding and was not visible from source inspection.
Every asset under `repobox-landing/public/` that is not served by the App Router
returns 404 on the live site:

`/feed.xml`, `/llms.txt`, `/SKILL.md`, `/whitepaper.txt`, `/canvas.js`,
`/jellyfish.json`, `/manta-final.json`, `/manta-loop.json`, `/file.svg`,
`/globe.svg`, `/next.svg`, `/vercel.svg`, `/window.svg` — all **404**.

`/sitemap.xml` and `/robots.txt` return 200, so the static root is not wholly
unmounted; the failure is selective and needs a deploy-side explanation.

Consequence for the existing dispositions: `REPOBOX-ASSETS-001`,
`REPOBOX-ASSETS-002` and `REPOBOX-CONTENT-001` are already satisfied in
production behaviour — those assets are not publicly reachable. They remain in
the repository but are not being served.

## Sitemap accuracy

The live sitemap is byte-identical in URL set to `public/sitemap.xml` (18 URLs).
**14 of 18 return 404:**

`/blog`, `/releases/latest`, and all 12 extensionless `/blog/<slug>` entries.

Only `/`, `/projects`, `/made-by-agents` and `/playground` resolve. This
confirms and quantifies `REPOBOX-SITEMAP-001` and `REPOBOX-BLOG-001`: the
sitemap advertises a blog that is not served at those URLs, while the one
working blog URL (`/blog/case-study-cabin`, an App Router page) is absent from
the sitemap.

## Remaining unknowns

- Why the static `public/` surface 404s while `sitemap.xml`/`robots.txt` serve.
  Likely Next standalone output not copying `public/`, but the deploy host was
  not inspected and this is unconfirmed.
- Whether the ten static blog HTML posts should be restored at their sitemap
  URLs or the sitemap should be regenerated to match served reality. That is a
  content decision for `REPOBOX-BLOG-001`, not an inventory gap.

## Root cause of the static `public/` 404s — determined 2026-09-05T02:27Z

The selective failure is fully explained. Two different servers answer for
`repo.box`, and only two exact paths are intercepted.

**1. `sitemap.xml` and `robots.txt` never reach the Next app.** The Caddy site
block for `repo.box` has explicit `handle` blocks for `/releases/*`,
`/install.sh`, `/sitemap.xml`, `/robots.txt` and `/assets/*`, each with
`root * /var/www/repo.box/subdomains/root` + `file_server`. Everything else
falls through to `handle { reverse_proxy localhost:3480 }`.

Evidence — response headers differ by origin:
- `/sitemap.xml` -> `server: Caddy`, `last-modified: Mon, 30 Mar 2026 06:09:48 GMT`
  (Caddy's own file_server, static root)
- `/llms.txt`    -> `via: 1.1 Caddy`, `content-type: text/html`, Next RSC `vary`
  headers and a `_next-static-sunset-git-20260817` preload link
  (Next app's 404 page, proxied)

So `sitemap.xml`/`robots.txt` serving is **not** evidence that `public/` is
mounted. They are served from a different directory entirely and are stale
(March 2026), which is also why the sitemap lists URLs that no longer exist.

**2. `public/` is not copied into the standalone build.** `next.config.ts` sets
`output: "standalone"`. Next's standalone output deliberately does **not** copy
`public/` or `.next/static`; the deploy is responsible for that step, and here
it was never done.

Decisive evidence:
- `repobox-landing/public/` contains 23 entries.
- `repobox-landing/.next/standalone/public/` contains exactly 1: `heatmap-data.json`.
- `https://repo.box/heatmap-data.json` returns **200** with `via: 1.1 Caddy`
  (served by the Next app, not the static root).
- Every other `public/` file — `/llms.txt`, `/feed.xml`, `/SKILL.md`,
  `/favicon.svg`, `/whitepaper.txt` — returns **404**.

One file in the standalone `public/` dir serves; the 22 that are missing from it
do not. That is a one-to-one match with the observed 404 set and rules out
prompt-level guesses such as route conflicts or Caddy misrouting.

**Consequence for the fix.** This is a deploy/packaging defect, not a source
defect. Restoring the files in source would change nothing. The build or deploy
step must copy `public/` (and `.next/static`) into the standalone output, per
Next's documented standalone requirements.

**Not yet determined.** Which build/deploy script produces the running service
was not identified: no `3480` listener and no matching systemd unit are visible
from this host, and `repo.box` resolves to `204.168.190.248`, which is not this
machine (`77.42.89.161`). The running app is therefore deployed elsewhere, and
the deploy script that must be corrected has not been located.
