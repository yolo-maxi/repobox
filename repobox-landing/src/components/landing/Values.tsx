import { sectionTitleStyle } from "./shared";

const values = [
  {
    name: "Privacy",
    description: "People should understand what a tool knows about them and keep control of it.",
  },
  {
    name: "Self Sovereignty",
    description: "Own your tools, your data, and the choices that shape your work.",
  },
  {
    name: "Freedom",
    description: "Build things that expand what people can do, rather than narrow it.",
  },
  {
    name: "Open Source",
    description: "Share the parts worth learning from, and make systems easier to inspect and improve.",
  },
  {
    name: "Human Flourishing",
    description: "Technology should leave people with more agency, curiosity, and room to grow.",
  },
];

export function LandingValues() {
  return (
    <section id="values" style={{ marginBottom: 72 }}>
      <h2 style={sectionTitleStyle}>What we care about</h2>
      <div style={{ display: "grid", gap: 12 }}>
        {values.map((value) => (
          <article
            key={value.name}
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: "20px 22px",
            }}
          >
            <h3
              style={{
                fontSize: 16,
                lineHeight: "24px",
                color: "var(--bp-heading)",
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              {value.name}
            </h3>
            <p style={{ fontSize: 14, lineHeight: "22px", color: "var(--bp-dim)" }}>
              {value.description}
            </p>
          </article>
        ))}
      </div>
    </section>
  );
}
