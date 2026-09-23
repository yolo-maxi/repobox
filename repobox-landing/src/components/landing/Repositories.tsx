import { sectionTitleStyle } from "./shared";

const repositories = [
  {
    name: "repo.box",
    href: "https://github.com/yolo-maxi/repobox",
    description: "Tools for giving software agents bounded, auditable access to Git repositories.",
  },
  {
    name: "Concierge",
    href: "https://github.com/yolo-maxi/concierge",
    description: "A deliberately constrained landing-page agent that answers from a page brief and nothing more.",
  },
  {
    name: "Frontier",
    href: "https://github.com/yolo-maxi/frontier-orderbook",
    description: "An onchain central-limit order book with basis-point ticks and whole-ladder fills.",
  },
  {
    name: "Oceangram",
    href: "https://github.com/yolo-maxi/oceangram",
    description: "A Telegram and AI-agent cockpit for VS Code and Cursor.",
  },
];

export function LandingRepositories() {
  return (
    <section id="repositories" style={{ marginBottom: 24 }}>
      <h2 style={sectionTitleStyle}>Public repositories</h2>
      <div style={{ display: "grid", gap: 12 }}>
        {repositories.map((repository) => (
          <a
            key={repository.name}
            href={repository.href}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: "20px 22px",
              color: "inherit",
              textDecoration: "none",
            }}
          >
            <h3
              style={{
                fontSize: 16,
                lineHeight: "24px",
                color: "var(--bp-accent2)",
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              {repository.name} ↗
            </h3>
            <p style={{ fontSize: 14, lineHeight: "22px", color: "var(--bp-dim)" }}>
              {repository.description}
            </p>
          </a>
        ))}
      </div>
    </section>
  );
}
