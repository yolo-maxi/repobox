"use client";

import { useState } from "react";
import Link from "next/link";

export function LandingHero() {
  const [copied, setCopied] = useState(false);
  const cmd = "curl -sSf https://repo.box/install.sh | sh";

  const handleCopy = () => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

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
          gap: 8,
          marginBottom: 24,
          fontSize: 12,
        }}
      >
        <Link
          href="/docs"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s", minHeight: 44, padding: "12px 8px", display: "inline-flex", alignItems: "center" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          docs
        </Link>
        <Link
          href="/playground"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s", minHeight: 44, padding: "12px 8px", display: "inline-flex", alignItems: "center" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          playground
        </Link>
        <Link
          href="/blog/"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s", minHeight: 44, padding: "12px 8px", display: "inline-flex", alignItems: "center" }}
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
        EVM keys, a git shim, and a <code style={{ color: "var(--bp-accent)", background: "rgba(79,195,247,0.08)", padding: "1px 6px", borderRadius: 3 }}>.repobox/config.yml</code> file
        that stands between your agents and your codebase. One config file. Per-agent permissions. Enforced before anything touches your repo.
      </p>
      {/* Inline CTA */}
      <div
        onClick={handleCopy}
        style={{
          marginTop: 32,
          background: "rgba(0, 0, 0, 0.5)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "14px 20px",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 16,
          cursor: "pointer",
          transition: "border-color 0.2s",
          maxWidth: 520,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 12, overflow: "hidden" }}>
          <span style={{ color: "var(--bp-accent)", fontWeight: 600, flexShrink: 0, fontSize: 15 }}>$</span>
          <code
            style={{
              fontSize: 14,
              lineHeight: "22px",
              color: "var(--bp-heading)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
              fontFamily: "var(--font-mono), monospace",
              fontWeight: 500,
            }}
          >
            {cmd}
          </code>
        </div>
        <span
          style={{
            color: copied ? "var(--bp-accent)" : "var(--bp-dim)",
            fontFamily: "var(--font-mono), monospace",
            fontSize: 11,
            flexShrink: 0,
            transition: "color 0.2s",
            border: `1px solid ${copied ? "var(--bp-accent)" : "var(--bp-border)"}`,
            padding: "4px 12px",
            borderRadius: 4,
          }}
        >
          {copied ? "copied!" : "copy"}
        </span>
      </div>
      <Link
        href="/playground"
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 6,
          marginTop: 16,
          fontFamily: "var(--font-mono), monospace",
          fontSize: 13,
          color: "var(--bp-accent)",
          textDecoration: "none",
          transition: "opacity 0.2s",
        }}
      >
        <span style={{ fontSize: 15 }}>▶</span> Try in Playground
      </Link>

      <div
        style={{
          marginTop: 40,
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
