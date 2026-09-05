import type { NextConfig } from "next";

const assetPrefix =
  process.env.REPOBOX_ASSET_PREFIX ?? "/_next-static-sunset-git-20260817";

const nextConfig: NextConfig = {
  // Self-contained server bundle: built on Hetzner, shipped to the repo.box VPS.
  // Keeps builds off repo.box and avoids rsyncing the full node_modules tree.
  output: "standalone",
  productionBrowserSourceMaps: false,
  // Turbopack chunk names can stay stable across tiny copy-only edits. Prefix
  // static assets for this deploy so browsers do not reuse old immutable JS.
  assetPrefix,
  // The blog index is a static file at public/blog/index.html, so Next only
  // serves it at that exact path and the homepage nav link to /blog 404s.
  // Rewrite (not redirect) so the canonical /blog URL renders the index.
  async rewrites() {
    return [{ source: "/blog", destination: "/blog/index.html" }];
  },
};

export default nextConfig;
