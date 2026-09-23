"use client";

import Link from "next/link";

const navItems = [
  { href: "#values", label: "values" },
  { href: "#repositories", label: "repositories" },
  { href: "/blog/", label: "blog" },
];

function NavLink({ href, children }: { href: string; children: React.ReactNode }) {
  const isAnchor = href.startsWith("#");
  const style = {
    color: "var(--bp-dim)",
    transition: "color 0.2s",
    minHeight: 44,
    padding: "12px 8px",
    display: "inline-flex",
    alignItems: "center",
  };

  if (isAnchor) {
    return (
      <a
        href={href}
        style={style}
        onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
        onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
      >
        {children}
      </a>
    );
  }

  return (
    <Link
      href={href}
      style={style}
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
          fontSize: 22,
          lineHeight: "32px",
          color: "var(--bp-heading)",
          maxWidth: 520,
          marginBottom: 12,
        }}
      >
        We build cool stuff.
      </p>

      <p
        style={{
          fontSize: 15,
          lineHeight: "24px",
          color: "var(--bp-dim)",
          maxWidth: 560,
        }}
      >
        Small experiments, useful tools, strange systems, and the occasional big idea.
      </p>

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
