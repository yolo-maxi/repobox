import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Self-contained server bundle: built on Hetzner, shipped to the repo.box VPS.
  // Keeps builds off repo.box and avoids rsyncing the full node_modules tree.
  output: "standalone",
};

export default nextConfig;
