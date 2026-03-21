"use client";

import Link from "next/link";
import { TerminalDemo } from "./TerminalDemo";

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
          href="/explore"
          style={{ color: "var(--bp-dim)", transition: "color 0.2s", minHeight: 44, padding: "12px 8px", display: "inline-flex", alignItems: "center" }}
          onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
          onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
        >
          explorer
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
          color: "var(--bp-heading)",
          maxWidth: 500,
          marginBottom: 16,
          fontWeight: 600,
        }}
      >
        AI agents shouldn&apos;t share your credentials.
      </p>
      <p
        style={{
          fontSize: 15,
          lineHeight: "24px",
          color: "#8eafc4",
          maxWidth: 560,
          marginBottom: 40,
        }}
      >
        Each agent gets its own wallet. Each wallet gets its own rules. All enforced in git.
      </p>

      {/* Terminal Demo */}
      <div style={{ marginBottom: 40 }}>
        <TerminalDemo />
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 12, marginTop: 16, flexWrap: "wrap" }}>
        <Link
          href="/explore"
          style={{
            display: "inline-flex",
            alignItems: "center",
            fontFamily: "var(--font-mono), monospace",
            fontSize: 13,
            color: "var(--bp-accent)",
            textDecoration: "none",
            fontWeight: 600,
            background: "rgba(79,195,247,0.1)",
            border: "1px solid rgba(79,195,247,0.25)",
            borderRadius: 6,
            padding: "8px 18px",
            transition: "background 0.2s",
          }}
        >
          Join the Wall
        </Link>
        <span style={{ fontSize: 13, color: "var(--bp-dim)", fontFamily: "var(--font-mono), monospace" }}>
          the first repo where anyone can push to main →
        </span>
      </div>

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
