"use client";

import Link from "next/link";

import { useReveal } from "./useReveal";
import { CardBorder, sectionTitleStyle } from "./shared";

type WorkStatus = "live" | "public" | "building";

interface WorkEntry {
  name: string;
  status: WorkStatus;
  what: string;
  proof: { label: string; href: string }[];
}

// Every link below was checked live before it was added here. If one of them
// stops resolving, remove the entry rather than leaving a dead receipt on the
// homepage — the whole point of this section is that it can be verified.
const WORK: WorkEntry[] = [
  {
    name: "RunYard",
    status: "live",
    what:
      "A control plane for long-running agent work on your own machines. Runs, runners, boards and approvals, so an agent's work is durable and reviewable instead of a chat log you lost.",
    proof: [{ label: "runyard.repo.box", href: "https://runyard.repo.box" }],
  },
  {
    name: "Frontier",
    status: "public",
    what:
      "An on-chain order book, plus a prediction-market app on top of it. Contracts are source-available under BUSL-1.1 rather than described in a pitch deck.",
    proof: [
      { label: "frontier.repo.box", href: "https://frontier.repo.box" },
      { label: "the app", href: "https://frontier-pm.repo.box" },
      { label: "source", href: "https://github.com/yolo-maxi/frontier-orderbook" },
    ],
  },
  {
    name: "Concierge",
    status: "public",
    what:
      "A deliberately powerless landing-page agent. It answers from one page brief and holds no keys, no tools and no database — the interesting engineering is everything it is not allowed to do.",
    proof: [{ label: "source", href: "https://github.com/yolo-maxi/concierge" }],
  },
  {
    name: "repo.box git layer",
    status: "building",
    what:
      "The thing this domain is named after: a permission layer that makes a git repository safe to hand to an agent. Every commit is signed and checked against a config file that lives in the repo.",
    proof: [
      { label: "what it does", href: "/git" },
      { label: "try the config", href: "/playground" },
    ],
  },
];

const STATUS_LABEL: Record<WorkStatus, string> = {
  live: "live",
  public: "source available",
  building: "in progress",
};

const proofLinkStyle = {
  fontFamily: "var(--font-mono), monospace",
  fontSize: 12,
  lineHeight: "20px",
  color: "var(--bp-accent2)",
} as const;

function StatusPill({ status }: { status: WorkStatus }) {
  return (
    <span
      style={{
        fontSize: 11,
        lineHeight: "20px",
        color: "var(--bp-accent)",
        background: "rgba(79,195,247,0.15)",
        padding: "0 10px",
        borderRadius: 2,
        fontWeight: 600,
        whiteSpace: "nowrap",
      }}
    >
      {STATUS_LABEL[status]}
    </span>
  );
}

function WorkCard({ entry }: { entry: WorkEntry }) {
  return (
    <div
      className="project-card"
      style={{
        position: "relative",
        overflow: "hidden",
        background: "var(--bp-surface)",
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
        padding: 20,
        marginBottom: 16,
        transition: "border-color 0.2s",
      }}
    >
      <CardBorder />
      <div style={{ position: "relative", zIndex: 2 }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "baseline",
            gap: 12,
            marginBottom: 8,
          }}
        >
          <div
            style={{
              fontWeight: 700,
              fontSize: 16,
              lineHeight: "22px",
              color: "#ffffff",
            }}
          >
            {entry.name}
          </div>
          <StatusPill status={entry.status} />
        </div>

        <p
          style={{
            fontSize: 13,
            lineHeight: "21px",
            color: "var(--bp-text)",
            margin: "0 0 14px",
          }}
        >
          {entry.what}
        </p>

        <div style={{ display: "flex", gap: 14, flexWrap: "wrap" }}>
          {entry.proof.map((link) =>
            link.href.startsWith("http") ? (
              <a
                key={link.href}
                href={link.href}
                target="_blank"
                rel="noopener noreferrer"
                style={proofLinkStyle}
              >
                {link.label} →
              </a>
            ) : (
              <Link key={link.href} href={link.href} style={proofLinkStyle}>
                {link.label} →
              </Link>
            )
          )}
        </div>
      </div>
    </div>
  );
}

export function LandingWork() {
  const sectionRef = useReveal<HTMLElement>();

  return (
    <section id="work" ref={sectionRef} className="reveal" style={{ marginBottom: 72 }}>
      <h2 style={sectionTitleStyle}>What we&apos;re building</h2>

      <p
        style={{
          fontSize: 14,
          lineHeight: "23px",
          color: "var(--bp-dim)",
          maxWidth: 560,
          margin: "0 0 24px",
        }}
      >
        Four things, all of them open to inspection. We would rather you clicked
        through and formed your own opinion than read an adjective about it.
      </p>

      {WORK.map((entry) => (
        <WorkCard key={entry.name} entry={entry} />
      ))}
    </section>
  );
}
