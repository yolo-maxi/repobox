"use client";

import { useEffect, useRef, useState } from "react";

interface CommitEntry {
  time: string;
  message: string;
}

interface RepoSection {
  name: string;
  commits: CommitEntry[];
}

// Static data for now — can be made dynamic later
const SHIPPED_DATA: RepoSection[] = [
  {
    name: "repobox",
    commits: [
      { time: "12:20", message: "feat: playground AI config generator with Venice API" },
      { time: "10:15", message: "docs: update SKILL.md with flat groups format" },
      { time: "08:30", message: "fix: YAML quoting for branch targets" },
    ],
  },
];

export function LandingShippedToday() {
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
        What We Shipped Today
      </h2>

      {SHIPPED_DATA.map((repo) => (
        <div
          key={repo.name}
          style={{
            background: "var(--bp-surface)",
            border: "1px solid var(--bp-border)",
            borderRadius: 8,
            padding: 20,
            marginBottom: 16,
          }}
        >
          <div
            style={{
              fontSize: 12,
              lineHeight: "20px",
              color: "var(--bp-accent)",
              fontWeight: 600,
              marginBottom: 12,
            }}
          >
            📦 {repo.name}
          </div>
          <div style={{ marginLeft: 12 }}>
            {repo.commits.map((commit, i) => (
              <div
                key={i}
                style={{
                  display: "flex",
                  alignItems: "flex-start",
                  gap: 8,
                  marginBottom: 8,
                  fontSize: 12,
                  lineHeight: "20px",
                }}
              >
                <span
                  style={{
                    color: "var(--bp-accent2)",
                    fontWeight: 500,
                    whiteSpace: "nowrap",
                  }}
                >
                  {commit.time}
                </span>
                <span style={{ color: "var(--bp-text)" }}>{commit.message}</span>
              </div>
            ))}
          </div>
        </div>
      ))}
    </section>
  );
}
