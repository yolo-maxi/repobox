"use client";

import { useEffect, useRef } from "react";

const PRINCIPLES = [
  "Agents use normal git — the shim is invisible",
  "EVM keypairs as the identity layer",
  "Every agent signs commits with its own key",
  "Everything is a file in the repo — rules, identities, workflows",
  "Enforced locally, no server required",
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
            fontSize: 12,
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
                fontSize: 13,
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
