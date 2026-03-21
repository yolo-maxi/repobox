"use client";

import { useEffect, useRef } from "react";

interface FeatureItem {
  icon: string;
  label: string;
  detail: string;
}

const AGENT_FEATURES: FeatureItem[] = [
  { icon: "🔑", label: "Per-agent keys", detail: "Every agent gets a secp256k1 wallet. Every commit is signed." },
  { icon: "🛡️", label: "Git shim", detail: "Intercepts push, merge, edit. Rules enforced before anything lands." },
  { icon: "📄", label: "Config as code", detail: "One YAML file in your repo. Permissions travel with the code." },
  { icon: "🔒", label: "Default deny", detail: "Agents only get what you explicitly allow. No implicit trust." },
];

const ONCHAIN_FEATURES: FeatureItem[] = [
  { icon: "🏷️", label: "ENS identity", detail: "Use ENS names instead of raw addresses. On-chain resolution." },
  { icon: "🪙", label: "Token-gated access", detail: "Gate repo access by NFT or ERC-20 ownership." },
  { icon: "💸", label: "Streaming payments", detail: "Pay contributors via Superfluid streams proportional to commits." },
  { icon: "🐛", label: "Bug bounties", detail: "ERC-8183 escrow. Agents report bugs, evaluator pays out." },
  { icon: "🌐", label: "x402 paid reads", detail: "Monetize repo access. Pay-per-clone via HTTP 402." },
];

function FeatureList({ items }: { items: FeatureItem[] }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {items.map((f) => (
        <div key={f.label} style={{ display: "flex", gap: 12, alignItems: "flex-start" }}>
          <span style={{ fontSize: 16, lineHeight: "22px", flexShrink: 0 }}>{f.icon}</span>
          <div>
            <span style={{ color: "#ffffff", fontWeight: 600, fontSize: 13 }}>{f.label}</span>
            <span style={{ color: "var(--bp-dim)", fontSize: 12 }}> — {f.detail}</span>
          </div>
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
      <div style={{ marginBottom: 32 }}>
        <h2
          style={{
            fontSize: 12,
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent)",
            fontWeight: 500,
            marginBottom: 16,
          }}
        >
          🤖 For AI Agents
        </h2>
        <FeatureList items={AGENT_FEATURES} />
      </div>

      <div>
        <h2
          style={{
            fontSize: 12,
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent2)",
            fontWeight: 500,
            marginBottom: 16,
          }}
        >
          ⛓️ On-Chain Integrations
        </h2>
        <FeatureList items={ONCHAIN_FEATURES} />
      </div>
    </section>
  );
}
