"use client";

import { useEffect, useRef } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function HirePage() {
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
          maxWidth: 600,
          margin: "0 auto",
          position: "relative",
          zIndex: 2,
          padding: "80px 40px 100px",
          textAlign: "center",
        }}
      >
        {/* Header */}
        <header style={{ marginBottom: 60 }}>
          <nav
            style={{
              display: "flex",
              gap: 16,
              justifyContent: "center",
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
              href="/proof"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              proof
            </Link>
          </nav>
          <div
            className="font-mono font-bold"
            style={{ fontSize: 36, lineHeight: 1.1, marginBottom: 16 }}
          >
            Start Your Build
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 500,
              margin: "0 auto",
            }}
          >
            Tell Ocean what you need built. From AI agents to web apps to automation—
            we ship fast, secure, and production-ready.
          </p>
        </header>

        {/* Main CTA */}
        <main ref={pageRef} className="reveal" style={{ marginBottom: 60 }}>
          <div
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 12,
              padding: 40,
              marginBottom: 40,
            }}
          >
            <div
              style={{
                fontSize: 18,
                fontWeight: 700,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Talk to Ocean
            </div>
            <p
              style={{
                fontSize: 14,
                lineHeight: "20px",
                color: "var(--bp-text)",
                marginBottom: 24,
              }}
            >
              Describe your project idea, timeline, and requirements. Ocean will
              route you to the right build approach and provide a realistic estimate.
            </p>
            <a
              href="https://t.me/ocean_king_bot"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                display: "inline-block",
                fontSize: 16,
                fontWeight: 600,
                color: "#ffffff",
                background: "var(--bp-accent)",
                padding: "16px 32px",
                borderRadius: 6,
                textDecoration: "none",
                transition: "opacity 0.2s",
              }}
              onMouseOver={(e) => (e.currentTarget.style.opacity = "0.8")}
              onMouseOut={(e) => (e.currentTarget.style.opacity = "1")}
            >
              Start Conversation →
            </a>
          </div>

          {/* What We Build */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
              gap: 20,
              marginBottom: 40,
            }}
          >
            <div
              style={{
                background: "rgba(79, 195, 247, 0.05)",
                border: "1px solid rgba(79, 195, 247, 0.2)",
                borderRadius: 8,
                padding: 20,
                textAlign: "left",
              }}
            >
              <div
                style={{
                  fontSize: 14,
                  fontWeight: 700,
                  color: "var(--bp-accent)",
                  marginBottom: 8,
                }}
              >
                AI Agents
              </div>
              <p
                style={{
                  fontSize: 12,
                  lineHeight: "16px",
                  color: "var(--bp-text)",
                }}
              >
                Autonomous agents for trading, travel, verification, social interaction
              </p>
            </div>
            <div
              style={{
                background: "rgba(74, 222, 128, 0.05)",
                border: "1px solid rgba(74, 222, 128, 0.2)",
                borderRadius: 8,
                padding: 20,
                textAlign: "left",
              }}
            >
              <div
                style={{
                  fontSize: 14,
                  fontWeight: 700,
                  color: "#4ade80",
                  marginBottom: 8,
                }}
              >
                Web Apps
              </div>
              <p
                style={{
                  fontSize: 12,
                  lineHeight: "16px",
                  color: "var(--bp-text)",
                }}
              >
                Full-stack apps with crypto payments, real-time features, modern UX
              </p>
            </div>
            <div
              style={{
                background: "rgba(251, 191, 36, 0.05)",
                border: "1px solid rgba(251, 191, 36, 0.2)",
                borderRadius: 8,
                padding: 20,
                textAlign: "left",
              }}
            >
              <div
                style={{
                  fontSize: 14,
                  fontWeight: 700,
                  color: "#fbbf24",
                  marginBottom: 8,
                }}
              >
                Automation
              </div>
              <p
                style={{
                  fontSize: 12,
                  lineHeight: "16px",
                  color: "var(--bp-text)",
                }}
              >
                Workflow automation, API integrations, scheduled tasks, monitoring
              </p>
            </div>
          </div>

          {/* Process */}
          <div style={{ textAlign: "left" }}>
            <h3
              style={{
                fontSize: 16,
                fontWeight: 700,
                color: "#ffffff",
                marginBottom: 16,
                textAlign: "center",
              }}
            >
              How It Works
            </h3>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(250px, 1fr))",
                gap: 16,
              }}
            >
              <div style={{ padding: "16px 0" }}>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  Step 1
                </div>
                <div
                  style={{
                    fontSize: 13,
                    color: "var(--bp-text)",
                    lineHeight: "18px",
                  }}
                >
                  Tell Ocean your idea, timeline, and rough budget
                </div>
              </div>
              <div style={{ padding: "16px 0" }}>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  Step 2
                </div>
                <div
                  style={{
                    fontSize: 13,
                    color: "var(--bp-text)",
                    lineHeight: "18px",
                  }}
                >
                  Get a detailed proposal with timeline, tech stack, and cost
                </div>
              </div>
              <div style={{ padding: "16px 0" }}>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--bp-accent)",
                    fontWeight: 600,
                    textTransform: "uppercase",
                    letterSpacing: "0.1em",
                    marginBottom: 6,
                  }}
                >
                  Step 3
                </div>
                <div
                  style={{
                    fontSize: 13,
                    color: "var(--bp-text)",
                    lineHeight: "18px",
                  }}
                >
                  Watch your build ship in weeks, not months
                </div>
              </div>
            </div>
          </div>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}