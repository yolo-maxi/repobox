"use client";

import Link from "next/link";

const navItems = [
  { href: "/projects", label: "projects" },
  { href: "/portfolio", label: "portfolio" },
  { href: "/proof", label: "proof" },
  { href: "/agents", label: "agents" },
  { href: "/packages", label: "packages" },
  { href: "/git", label: "git" },
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
          marginBottom: 48,
        }}
      >
        An independent team building cool ideas with cool people.
      </p>

      <p
        style={{
          fontSize: 15,
          lineHeight: "24px",
          color: "#8eafc4",
          maxWidth: 580,
        }}
      >
        We ship sharp little products, agent-powered systems, and weird useful
        infrastructure. No pitch-deck theatre — just code, taste, receipts, and curiosity.
      </p>

      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginTop: 32 }}>
        <Link
          href="/portfolio"
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
        </Link>
        <Link
          href="/git"
          style={{
            border: "1px solid rgba(79,195,247,0.35)",
            borderRadius: 8,
            color: "var(--bp-accent)",
            padding: "12px 18px",
            textDecoration: "none",
            background: "rgba(79,195,247,0.08)",
            fontSize: 13,
          }}
        >
          repo.box git layer →
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
