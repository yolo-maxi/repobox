# repobox-landing

Next.js app for the repo.box homepage and marketing/docs routes.

## Local Development

Use the checked-in pnpm lockfile as the source of truth.

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm dev
```

The dev server listens on `http://localhost:3480`.

## Build From Source

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm build
```

The app uses `output: "standalone"` in `next.config.ts`, so a production build emits a self-contained Node server at `.next/standalone/server.js`. The default static asset prefix is `/_next-static-sunset-git-20260817` to match the current repo.box cache-busting deployment shape. Override it only for a coordinated deploy:

```bash
REPOBOX_ASSET_PREFIX="/_next-static-<release-id>" pnpm build
```

Browser source maps are disabled in production config. Do not commit `.next/`, `out/`, source maps, env files, `next-env.d.ts`, or generated monitor output.

## Standalone Deploy Shape

**Use `./scripts/package-standalone.sh`.** Do not hand-roll the copy steps below.

`next build` does *not* copy `public/` or `.next/static` into
`.next/standalone`. On this app it emits a `.next/standalone/public` containing
only `heatmap-data.json`, so a standalone server started against the raw build
output returns **404 for `/sitemap.xml`, `/robots.txt`, `/llms.txt`, `/SKILL.md`,
`/feed.xml`, `/favicon.svg` and every `/blog/*` file**. That is the production
404 signature diagnosed on `fi_6577f2d0ab9f8e4550a1`; the build reproduces it
every time, so it is a packaging defect, not an rsync mistake.

```bash
./scripts/package-standalone.sh              # build + package + verify
./scripts/package-standalone.sh --no-build   # package an existing build
```

The script flattens both trees, refuses to continue if the nested-`public/public`
shape survives or the entry count does not match, then boots the packaged server
and sweeps the asset routes. It exits non-zero if any of them is not 200, so a
broken tree cannot be shipped silently.

### Two static roots

repo.box does **not** serve everything from this app. The Caddy site has explicit
`handle` blocks serving these paths from `/var/www/repo.box/subdomains/root`,
bypassing Next entirely:

    /sitemap.xml   /robots.txt   /releases/*   /install.sh   /assets/*

Those files exist in two places and the Caddy copy wins. Editing them in
`public/` alone will build, package and verify green here and still not change
the live site. Update the Caddy static root as a deliberate separate step.

The checked-in `ecosystem.config.js` starts `.next/standalone/server.js` with
`PORT=3480`, `HOSTNAME=0.0.0.0`, `NODE_ENV=production`.


If `REPOBOX_ASSET_PREFIX` is set, the reverse proxy must serve that prefix to the same files as `.next/static`. With the default prefix, `/_next-static-sunset-git-20260817/_next/static/...` must resolve to the built static chunks. Keep the old prefix mounted until clients no longer hold cached HTML that references it.
