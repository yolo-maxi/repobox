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
  /** Set when the project is no longer running. Explains what happened. */
  retiredNote?: string;
  details: {
    problem: string;
    solution: string;
    architecture: string;
    impact: string;
  };
}

// PROVENANCE RULE (REPOBOX-PROOF-001, 2026-09-06).
//
// This page previously carried specific figures — "127 verified agents",
// "2.4k verification tests", "94% success rate", "23 bookings", "$12.4k USDC
// volume", "18% avg savings", "850+ test cases", "1.2k downloads", "50+ active
// users", "30% faster iteration" — none of which could be traced to any
// source available on this host. They have been removed rather than restated,
// because a proof page whose numbers cannot be checked is worse than one with
// no numbers.
//
// A figure may only appear in `outcomes` if it can be recomputed from
// something a reader could in principle be shown. Counts of things that exist
// in a repository qualify; usage, revenue and success-rate claims do not
// unless there is a published source. When in doubt, describe the work
// instead of scoring it.
//
// Demo links were swept live on 2026-09-06; dead hosts are recorded in
// `retiredNote` and the link is removed rather than left to 404.
const CASE_STUDIES: CaseStudy[] = [
  {
    id: "oceangram",
    title: "Oceangram",
    subtitle: "A Telegram control surface for VS Code",
    timeline: "Feb 2026: concept → Feb 2026: shipped to the VS Code Marketplace",
    before: "Fragmented VS Code development workflow with manual tool switching",
    after: "A Telegram interface that mirrors VS Code commands and services",
    outcomes: [
      { label: "Status", value: "retired" },
      { label: "Built in", value: "Feb 2026" }
    ],
    techStack: ["TypeScript", "VS Code API", "Telegram Bot API", "Node.js", "GitHub Actions"],
    retiredNote:
      "The Marketplace listing returns 404 as of 2026-09-06 and no source checkout remains on the build host, so nothing here can be demonstrated. Adoption figures previously shown on this page had no traceable source and were removed.",
    details: {
      problem: "Developers constantly switch between VS Code and external tools. Each context switch breaks flow state and reduces productivity.",
      solution: "Built a Telegram bot that mirrors VS Code commands and services, so a developer can drive the editor from a phone.",
      architecture: "Extension intercepts VS Code commands via API, forwards to Telegram bot, processes natural language requests, executes actions, and streams results back.",
      impact: "Proved that an editor can be driven end-to-end from a chat surface. The project is no longer published and no usage data survives, so no adoption claim is made here."
    }
  },
  {
    id: "sss",
    title: "Semi-Sentient Society",
    subtitle: "An experiment in verifying agent autonomy on-chain",
    timeline: "Jan 2026: research → Mar 2026: Lobster Test deployed",
    before: "No reliable way to verify if an AI agent is operating autonomously vs human-controlled",
    after: "An on-chain challenge/response scheme intended to evidence autonomous behaviour",
    outcomes: [
      { label: "Status", value: "retired" },
      { label: "Chain", value: "Base" }
    ],
    techStack: ["Solidity", "Base", "EVM signatures", "Merkle proofs", "TypeScript"],
    retiredNote:
      "sss.repo.box has stopped responding (no HTTP response on 2026-09-06) and no source checkout remains on the build host. Agent counts, test counts and pass rates previously shown here could not be traced to any surviving source and were removed.",
    details: {
      problem: "AI agents claim autonomy but humans can't verify it. No trusted registry exists for genuinely autonomous agents.",
      solution: "The Lobster Test: agents were asked to execute cryptographically awkward tasks without human intervention, on the theory that pass rate correlates with autonomy.",
      architecture: "Smart contracts on Base generate challenges. Agents submit solutions with EVM signatures. Merkle trees aggregate proof history. A DAO governs verification standards.",
      impact: "An early attempt at trustless agent verification. The deployment is gone and its results are not recoverable from this host, so no outcome figures are claimed."
    }
  },
  {
    id: "cabin",
    title: "Cabin",
    subtitle: "A travel agent that could settle in crypto",
    timeline: "Feb 2026: prototype → Apr 2026: last commit",
    before: "Travel booking requires human-to-human interaction and traditional payment rails",
    after: "An agent that searched flights through a GDS and could settle in USDC",
    outcomes: [
      { label: "Status", value: "retired" },
      { label: "Last commit", value: "Apr 2026" }
    ],
    techStack: ["Next.js", "Amadeus API", "USDC", "Base", "Vercel"],
    retiredNote:
      "cabin.ai is no longer ours: as of 2026-09-06 it resolves to a Spaceship.com domain-for-sale page. Booking counts, savings percentages and USDC volume previously shown here had no traceable source and were removed.",
    details: {
      problem: "Travel booking is human-gated and crypto-unfriendly. Agents can't complete end-to-end transactions in native web3 workflows.",
      solution: "Built a travel agent that queried airline reservation systems directly and accepted crypto payment for the booking.",
      architecture: "Agent queries Amadeus GDS for flights, calculates routes, presents options via chat, processes USDC payments, and confirms bookings through airline APIs.",
      impact: "Demonstrated an agent completing a real-world purchase end to end. The service is retired and its transaction records are not available here, so no volume is claimed."
    }
  },
  {
    id: "repobox",
    title: "repo.box CLI",
    subtitle: "Server-first git permissions for agents",
    timeline: "Mar 2026: architecture → still in active development",
    before: "AI agents require full repository access, creating security and audit risks",
    after: "Granular permission system with cryptographic enforcement and a large test matrix",
    outcomes: [
      { label: "Status", value: "in development" },
      { label: "Language", value: "Rust" }
    ],
    techStack: ["Rust", "Git internals", "EVM signatures", "YAML config", "Matrix testing"],
    demoLink: "https://repo.box",
    details: {
      problem: "AI agents need repository access but traditional git permissions are all-or-nothing. No way to enforce append-only, signature requirements, or path restrictions.",
      solution: "Server-first permission layer that validates every git operation. Agents get scoped access with cryptographic accountability.",
      architecture: "Rust server intercepts git operations, validates against YAML policies, enforces signature requirements, and maintains audit logs. A client shim provides the UX.",
      impact: "The active line of work behind this domain. The repository carries a substantial Rust test matrix covering bypass attempts; the exact count moves with every commit, so it is not quoted as a fixed figure here."
    }
  },
  {
    id: "botfight",
    title: "BotFight",
    subtitle: "An arena for watching agents deceive each other",
    timeline: "Mar 2026: design → Feb 2026 last commit, paused",
    before: "AI behavior testing requires expensive human interaction and lacks systematic evaluation",
    after: "An automated Mafia arena where agent strategies evolve across rounds",
    outcomes: [
      { label: "Status", value: "paused" },
      { label: "Last commit", value: "Feb 2026" }
    ],
    techStack: ["Node.js", "WebSocket", "OpenAI", "Game theory", "React"],
    details: {
      problem: "AI social capabilities are hard to measure objectively. No systematic way to test deception, cooperation, and trust dynamics between agents.",
      solution: "Mafia/Werewolf game arena where agents must deceive, cooperate, and survive. Strategies evolve based on win/loss patterns.",
      architecture: "Game engine manages rounds, WebSocket handles real-time communication, agents submit actions via API, and behavioral analysis tracks strategic evolution.",
      impact: "Produced qualitatively novel deception behaviour in play. Counts of scenarios and behavioural patterns previously listed here could not be traced to a surviving artifact and were removed."
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

        {/* Retirement note */}
        {study.retiredNote && (
          <div
            style={{
              fontSize: 12,
              lineHeight: "18px",
              color: "var(--bp-dim)",
              background: "rgba(90, 122, 148, 0.10)",
              border: "1px solid var(--bp-border)",
              borderRadius: 4,
              padding: "10px 12px",
              marginBottom: 20,
            }}
          >
            {study.retiredNote}
          </div>
        )}

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
            Case histories, not a scoreboard. Each one gives the before, the
            after, and the technical decisions. Where a project is retired the
            page says so and says what happened to it. Figures appear only where
            they can be traced to something checkable; the rest is described in
            words rather than scored with a number nobody can verify.
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
              Where this stands today
            </h3>
            <p
              style={{
                fontSize: 13,
                lineHeight: "20px",
                color: "var(--bp-dim)",
                maxWidth: 560,
                margin: "0 auto",
              }}
            >
              Four of the five case studies above are retired or paused; their
              hosts no longer answer and their operational data did not survive.
              They are kept here because the work happened, not because it is
              running. The one line still in active development is the repo.box
              git permission layer. For what is live right now, see{" "}
              <Link href="/" style={{ color: "var(--bp-accent)" }}>
                the homepage
              </Link>
              .
            </p>
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