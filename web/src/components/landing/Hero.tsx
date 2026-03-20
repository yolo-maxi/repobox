"use client";

import Link from "next/link";

export function LandingHero() {
  return (
    <header
      style={{
        marginBottom: 0,
        minHeight: "70vh",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
      }}
    >
      <nav
        style={{
          display: "flex",
          gap: 16,
          marginBottom: 24,
          fontSize: 12,
        }}
      >
        <Link
          href="/docs"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          docs
        </Link>
        <Link
          href="/explore"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          explore
        </Link>
        <Link
          href="/playground"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          playground
        </Link>
        <Link
          href="/blog/"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          blog
        </Link>
      </nav>
      <div
        className="font-mono font-bold"
        style={{ fontSize: 56, lineHeight: 1.1, marginBottom: 24 }}
      >
        repo<span className="logo-dot">.</span>box
      </div>
      <p
        style={{
          fontSize: 18,
          lineHeight: "28px",
          color: "var(--bp-dim)",
          maxWidth: 500,
          marginBottom: 48,
        }}
      >
        Git permission layer that makes repositories safe for AI agents.
      </p>
      <p
        style={{
          fontSize: 15,
          lineHeight: "24px",
          color: "#8eafc4",
          maxWidth: 560,
        }}
      >
        EVM keys, a git shim, and a <code style={{ color: "var(--bp-accent)", background: "rgba(79,195,247,0.08)", padding: "1px 6px", borderRadius: 3 }}>.repobox.yml</code> file
        that stands between your agents and your codebase. One config file. Per-agent permissions. Enforced before anything touches your repo.
      </p>
      <div
        style={{
          marginTop: 60,
          color: "#253a4f",
          fontSize: 11,
          letterSpacing: "0.15em",
          textTransform: "uppercase",
        }}
      >
        ↓ scroll
      </div>
    </header>
  );
}
