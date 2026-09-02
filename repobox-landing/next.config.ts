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
};

export default nextConfig;
