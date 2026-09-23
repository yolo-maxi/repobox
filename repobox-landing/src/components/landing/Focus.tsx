import { sectionTitleStyle } from "./shared";

const focusAreas = [
  {
    label: "Now",
    name: "Synclave",
    href: "https://synclave.net",
    description: "The main focus: making software that can keep meaningful promises, even when its owner has every reason not to.",
  },
  {
    label: "Previously",
    name: "Superfluid",
    href: "https://superfluid.org",
    description: "Fran previously worked on the protocol for streaming money and value in real time.",
  },
];

export function LandingFocus() {
  return (
    <section id="focus" style={{ marginBottom: 72 }}>
      <h2 style={sectionTitleStyle}>What we&apos;re focused on</h2>
      <div style={{ display: "grid", gap: 12 }}>
        {focusAreas.map((area) => (
          <a
            key={area.name}
            href={area.href}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              display: "block",
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: "24px 22px",
              color: "inherit",
              textDecoration: "none",
            }}
          >
            <p
              style={{
                fontSize: 11,
                lineHeight: "18px",
                letterSpacing: "0.12em",
                textTransform: "uppercase",
                color: "var(--bp-dim)",
                marginBottom: 6,
              }}
            >
              {area.label}
            </p>
            <h3
              style={{
                fontSize: 20,
                lineHeight: "28px",
                color: "var(--bp-heading)",
                fontWeight: 600,
                marginBottom: 6,
              }}
            >
              {area.name} ↗
            </h3>
            <p style={{ fontSize: 14, lineHeight: "22px", color: "var(--bp-dim)" }}>
              {area.description}
            </p>
          </a>
        ))}
      </div>
    </section>
  );
}
