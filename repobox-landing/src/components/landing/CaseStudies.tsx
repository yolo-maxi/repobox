"use client";

import { useEffect, useRef } from "react";

const CASE_STUDIES = [
  {
    title: "Solo + Codex",
    tag: "simple",
    description:
      "One developer, one AI agent. Agent gets feature branches, can't touch main or the config.",
    before: "Manual review of every AI commit",
    after: "Auto-sandboxed, merge when ready",
  },
  {
    title: "Team + Agents",
    tag: "standard",
    description:
      "Founders control everything. Multiple agents work in parallel on feature branches.",
    before: "Shared credentials, trust issues",
    after: "Per-agent keys, audit trail",
  },
  {
    title: "Multi-Agent Swarm",
    tag: "advanced",
    description:
      "Orchestrator agent spawns sub-agents. Each gets scoped permissions via append-only config changes.",
    before: "Can't safely delegate",
    after: "Hierarchical trust model",
  },
];

export function LandingCaseStudies() {
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
      <h2
        style={{
          fontSize: 12,
          lineHeight: "20px",
          textTransform: "uppercase",
          letterSpacing: "0.12em",
          color: "var(--bp-dim)",
          fontWeight: 500,
          marginBottom: 20,
        }}
      >
        Use Cases
      </h2>

      <div style={{ position: "relative" }}>
        <div
          className="case-studies-row"
          style={{
            display: "flex",
            gap: 16,
            overflowX: "auto",
            scrollSnapType: "x mandatory",
            WebkitOverflowScrolling: "touch",
            paddingBottom: 8,
          }}
        >
          {CASE_STUDIES.map((study) => (
            <div
              key={study.title}
              className="case-study-card"
              style={{
                flex: "0 0 280px",
                scrollSnapAlign: "start",
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 20,
              }}
            >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "baseline",
                marginBottom: 10,
              }}
            >
              <div style={{ fontWeight: 700, fontSize: 15, color: "#ffffff" }}>
                {study.title}
              </div>
              <span
                style={{
                  fontSize: 11,
                  color: "var(--bp-accent)",
                  background: "rgba(79,195,247,0.15)",
                  padding: "0 10px",
                  borderRadius: 2,
                  fontWeight: 600,
                }}
              >
                {study.tag}
              </span>
            </div>
            <div
              style={{
                fontSize: 12,
                lineHeight: "18px",
                color: "var(--bp-text)",
                marginBottom: 14,
              }}
            >
              {study.description}
            </div>
            <div style={{ fontSize: 11, lineHeight: "16px", color: "var(--bp-dim)" }}>
              <span style={{ color: "var(--bp-accent)" }}>Before:</span> {study.before}
              <br />
              <span style={{ color: "var(--bp-accent)" }}>After:</span> {study.after}
            </div>
          </div>
        ))}
        </div>
        {/* Right-edge gradient fade to hint scrollability */}
        <div
          className="case-studies-fade"
          style={{
            position: "absolute",
            top: 0,
            right: 0,
            width: 40,
            height: "100%",
            background: "linear-gradient(to right, transparent, var(--bp-bg))",
            pointerEvents: "none",
            borderRadius: "0 8px 8px 0",
          }}
        />
      </div>
    </section>
  );
}
