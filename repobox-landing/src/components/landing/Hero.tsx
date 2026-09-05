"use client";

import Link from "next/link";

const navItems = [
  { href: "/projects", label: "projects" },
  { href: "/portfolio", label: "portfolio" },
  { href: "/proof", label: "proof" },
  // /agents and /packages quarantined from public nav by 2026-08-17 disposition.
  { href: "/blog/", label: "blog" },
  { href: "/hire", label: "hire us" },
];

function NavLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <Link
      href={href}
      style={{
        color: "var(--bp-dim)",
        transition: "color 0.2s",
        minHeight: 44,
        padding: "12px 8px",
        display: "inline-flex",
        alignItems: "center",
      }}
      onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
      onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
    >
      {children}
    </Link>
  );
}

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
          flexWrap: "wrap",
        }}
      >
        {navItems.map((item) => (
          <NavLink href={item.href} key={item.href}>
            {item.label}
          </NavLink>
        ))}
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
          maxWidth: 520,
          marginBottom: 32,
        }}
      >
        An independent team building infrastructure for software that runs
        itself.
      </p>

      <p
        style={{
          fontSize: 15,
          lineHeight: "24px",
          color: "#8eafc4",
          maxWidth: 580,
        }}
      >
        Right now that means a control plane for long-running agent work, an
        on-chain order book, a landing-page agent that is deliberately powerless,
        and a permission layer that makes a git repo safe to hand to a machine.
        Everything below is running somewhere you can click.
      </p>

      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginTop: 32 }}>
        <a
          href="#work"
          style={{
            border: "1px solid var(--bp-border)",
            borderRadius: 8,
            color: "var(--bp-heading)",
            padding: "12px 18px",
            textDecoration: "none",
            background: "rgba(6, 14, 26, 0.72)",
            fontSize: 13,
          }}
        >
          see the work
        </a>
        <Link
          href="/hire"
          style={{
            border: "1px solid var(--bp-border)",
            borderRadius: 8,
            color: "var(--bp-heading)",
            padding: "12px 18px",
            textDecoration: "none",
            background: "rgba(6, 14, 26, 0.72)",
            fontSize: 13,
          }}
        >
          work with us
        </Link>
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
