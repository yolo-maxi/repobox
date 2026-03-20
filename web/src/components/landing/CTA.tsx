"use client";

import { useEffect, useRef, useState } from "react";

export function LandingCTA() {
  const [copied, setCopied] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);
  const cmd = "curl -sSf https://repo.box/install.sh | sh";

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

  const handleCopy = () => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 60 }}>
      <div
        style={{
          background: "rgba(6, 14, 26, 0.85)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "clamp(24px, 6vw, 40px) clamp(20px, 5vw, 36px)",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          textAlign: "center",
        }}
      >
        <h2
          style={{
            fontSize: "clamp(22px, 5vw, 28px)",
            lineHeight: "clamp(28px, 6vw, 36px)",
            color: "var(--bp-heading)",
            fontWeight: 700,
            marginBottom: "clamp(6px, 2vw, 8px)",
          }}
        >
          Get started
        </h2>
        <p
          style={{
            fontSize: "clamp(13px, 3vw, 14px)",
            lineHeight: "clamp(18px, 4vw, 22px)",
            color: "var(--bp-dim)",
            marginBottom: "clamp(24px, 5vw, 32px)",
          }}
        >
          One command. Secure your first repo in under a minute.
        </p>

        {/* Curl command — the main CTA */}
        <div
          onClick={handleCopy}
          style={{
            background: "rgba(0, 0, 0, 0.5)",
            border: "1px solid var(--bp-border)",
            borderRadius: 8,
            padding: "clamp(16px, 4vw, 20px) clamp(16px, 4vw, 24px)",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: "clamp(12px, 3vw, 16px)",
            cursor: "pointer",
            transition: "border-color 0.2s",
            marginBottom: "clamp(20px, 5vw, 28px)",
          }}
          className="cta-command-box"
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              overflow: "hidden",
            }}
          >
            <span
              style={{
                color: "var(--bp-accent)",
                fontWeight: 600,
                flexShrink: 0,
                fontSize: 16,
              }}
            >
              $
            </span>
            <code
              style={{
                fontSize: 15,
                lineHeight: "22px",
                color: "var(--bp-heading)",
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
                fontFamily: "var(--font-mono), monospace",
                fontWeight: 500,
              }}
            >
              {cmd}
            </code>
          </div>
          <span
            style={{
              color: copied ? "var(--bp-accent)" : "var(--bp-dim)",
              fontFamily: "var(--font-mono), monospace",
              fontSize: 12,
              flexShrink: 0,
              transition: "color 0.2s",
              border: `1px solid ${copied ? "var(--bp-accent)" : "var(--bp-border)"}`,
              padding: "6px 14px",
              borderRadius: 4,
            }}
          >
            {copied ? "copied!" : "copy"}
          </span>
        </div>

        {/* Secondary links */}
        <div
          style={{
            display: "flex",
            justifyContent: "center",
            gap: 32,
            flexWrap: "wrap",
          }}
        >
          <a
            href="/playground"
            style={{
              fontFamily: "var(--font-mono), monospace",
              fontSize: 13,
              color: "var(--bp-accent)",
              textDecoration: "none",
              transition: "opacity 0.2s",
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
            }}
          >
            <span style={{ fontSize: 15 }}>▶</span> Try in Playground
          </a>
          <a
            href="/SKILL.md"
            style={{
              fontFamily: "var(--font-mono), monospace",
              fontSize: 13,
              color: "var(--bp-accent2)",
              textDecoration: "none",
              transition: "opacity 0.2s",
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
            }}
          >
            <span style={{ fontSize: 15 }}>📄</span> SKILL.md for agents
          </a>
          <a
            href="/llms.txt"
            style={{
              fontFamily: "var(--font-mono), monospace",
              fontSize: 13,
              color: "var(--bp-accent2)",
              textDecoration: "none",
              transition: "opacity 0.2s",
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
            }}
          >
            <span style={{ fontSize: 15 }}>🤖</span> llms.txt
          </a>
        </div>
      </div>
    </section>
  );
}
