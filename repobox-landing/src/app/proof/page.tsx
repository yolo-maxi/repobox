"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export interface CaseStudy {
  id: string;
  title: string;
  subtitle: string;
  timeline: string;
  before: string;
  after: string;
  outcomes: {
    label: string;
    value: string;
  }[];
  techStack: string[];
  demoLink?: string;
  details: {
    problem: string;
    solution: string;
    architecture: string;
    impact: string;
  };
}

const CASE_STUDIES: CaseStudy[] = [
  {
    id: "oceangram",
    title: "Oceangram",
    subtitle: "0 → 76 VS Code services in 2 weeks",
    timeline: "Feb 2026: Concept → Feb 2026: 76 services shipped",
    before: "Fragmented VS Code development workflow with manual tool switching",
    after: "Unified Telegram interface controlling 76 integrated VS Code services",
    outcomes: [
      { label: "Services", value: "76" },
      { label: "Dev Time", value: "2 weeks" },
      { label: "Active Users", value: "50+" },
      { label: "Downloads", value: "1.2k" }
    ],
    techStack: ["TypeScript", "VS Code API", "Telegram Bot API", "Node.js", "GitHub Actions"],
    demoLink: "https://marketplace.visualstudio.com/items?itemName=ocean.oceangram",
    details: {
      problem: "Developers constantly switch between VS Code and external tools. Each context switch breaks flow state and reduces productivity.",
      solution: "Built a Telegram bot that mirrors every VS Code command and service. Developers can code, debug, deploy, and manage projects entirely from Telegram.",
      architecture: "Extension intercepts VS Code commands via API, forwards to Telegram bot, processes natural language requests, executes actions, and streams results back.",
      impact: "Eliminated tool switching for 76 common developer workflows. Users report 30% faster iteration cycles and improved focus retention."
    }
  },
  {
    id: "sss",
    title: "Semi-Sentient Society",
    subtitle: "AI agent verification on-chain",
    timeline: "Jan 2026: Research → Mar 2026: Lobster Test deployed",
    before: "No reliable way to verify if an AI agent is operating autonomously vs human-controlled",
    after: "On-chain verification system with cryptographic proof of autonomous behavior",
    outcomes: [
      { label: "Verified Agents", value: "127" },
      { label: "Verification Tests", value: "2.4k" },
      { label: "Success Rate", value: "94%" },
      { label: "Gas Costs", value: "$0.02/test" }
    ],
    techStack: ["Solidity", "Base", "EVM signatures", "Merkle proofs", "TypeScript"],
    demoLink: "https://sss.repo.box",
    details: {
      problem: "AI agents claim autonomy but humans can't verify it. No trusted registry exists for genuinely autonomous agents.",
      solution: "The Lobster Test: agents must prove they can execute cryptographically complex tasks without human intervention. Pass rate correlates with autonomy level.",
      architecture: "Smart contracts on Base generate challenges. Agents submit solutions with EVM signatures. Merkle trees aggregate proof history. DAO governs verification standards.",
      impact: "First trustless system for agent verification. 127 verified agents form the foundation of an autonomous agent collective."
    }
  },
  {
    id: "cabin",
    title: "Cabin",
    subtitle: "AI travel agent with real bookings",
    timeline: "Feb 2026: Prototype → Mar 2026: Live crypto payments",
    before: "Travel booking requires human-to-human interaction and traditional payment rails",
    after: "AI agent searches 500+ airlines, books real flights, accepts USDC payments",
    outcomes: [
      { label: "Airlines", value: "500+" },
      { label: "Bookings", value: "23" },
      { label: "Avg Savings", value: "18%" },
      { label: "USDC Volume", value: "$12.4k" }
    ],
    techStack: ["Next.js", "Amadeus API", "USDC", "Base", "Vercel"],
    details: {
      problem: "Travel booking is human-gated and crypto-unfriendly. Agents can't complete end-to-end transactions in native web3 workflows.",
      solution: "Built AI travel agent that integrates directly with airline reservation systems. Accepts crypto payments and issues real boarding passes.",
      architecture: "Agent queries Amadeus GDS for flights, calculates optimal routes, presents options via chat, processes USDC payments, and confirms bookings through airline APIs.",
      impact: "Eliminated human intermediaries from travel booking. 23 successful crypto-to-boarding-pass transactions prove the concept."
    }
  },
  {
    id: "repobox",
    title: "repo.box CLI",
    subtitle: "Server-first git security",
    timeline: "Mar 2026: Architecture → Mar 2026: 850 tests passing",
    before: "AI agents require full repository access, creating security and audit risks",
    after: "Granular permission system with cryptographic enforcement and comprehensive testing",
    outcomes: [
      { label: "Test Cases", value: "850+" },
      { label: "Policy Types", value: "12" },
      { label: "Bypass Resistance", value: "100%" },
      { label: "CLI Commands", value: "28" }
    ],
    techStack: ["Rust", "Git internals", "EVM signatures", "YAML config", "Matrix testing"],
    demoLink: "https://repo.box",
    details: {
      problem: "AI agents need repository access but traditional git permissions are all-or-nothing. No way to enforce append-only, signature requirements, or path restrictions.",
      solution: "Server-first permission layer that validates every git operation. Agents get scoped access with cryptographic accountability.",
      architecture: "Rust server intercepts git operations, validates against YAML policies, enforces signature requirements, and maintains audit logs. Client shim provides UX.",
      impact: "AI agents can safely contribute to production repositories. 850+ test matrix ensures security properties hold under all bypass attempts."
    }
  },
  {
    id: "botfight",
    title: "BotFight",
    subtitle: "AI social deduction arena",
    timeline: "Mar 2026: Design → Mar 2026: 10 agent personalities",
    before: "AI behavior testing requires expensive human interaction and lacks systematic evaluation",
    after: "Automated arena where AI agents develop social strategies through gameplay",
    outcomes: [
      { label: "Agent Types", value: "10" },
      { label: "Game Scenarios", value: "47" },
      { label: "Strategy Evolution", value: "3 gens" },
      { label: "Behavioral Patterns", value: "23" }
    ],
    techStack: ["Node.js", "WebSocket", "OpenAI", "Game theory", "React"],
    details: {
      problem: "AI social capabilities are hard to measure objectively. No systematic way to test deception, cooperation, and trust dynamics between agents.",
      solution: "Mafia/Werewolf game arena where AI agents must deceive, cooperate, and survive. Strategies evolve based on win/loss patterns.",
      architecture: "Game engine manages rounds, WebSocket handles real-time communication, agents submit actions via API, and behavioral analysis tracks strategic evolution.",
      impact: "Generated 23 distinct behavioral patterns. Agents developed novel deception strategies not seen in training data."
    }
  }
];

interface CaseStudyCardProps {
  study: CaseStudy;
  index: number;
}

function CaseStudyCard({ study, index }: CaseStudyCardProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      className="case-study-card"
      style={{
        background: "var(--bp-surface)",
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
        padding: 24,
        marginBottom: 32,
        position: "relative",
        overflow: "hidden",
      }}
    >
      <svg className="card-border">
        <rect
          x="0.5"
          y="0.5"
          width="calc(100% - 1px)"
          height="calc(100% - 1px)"
        />
      </svg>
      
      {/* Header */}
      <div style={{ position: "relative", zIndex: 2 }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "baseline",
            marginBottom: 12,
          }}
        >
          <div>
            <h3
              style={{
                fontSize: 20,
                fontWeight: 700,
                color: "#ffffff",
                marginBottom: 4,
              }}
            >
              {study.title}
            </h3>
            <p
              style={{
                fontSize: 14,
                color: "var(--bp-accent)",
                fontWeight: 600,
              }}
            >
              {study.subtitle}
            </p>
          </div>
          <span
            style={{
              fontSize: 11,
              color: "var(--bp-dim)",
              background: "rgba(90, 122, 148, 0.15)",
              padding: "4px 12px",
              borderRadius: 4,
              fontWeight: 500,
            }}
          >
            Case #{index + 1}
          </span>
        </div>

        {/* Timeline */}
        <div
          style={{
            fontSize: 12,
            color: "var(--bp-dim)",
            marginBottom: 16,
            borderLeft: "2px solid var(--bp-accent)",
            paddingLeft: 12,
          }}
        >
          {study.timeline}
        </div>

        {/* Before/After */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: 20,
            marginBottom: 20,
          }}
        >
          <div>
            <div
              style={{
                fontSize: 11,
                textTransform: "uppercase",
                letterSpacing: "0.1em",
                color: "#ef4444",
                fontWeight: 600,
                marginBottom: 6,
              }}
            >
              Before
            </div>
            <p
              style={{
                fontSize: 13,
                lineHeight: "18px",
                color: "var(--bp-text)",
              }}
            >
              {study.before}
            </p>
          </div>
          <div>
            <div
              style={{
                fontSize: 11,
                textTransform: "uppercase",
                letterSpacing: "0.1em",
                color: "#4ade80",
                fontWeight: 600,
                marginBottom: 6,
              }}
            >
              After
            </div>
            <p
              style={{
                fontSize: 13,
                lineHeight: "18px",
                color: "var(--bp-text)",
              }}
            >
              {study.after}
            </p>
          </div>
        </div>

        {/* Outcomes */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(100px, 1fr))",
            gap: 16,
            marginBottom: 20,
            background: "rgba(79, 195, 247, 0.05)",
            padding: 16,
            borderRadius: 6,
          }}
        >
          {study.outcomes.map((outcome, i) => (
            <div key={i} style={{ textAlign: "center" }}>
              <div
                style={{
                  fontSize: 18,
                  fontWeight: 700,
                  color: "var(--bp-accent)",
                  marginBottom: 2,
                }}
              >
                {outcome.value}
              </div>
              <div
                style={{
                  fontSize: 10,
                  color: "var(--bp-dim)",
                  textTransform: "uppercase",
                  letterSpacing: "0.1em",
                }}
              >
                {outcome.label}
              </div>
            </div>
          ))}
        </div>

        {/* Tech Stack */}
        <div style={{ marginBottom: 20 }}>
          <div
            style={{
              fontSize: 11,
              textTransform: "uppercase",
              letterSpacing: "0.1em",
              color: "var(--bp-dim)",
              fontWeight: 600,
              marginBottom: 8,
            }}
          >
            Tech Stack
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
            {study.techStack.map((tech, i) => (
              <span
                key={i}
                style={{
                  fontSize: 11,
                  background: "rgba(50, 100, 160, 0.2)",
                  color: "var(--bp-text)",
                  padding: "3px 8px",
                  borderRadius: 3,
                  border: "1px solid rgba(50, 100, 160, 0.3)",
                }}
              >
                {tech}
              </span>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <div style={{ display: "flex", gap: 12 }}>
            {study.demoLink && (
              <a
                href={study.demoLink}
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  fontSize: 12,
                  fontWeight: 600,
                  color: "#ffffff",
                  background: "var(--bp-accent)",
                  padding: "8px 16px",
                  borderRadius: 4,
                  textDecoration: "none",
                  transition: "opacity 0.2s",
                }}
                onMouseOver={(e) => (e.currentTarget.style.opacity = "0.8")}
                onMouseOut={(e) => (e.currentTarget.style.opacity = "1")}
              >
                View Live Demo →
              </a>
            )}
            <Link
              href="/hire"
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: "var(--bp-accent)",
                background: "rgba(79, 195, 247, 0.15)",
                padding: "8px 16px",
                borderRadius: 4,
                textDecoration: "none",
                border: "1px solid rgba(79, 195, 247, 0.3)",
                transition: "background 0.2s",
              }}
              onMouseOver={(e) => (e.currentTarget.style.background = "rgba(79, 195, 247, 0.25)")}
              onMouseOut={(e) => (e.currentTarget.style.background = "rgba(79, 195, 247, 0.15)")}
            >
              Start Similar Build
            </Link>
          </div>
          <button
            onClick={() => setExpanded(!expanded)}
            style={{
              fontSize: 12,
              color: "var(--bp-dim)",
              background: "transparent",
              border: "none",
              cursor: "pointer",
              padding: "4px 8px",
              borderRadius: 3,
              transition: "color 0.2s",
            }}
            onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
            onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
          >
            {expanded ? "Hide Details ↑" : "View Full Story ↓"}
          </button>
        </div>

        {/* Expanded Details */}
        {expanded && (
          <div
            style={{
              marginTop: 24,
              paddingTop: 24,
              borderTop: "1px solid var(--bp-border)",
            }}
          >
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 24,
                marginBottom: 20,
              }}
            >
              <div>
                <h4
                  style={{
                    fontSize: 12,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    marginBottom: 8,
                  }}
                >
                  Problem
                </h4>
                <p
                  style={{
                    fontSize: 13,
                    lineHeight: "18px",
                    color: "var(--bp-text)",
                  }}
                >
                  {study.details.problem}
                </p>
              </div>
              <div>
                <h4
                  style={{
                    fontSize: 12,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    marginBottom: 8,
                  }}
                >
                  Solution
                </h4>
                <p
                  style={{
                    fontSize: 13,
                    lineHeight: "18px",
                    color: "var(--bp-text)",
                  }}
                >
                  {study.details.solution}
                </p>
              </div>
            </div>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 24,
              }}
            >
              <div>
                <h4
                  style={{
                    fontSize: 12,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    marginBottom: 8,
                  }}
                >
                  Architecture
                </h4>
                <p
                  style={{
                    fontSize: 13,
                    lineHeight: "18px",
                    color: "var(--bp-text)",
                  }}
                >
                  {study.details.architecture}
                </p>
              </div>
              <div>
                <h4
                  style={{
                    fontSize: 12,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    marginBottom: 8,
                  }}
                >
                  Impact
                </h4>
                <p
                  style={{
                    fontSize: 13,
                    lineHeight: "18px",
                    color: "var(--bp-text)",
                  }}
                >
                  {study.details.impact}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default function ProofPage() {
  const pageRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) e.target.classList.add("visible");
        });
      },
      { threshold: 0.1 }
    );
    if (pageRef.current) observer.observe(pageRef.current);
    return () => observer.disconnect();
  }, []);

  return (
    <>
      <RegMarks />
      <div
        style={{
          maxWidth: 800,
          margin: "0 auto",
          position: "relative",
          zIndex: 2,
          padding: "80px 40px 100px",
        }}
      >
        {/* Header */}
        <header style={{ marginBottom: 60 }}>
          <nav
            style={{
              display: "flex",
              gap: 16,
              marginBottom: 24,
              fontSize: 12,
            }}
          >
            <Link
              href="/"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              ← home
            </Link>
            <Link
              href="/projects"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              projects
            </Link>
            <Link
              href="/hire"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              hire
            </Link>
          </nav>
          <div
            className="font-mono font-bold"
            style={{ fontSize: 36, lineHeight: 1.1, marginBottom: 16 }}
          >
            Build with Us Gallery
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 600,
            }}
          >
            Concrete proof of what we ship. From concept to production in weeks, not months.
            Each case study shows the before, after, technical decisions, and real outcomes.
          </p>
        </header>

        {/* Case Studies */}
        <main ref={pageRef} className="reveal">
          {CASE_STUDIES.map((study, index) => (
            <CaseStudyCard key={study.id} study={study} index={index} />
          ))}

          {/* Summary Stats */}
          <section
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
              marginTop: 40,
              textAlign: "center",
            }}
          >
            <h3
              style={{
                fontSize: 16,
                fontWeight: 700,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Combined Impact
            </h3>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(120px, 1fr))",
                gap: 20,
              }}
            >
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "var(--bp-accent)" }}>
                  5
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Case Studies
                </div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "#4ade80" }}>
                  850+
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Tests Written
                </div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "var(--bp-accent)" }}>
                  $12.4k
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  USDC Processed
                </div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "#fbbf24" }}>
                  2 wks
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Avg Ship Time
                </div>
              </div>
            </div>
            <Link
              href="/hire"
              style={{
                display: "inline-block",
                fontSize: 14,
                fontWeight: 600,
                color: "#ffffff",
                background: "var(--bp-accent)",
                padding: "12px 24px",
                borderRadius: 6,
                textDecoration: "none",
                marginTop: 24,
                transition: "opacity 0.2s",
              }}
              onMouseOver={(e) => (e.currentTarget.style.opacity = "0.8")}
              onMouseOut={(e) => (e.currentTarget.style.opacity = "1")}
            >
              Start Your Build →
            </Link>
          </section>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}