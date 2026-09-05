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
