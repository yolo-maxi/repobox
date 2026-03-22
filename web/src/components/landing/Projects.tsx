"use client";

import { useEffect, useRef } from "react";

interface FeatureItem {
  icon: string;
  label: string;
  detail: string;
}

const AGENT_FEATURES: FeatureItem[] = [
  { icon: "🔑", label: "Per-agent keys", detail: "Every agent gets its own wallet. Every commit signed." },
  { icon: "🛡️", label: "Git shim", detail: "Intercepts push, merge, edit before anything lands." },
  { icon: "📄", label: "Config as code", detail: "One YAML file. Permissions travel with the repo." },
  { icon: "🔒", label: "Default deny", detail: "Agents only get what you explicitly allow." },
];

const ONCHAIN_FEATURES: FeatureItem[] = [
  { icon: "🏷️", label: "ENS identity", detail: "Use ENS names instead of raw addresses." },
  { icon: "🪙", label: "Token-gated access", detail: "Use any onchain state for repo permissions." },
  { icon: "🐛", label: "Bug bounties", detail: "ERC-8183 escrow for agent bug reports." },
  { icon: "🌐", label: "x402 paid reads", detail: "Monetize repo access via HTTP 402." },
];

function FeatureList({ items }: { items: FeatureItem[] }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {items.map((f) => (
        <div key={f.label} style={{ display: "flex", gap: 10, alignItems: "baseline" }}>
          <span style={{ fontSize: 14, flexShrink: 0 }}>{f.icon}</span>
          <span>
            <span style={{ color: "#ffffff", fontWeight: 600, fontSize: 14 }}>{f.label}</span>
            <span style={{ color: "var(--bp-dim)", fontSize: 13 }}> — {f.detail}</span>
          </span>
        </div>
      ))}
    </div>
  );
}

export function LandingProjects() {
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
      <div style={{ 
        marginBottom: 48, 
        background: "rgba(0, 0, 0, 0.2)",
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
        padding: "24px 28px"
      }}>
        <h2
          style={{
            fontSize: 14,
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent)",
            fontWeight: 500,
            marginBottom: 14,
          }}
        >
          🤖 For AI Agents
        </h2>
        <FeatureList items={AGENT_FEATURES} />
      </div>

      <div style={{
        background: "rgba(0, 0, 0, 0.2)",
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
        padding: "24px 28px"
      }}>
        <h2
          style={{
            fontSize: 14,
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent2)",
            fontWeight: 500,
            marginBottom: 14,
          }}
        >
          ⛓️ On-Chain Integrations
        </h2>
        <FeatureList items={ONCHAIN_FEATURES} />
      </div>
    </section>
  );
}
