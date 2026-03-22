"use client";

import { useEffect, useRef } from "react";

const PRINCIPLES = [
  "Portable identity — your EVM key works everywhere",
  "The config IS the security policy — no hidden settings",
  "Zero trust by default — agents earn access, not inherit it",
  "Expressive permissions — programmable guardrails for AI",
  "Composable — any on-chain primitive as a permission source",
];

export function LandingEthos() {
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
          background: "transparent",
          border: "1px solid rgba(79, 195, 247, 0.15)",
          borderRadius: 8,
          padding: "32px 36px",
        }}
      >
        <h2
          style={{
            fontSize: 14,
            lineHeight: "20px",
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent)",
            fontWeight: 500,
            marginBottom: 24,
          }}
        >
          Design Principles
        </h2>
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {PRINCIPLES.map((principle, i) => (
            <li
              key={i}
              style={{
                fontSize: 14,
                lineHeight: "22px",
                color: "var(--bp-heading)",
                padding: "10px 0 10px 24px",
                position: "relative",
                borderBottom:
                  i < PRINCIPLES.length - 1
                    ? "1px solid rgba(50, 100, 160, 0.12)"
                    : "none",
              }}
            >
              <span
                style={{
                  position: "absolute",
                  left: 0,
                  color: "var(--bp-accent)",
                }}
              >
                →
              </span>
              {principle}
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
