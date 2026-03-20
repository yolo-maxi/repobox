"use client";

import { useEffect, useRef } from "react";

export function LandingHireCTA() {
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
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 80 }}>
      <div
        style={{
          background: "rgba(6, 14, 26, 0.85)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "40px 36px",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          textAlign: "center",
        }}
      >
        <h2
          style={{
            fontSize: 28,
            lineHeight: "36px",
            color: "var(--bp-heading)",
            fontWeight: 700,
            marginBottom: 8,
          }}
        >
          Hire Our Agents
        </h2>
        <p
          style={{
            fontSize: 16,
            lineHeight: "24px",
            color: "var(--bp-dim)",
            marginBottom: 32,
            maxWidth: 480,
            margin: "0 auto 32px",
          }}
        >
          Need an AI agent for your project? Our studio builds custom agents for coding, research, social media, and more.
        </p>

        {/* Primary CTA Button */}
        <div style={{ marginBottom: 24 }}>
          <a
            href="https://t.me/ocean_king_bot"
            target="_blank"
            rel="noopener noreferrer"
            style={{
              display: "inline-flex",
              alignItems: "center",
              justifyContent: "center",
              gap: 12,
              background: "var(--bp-accent)",
              color: "#ffffff",
              padding: "16px 32px",
              borderRadius: 8,
              textDecoration: "none",
              fontWeight: 600,
              fontSize: 16,
              transition: "all 0.2s",
              border: "2px solid var(--bp-accent)",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.color = "var(--bp-accent)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "var(--bp-accent)";
              e.currentTarget.style.color = "#ffffff";
            }}
          >
            <span style={{ fontSize: 20 }}>💬</span>
            Start a conversation
          </a>
        </div>

        {/* Secondary CTA */}
        <div>
          <a
            href="/docs"
            style={{
              fontFamily: "var(--font-mono), monospace",
              fontSize: 14,
              color: "var(--bp-accent2)",
              textDecoration: "none",
              transition: "color 0.2s",
              display: "inline-flex",
              alignItems: "center",
              gap: 8,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.color = "var(--bp-heading)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = "var(--bp-accent2)";
            }}
          >
            <span style={{ fontSize: 16 }}>📖</span> Read the docs
          </a>
        </div>
      </div>
    </section>
  );
}