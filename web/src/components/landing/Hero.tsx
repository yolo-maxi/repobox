"use client";

import Link from "next/link";

const navItems = [
  { href: "/explore", label: "explore" },
  { href: "/projects", label: "projects" },
  { href: "/portfolio", label: "portfolio" },
  { href: "/proof", label: "proof" },
  { href: "/agents", label: "agents" },
  { href: "/packages", label: "packages" },
  { href: "/git", label: "git" },
  { href: "/blog/", label: "blog" },
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
          color: "var(--bp-heading)",
          maxWidth: 560,
          marginBottom: 16,
          fontWeight: 600,
        }}
      >
        An independent team building cool ideas with cool people.
      </p>

      <p
        style={{
          fontSize: 15,
          lineHeight: "24px",
          color: "#a0c4d8",
          maxWidth: 600,
          marginBottom: 40,
        }}
      >
        We ship sharp little products, agent-powered systems, and weird useful infrastructure.
        No pitch-deck theatre — just code, taste, receipts, and curiosity.
      </p>

      <div style={{ display: "flex", alignItems: "center", gap: 12, marginTop: 16, flexWrap: "wrap" }}>
        <Link
          href="/explore"
          style={{
            display: "inline-flex",
            alignItems: "center",
            fontFamily: "var(--font-mono), monospace",
            fontSize: 14,
            color: "var(--bp-accent)",
            textDecoration: "none",
            fontWeight: 600,
            background: "rgba(79,195,247,0.15)",
            border: "1px solid rgba(79,195,247,0.4)",
            borderRadius: 6,
            padding: "10px 22px",
            transition: "background 0.2s",
          }}
        >
          See what we are building
        </Link>
        <Link href="/git" style={{ fontSize: 13, color: "var(--bp-accent)", fontFamily: "var(--font-mono), monospace", textDecoration: "none" }}>
          git permission layer →
        </Link>
      </div>

      <div
        style={{
          marginTop: 40,
          color: "#3a5570",
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
