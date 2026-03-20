"use client";

import { useEffect, useRef } from "react";

const CONFIG_YAML = `# .repobox/config.yml — protect main
groups:
  founders: [alice]
  agents:   [codex-bot]

rules:
  - group: founders
    allow: ["*"]

  - group: agents
    allow: ["feature/**"]
    deny:  ["main", ".repobox/*"]`;

export function ConfigExample() {
  const sectionRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) e.target.classList.add("visible");
        });
      },
      { threshold: 0.1 }
    );
    if (sectionRef.current) observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 60 }}>
      <div
        style={{
          background: "rgba(6, 14, 26, 0.85)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "32px 36px",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
        }}
      >
        <h2
          style={{
            fontSize: 12,
            lineHeight: "20px",
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent)",
            fontWeight: 500,
            marginBottom: 20,
          }}
        >
          Minimal Config
        </h2>
        <pre
          style={{
            background: "rgba(0, 0, 0, 0.5)",
            border: "1px solid var(--bp-border)",
            borderRadius: 6,
            padding: 20,
            overflow: "auto",
            fontFamily: "var(--font-mono), monospace",
            fontSize: 12,
            lineHeight: "20px",
            color: "var(--bp-text)",
            margin: 0,
          }}
        >
          <code>{CONFIG_YAML}</code>
        </pre>
        <p
          style={{
            fontSize: 12,
            lineHeight: "20px",
            color: "var(--bp-dim)",
            marginTop: 16,
          }}
        >
          Drop this in your repo. Agents get feature branches, founders keep full control.
        </p>
      </div>
    </section>
  );
}
