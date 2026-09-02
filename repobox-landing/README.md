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

Build on the builder host, then ship only the files needed by the standalone server and static/public assets:

```bash
pnpm build
rm -rf .next/standalone/.next/static .next/standalone/public
cp -R .next/static .next/standalone/.next/static
cp -R public .next/standalone/public
```

The checked-in `ecosystem.config.js` expects the release to live at `/home/xiko/repobox-landing` and starts `.next/standalone/server.js` with `PORT=3480`, `HOSTNAME=0.0.0.0`, and `NODE_ENV=production`.

If `REPOBOX_ASSET_PREFIX` is set, the reverse proxy must serve that prefix to the same files as `.next/static`. With the default prefix, `/_next-static-sunset-git-20260817/_next/static/...` must resolve to the built static chunks. Keep the old prefix mounted until clients no longer hold cached HTML that references it.
