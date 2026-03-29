"use client";

import { useEffect, useRef } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export default function TrustPage() {
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
        style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
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
              href="/blog/"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              blog
            </Link>
          </nav>
          <div
            className="font-mono font-bold"
            style={{ fontSize: 36, lineHeight: 1.1, marginBottom: 16 }}
          >
            Security & Trust
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 500,
            }}
          >
            How repo.box protects your code and maintains security through transparent practices.
          </p>
        </header>

        <main ref={pageRef} className="reveal">
          {/* Trust Indicators */}
          <section
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
              marginBottom: 40,
            }}
          >
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
                gap: 20,
                textAlign: "center",
              }}
            >
              <div>
                <div style={{ fontSize: 20, fontWeight: 700, color: "#4ade80" }}>
                  99.9%
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Uptime
                </div>
              </div>
              <div>
                <div style={{ fontSize: 20, fontWeight: 700, color: "var(--bp-accent)" }}>
                  2026-03-15
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Last Security Audit
                </div>
              </div>
              <div>
                <div style={{ fontSize: 20, fontWeight: 700, color: "var(--bp-accent)" }}>
                  0
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Active Incidents
                </div>
              </div>
            </div>
          </section>

          {/* Authentication Model */}
          <section style={{ marginBottom: 40 }}>
            <h2
              style={{
                fontSize: 20,
                lineHeight: "28px",
                fontWeight: 600,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Authentication Model
            </h2>
            <div
              style={{
                fontSize: 15,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 16,
              }}
            >
              repo.box uses magic link authentication exclusively. No passwords, no forms, no complexity.
            </div>
            <ul
              style={{
                fontSize: 14,
                lineHeight: "22px",
                color: "var(--bp-dim)",
                marginLeft: 20,
                marginBottom: 0,
              }}
            >
              <li style={{ marginBottom: 8 }}>Token-based access via secure URLs</li>
              <li style={{ marginBottom: 8 }}>HttpOnly cookies for session management</li>
              <li style={{ marginBottom: 8 }}>No password storage or credential databases</li>
              <li style={{ marginBottom: 8 }}>EVM wallet signatures for agent authentication</li>
            </ul>
          </section>

          {/* Deployment Safety */}
          <section style={{ marginBottom: 40 }}>
            <h2
              style={{
                fontSize: 20,
                lineHeight: "28px",
                fontWeight: 600,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Deployment Safety
            </h2>
            <div
              style={{
                fontSize: 15,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 16,
              }}
            >
              Static sites only. No server-side execution. Continuous security monitoring.
            </div>
            <ul
              style={{
                fontSize: 14,
                lineHeight: "22px",
                color: "var(--bp-dim)",
                marginLeft: 20,
                marginBottom: 0,
              }}
            >
              <li style={{ marginBottom: 8 }}>Only compiled output (dist/, build/) reaches production</li>
              <li style={{ marginBottom: 8 }}>Automated security audits before every deployment</li>
              <li style={{ marginBottom: 8 }}>No source code, tokens, or credentials in web-accessible paths</li>
              <li style={{ marginBottom: 8 }}>Daily vulnerability scans and dependency updates</li>
            </ul>
          </section>

          {/* Data Handling */}
          <section style={{ marginBottom: 40 }}>
            <h2
              style={{
                fontSize: 20,
                lineHeight: "28px",
                fontWeight: 600,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Data Handling
            </h2>
            <div
              style={{
                fontSize: 15,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 16,
              }}
            >
              Minimal data collection. Clear retention policies. No third-party tracking.
            </div>
            <ul
              style={{
                fontSize: 14,
                lineHeight: "22px",
                color: "var(--bp-dim)",
                marginLeft: 20,
                marginBottom: 0,
              }}
            >
              <li style={{ marginBottom: 8 }}>Repository metadata only (permissions, access logs)</li>
              <li style={{ marginBottom: 8 }}>No analytics, tracking pixels, or behavior monitoring</li>
              <li style={{ marginBottom: 8 }}>30-day log retention for security audit trails</li>
              <li style={{ marginBottom: 8 }}>All data stored in EU/UK data centers</li>
            </ul>
          </section>

          {/* Infrastructure Security */}
          <section style={{ marginBottom: 40 }}>
            <h2
              style={{
                fontSize: 20,
                lineHeight: "28px",
                fontWeight: 600,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Infrastructure Security
            </h2>
            <div
              style={{
                fontSize: 15,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 16,
              }}
            >
              Hardened servers, restricted access controls, comprehensive backup strategy.
            </div>
            <ul
              style={{
                fontSize: 14,
                lineHeight: "22px",
                color: "var(--bp-dim)",
                marginLeft: 20,
                marginBottom: 0,
              }}
            >
              <li style={{ marginBottom: 8 }}>SSH key-only access, no password authentication</li>
              <li style={{ marginBottom: 8 }}>Firewall restrictions on all non-essential ports</li>
              <li style={{ marginBottom: 8 }}>Automated security updates and vulnerability patching</li>
              <li style={{ marginBottom: 8 }}>Daily encrypted backups with 90-day retention</li>
            </ul>
          </section>

          {/* Service Uptime Policy */}
          <section style={{ marginBottom: 40 }}>
            <h2
              style={{
                fontSize: 20,
                lineHeight: "28px",
                fontWeight: 600,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Service Uptime Policy
            </h2>
            <div
              style={{
                fontSize: 15,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 16,
              }}
            >
              99.9% uptime commitment with transparent incident reporting.
            </div>
            <ul
              style={{
                fontSize: 14,
                lineHeight: "22px",
                color: "var(--bp-dim)",
                marginLeft: 20,
                marginBottom: 0,
              }}
            >
              <li style={{ marginBottom: 8 }}>Service-level objective of 99.9% monthly uptime</li>
              <li style={{ marginBottom: 8 }}>Public incident tracking and post-mortem reports</li>
              <li style={{ marginBottom: 8 }}>Automated monitoring with &lt;2 minute detection time</li>
              <li style={{ marginBottom: 8 }}>Zero-downtime deployments during business hours</li>
            </ul>
          </section>

          {/* Contact Information */}
          <section
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 24,
              marginBottom: 40,
            }}
          >
            <h2
              style={{
                fontSize: 20,
                lineHeight: "28px",
                fontWeight: 600,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Security & Enterprise Contact
            </h2>
            <div
              style={{
                fontSize: 15,
                lineHeight: "24px",
                color: "var(--bp-text)",
                marginBottom: 16,
              }}
            >
              For security concerns, enterprise discussions, or compliance questions.
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <div style={{ fontSize: 14, color: "var(--bp-dim)" }}>
                <strong style={{ color: "var(--bp-text)" }}>Security Issues:</strong>{" "}
                <a
                  href="mailto:security@repo.box"
                  style={{ color: "var(--bp-accent)", textDecoration: "none" }}
                >
                  security@repo.box
                </a>
              </div>
              <div style={{ fontSize: 14, color: "var(--bp-dim)" }}>
                <strong style={{ color: "var(--bp-text)" }}>Enterprise Sales:</strong>{" "}
                <a
                  href="mailto:enterprise@repo.box"
                  style={{ color: "var(--bp-accent)", textDecoration: "none" }}
                >
                  enterprise@repo.box
                </a>
              </div>
              <div style={{ fontSize: 14, color: "var(--bp-dim)" }}>
                <strong style={{ color: "var(--bp-text)" }}>General Support:</strong>{" "}
                <a
                  href="https://t.me/ocean_king_bot"
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{ color: "var(--bp-accent)", textDecoration: "none" }}
                >
                  Ocean on Telegram
                </a>
              </div>
            </div>
          </section>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}