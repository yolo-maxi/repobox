"use client";

import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

// Note: metadata moved to layout.tsx or _document.tsx for client components

export default function SUPStrategyPage() {
  // Check if this is a contact request from the hire form
  const isContactRequest = typeof window !== 'undefined' ? 
    new URLSearchParams(window.location.search).get('contact') === 'true' : false;

  return (
    <>
      <RegMarks />
      <div
        style={{
          maxWidth: 720,
          margin: "0 auto",
          position: "relative",
          zIndex: 2,
          padding: "80px 40px 100px",
        }}
      >
        {/* Navigation */}
        <nav
          style={{
            display: "flex",
            gap: 16,
            marginBottom: 40,
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
        </nav>

        {/* Header */}
        <header style={{ marginBottom: 60 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              marginBottom: 16,
            }}
          >
            <h1
              style={{
                fontSize: 36,
                lineHeight: "44px",
                fontWeight: 700,
                color: "var(--bp-heading)",
              }}
            >
              SUPStrategy
            </h1>
            <span
              style={{
                fontSize: 12,
                lineHeight: "20px",
                color: "var(--bp-dim)",
                background: "rgba(90, 122, 148, 0.15)",
                padding: "4px 12px",
                borderRadius: 2,
                fontWeight: 600,
                textTransform: "uppercase",
              }}
            >
              Retired
            </span>
          </div>
          <p
            style={{
              fontSize: 18,
              lineHeight: "26px",
              color: "var(--bp-dim)",
              marginBottom: 24,
            }}
          >
            A Superfluid token trading monitor with generated signals. This is a
            case history: supstrategy.repo.box stopped responding and the
            application is no longer hosted (checked 2026-09-06).
          </p>
          <div
            style={{
              display: "flex",
              gap: 8,
              flexWrap: "wrap",
            }}
          >
            {["trading", "superfluid", "defi", "ai-agent"].map((tag) => (
              <span
                key={tag}
                style={{
                  background: "rgba(50, 100, 160, 0.2)",
                  color: "var(--bp-text)",
                  padding: "4px 12px",
                  borderRadius: 4,
                  fontSize: 12,
                }}
              >
                {tag}
              </span>
            ))}
          </div>
        </header>

        {/* Contact CTA for hire form routing */}
        {isContactRequest && (
          <section
            style={{
              background: "rgba(6, 14, 26, 0.85)",
              border: "1px solid var(--bp-accent)",
              borderRadius: 8,
              padding: "32px",
              backdropFilter: "blur(8px)",
              WebkitBackdropFilter: "blur(8px)",
              marginBottom: 60,
              textAlign: "center",
            }}
          >
            <h2
              style={{
                fontSize: 20,
                fontWeight: 600,
                color: "var(--bp-heading)",
                marginBottom: 16,
              }}
            >
              Interested in Trading/Finance Solutions?
            </h2>
            <p
              style={{
                fontSize: 16,
                color: "var(--bp-dim)",
                marginBottom: 24,
                lineHeight: "24px",
              }}
            >
              SUPStrategy showcases our expertise in DeFi trading systems. Let's discuss your
              trading automation or financial application project.
            </p>
            <div style={{ display: "flex", gap: 16, justifyContent: "center", flexWrap: "wrap" }}>
              <a
                href="https://t.me/ocean_king_bot?start=hire_trading_finance"
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  background: "var(--bp-accent)",
                  color: "#ffffff",
                  padding: "12px 24px",
                  borderRadius: 6,
                  textDecoration: "none",
                  fontWeight: 600,
                  fontSize: 14,
                }}
              >
                💬 Discuss Your Project
              </a>
              <span
                style={{
                  border: "1px solid var(--bp-border)",
                  color: "var(--bp-dim)",
                  padding: "12px 24px",
                  borderRadius: 6,
                  fontWeight: 600,
                  fontSize: 14,
                }}
              >
                Demo retired — no longer hosted
              </span>
            </div>
          </section>
        )}

        {/* Project Details */}
        <main>
          {/* Overview */}
          <section style={{ marginBottom: 60 }}>
            <h2
              style={{
                fontSize: 20,
                fontWeight: 600,
                color: "var(--bp-heading)",
                marginBottom: 20,
              }}
            >
              Overview
            </h2>
            <div
              style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 24,
                lineHeight: "24px",
                color: "var(--bp-text)",
              }}
            >
              <p style={{ marginBottom: 16 }}>
                SUPStrategy is an intelligent monitoring system for Superfluid token streams and pools. 
                It provides real-time analytics, trading signals, and automated insights for DeFi 
                participants in the Superfluid ecosystem.
              </p>
              <p style={{ marginBottom: 16 }}>
                The system tracks stream flows, pool distributions, and market dynamics to identify 
                trading opportunities and risk factors. It features AI-powered analysis that learns 
                from historical patterns and current market conditions.
              </p>
              <p>
                Built with a focus on actionable intelligence, SUPStrategy helps traders and 
                liquidity providers make informed decisions in real-time streaming finance.
              </p>
            </div>
          </section>

          {/* Features */}
          <section style={{ marginBottom: 60 }}>
            <h2
              style={{
                fontSize: 20,
                fontWeight: 600,
                color: "var(--bp-heading)",
                marginBottom: 20,
              }}
            >
              Key Features
            </h2>
            <div
              style={{
                display: "grid",
                gap: 16,
              }}
            >
              {[
                {
                  icon: "📊",
                  title: "Real-Time Analytics",
                  description: "Live monitoring of Superfluid streams, pools, and token flows",
                },
                {
                  icon: "🤖",
                  title: "AI Trading Signals",
                  description: "Machine learning algorithms identify opportunities and risks",
                },
                {
                  icon: "⚡",
                  title: "Smart Alerts",
                  description: "Configurable notifications for significant market movements",
                },
                {
                  icon: "🔄",
                  title: "Stream Monitoring",
                  description: "Track individual and aggregate streaming patterns",
                },
                {
                  icon: "💰",
                  title: "Pool Analytics",
                  description: "Distribution analysis and yield optimization insights",
                },
                {
                  icon: "📈",
                  title: "Market Intelligence",
                  description: "Historical analysis and predictive modeling",
                },
              ].map((feature) => (
                <div
                  key={feature.title}
                  style={{
                    background: "var(--bp-surface)",
                    border: "1px solid var(--bp-border)",
                    borderRadius: 8,
                    padding: 20,
                    display: "flex",
                    gap: 16,
                    alignItems: "flex-start",
                  }}
                >
                  <div style={{ fontSize: 24, flexShrink: 0 }}>{feature.icon}</div>
                  <div>
                    <h3
                      style={{
                        fontSize: 16,
                        fontWeight: 600,
                        color: "var(--bp-heading)",
                        marginBottom: 8,
                      }}
                    >
                      {feature.title}
                    </h3>
                    <p
                      style={{
                        fontSize: 14,
                        color: "var(--bp-text)",
                        lineHeight: "20px",
                      }}
                    >
                      {feature.description}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* Links */}
          <section style={{ marginBottom: 60 }}>
            <h2
              style={{
                fontSize: 20,
                fontWeight: 600,
                color: "var(--bp-heading)",
                marginBottom: 20,
              }}
            >
              Links
            </h2>
            <div
              style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: 24,
              }}
            >
              <div style={{ display: "flex", gap: 20, flexWrap: "wrap" }}>
                <span
                  style={{
                    color: "var(--bp-dim)",
                    fontSize: 14,
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                  }}
                >
                  Application retired — supstrategy.repo.box stopped responding (checked 2026-09-06)
                </span>
                <a
                  href="https://t.me/ocean_king_bot"
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{
                    color: "var(--bp-accent)",
                    textDecoration: "none",
                    fontSize: 14,
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                  }}
                >
                  💬 Contact Developer
                </a>
              </div>
            </div>
          </section>

          {/* CTA */}
          <section>
            <div
              style={{
                background: "rgba(6, 14, 26, 0.85)",
                border: "1px solid var(--bp-border)",
                borderRadius: 8,
                padding: "32px",
                backdropFilter: "blur(8px)",
                WebkitBackdropFilter: "blur(8px)",
                textAlign: "center",
              }}
            >
              <h2
                style={{
                  fontSize: 24,
                  fontWeight: 600,
                  color: "var(--bp-heading)",
                  marginBottom: 16,
                }}
              >
                Need a Similar Solution?
              </h2>
              <p
                style={{
                  fontSize: 16,
                  color: "var(--bp-dim)",
                  marginBottom: 24,
                  lineHeight: "24px",
                  maxWidth: 480,
                  margin: "0 auto 24px",
                }}
              >
                We can build custom trading systems, DeFi analytics tools, and automated 
                financial applications tailored to your needs.
              </p>
              <Link
                href="/hire"
                style={{
                  background: "var(--bp-accent)",
                  color: "#ffffff",
                  padding: "16px 32px",
                  borderRadius: 8,
                  textDecoration: "none",
                  fontWeight: 600,
                  fontSize: 16,
                  display: "inline-block",
                }}
              >
                🚀 Start Your Project
              </Link>
            </div>
          </section>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}