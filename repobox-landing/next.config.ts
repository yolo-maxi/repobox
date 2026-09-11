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
  // /projects is the one curated list of current work (2026-09-11). The
  // routes below were overlapping views of the same data, or detail pages for
  // work that has since been retired; they redirect permanently rather than
  // 404 so old links and search results still land somewhere useful.
  async redirects() {
    return [
      { source: "/portfolio", destination: "/projects", permanent: true },
      { source: "/building", destination: "/projects", permanent: true },
      { source: "/repos", destination: "/projects", permanent: true },
      { source: "/repos/:path*", destination: "/projects", permanent: true },
      { source: "/projects/:slug", destination: "/projects", permanent: true },
    ];
  },
};

export default nextConfig;
