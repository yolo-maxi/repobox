"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export interface ServicePackage {
  id: string;
  name: string;
  priceRange: string;
  timeline: string;
  description: string;
  deliverables: string[];
  milestones: string[];
  techStack: string[];
  included: string[];
  excluded: string[];
  exampleProjects: string[];
  bestFit: string;
  tiers: {
    simple: { price: string; description: string };
    standard: { price: string; description: string };
    complex: { price: string; description: string };
  };
}

const SERVICE_PACKAGES: ServicePackage[] = [
  {
    id: "prototype",
    name: "Prototype Sprint",
    priceRange: "$2k-5k",
    timeline: "1-2 weeks",
    description: "MVP with core features, basic UI, proof of concept",
    deliverables: [
      "Working prototype with core functionality",
      "Basic UI/UX design",
      "Core feature implementation",
      "Technical documentation",
      "Deployment guide"
    ],
    milestones: ["Requirements → Design → Development → Testing → Delivery"],
    techStack: ["React/Next.js", "Node.js", "PostgreSQL", "Vercel/Railway"],
    included: [
      "Up to 3 core features",
      "Responsive web design",
      "Basic error handling",
      "1 round of revisions",
      "Source code delivery"
    ],
    excluded: [
      "Advanced security audits",
      "Production monitoring",
      "Complex integrations",
      "Custom branding",
      "Ongoing maintenance"
    ],
    exampleProjects: ["BotFight (social deduction game)", "SSS agent verification", "Cabin flight booking"],
    bestFit: "Testing new ideas, validating concepts, showcasing functionality to stakeholders",
    tiers: {
      simple: { price: "$2k", description: "Single feature, basic UI, standard tech stack" },
      standard: { price: "$3.5k", description: "2-3 features, custom design, API integrations" },
      complex: { price: "$5k", description: "Multi-feature MVP, real-time data, advanced UI" }
    }
  },
  {
    id: "automation",
    name: "Automation Sprint",
    priceRange: "$1k-3k",
    timeline: "3-7 days",
    description: "Workflow automation, API integration, scheduled tasks",
    deliverables: [
      "Automated workflow implementation",
      "API integration setup",
      "Scheduling configuration",
      "Error handling & monitoring",
      "Documentation & handoff"
    ],
    milestones: ["Analysis → Integration → Testing → Deployment → Monitoring"],
    techStack: ["Node.js", "Cron/GitHub Actions", "API integrations", "Webhooks"],
    included: [
      "Up to 5 automation workflows",
      "Error handling & retries",
      "Basic monitoring alerts",
      "Documentation",
      "1 month of bug fixes"
    ],
    excluded: [
      "Complex ML/AI processing",
      "Real-time UI dashboards",
      "Multi-system orchestration",
      "Custom hardware integration",
      "24/7 support"
    ],
    exampleProjects: ["Farcaster posting automation", "GitHub issue triage", "Email digest systems"],
    bestFit: "Eliminating repetitive tasks, connecting disparate systems, improving operational efficiency",
    tiers: {
      simple: { price: "$1k", description: "Single workflow, standard APIs, basic monitoring" },
      standard: { price: "$2k", description: "2-3 workflows, custom logic, enhanced monitoring" },
      complex: { price: "$3k", description: "Complex orchestration, multiple integrations, advanced error handling" }
    }
  },
  {
    id: "launch",
    name: "Launch Sprint",
    priceRange: "$5k-15k",
    timeline: "2-4 weeks",
    description: "Production-ready app with hosting, monitoring, security",
    deliverables: [
      "Production-ready application",
      "Scalable hosting setup",
      "Security implementation",
      "Monitoring & analytics",
      "Performance optimization",
      "Launch support"
    ],
    milestones: ["Architecture → Development → Security → Testing → Launch → Support"],
    techStack: ["React/Next.js", "Node.js", "PostgreSQL", "Redis", "Docker", "CloudFlare"],
    included: [
      "Full feature development",
      "Security audit & hardening",
      "Production hosting setup",
      "Performance optimization",
      "Monitoring & alerts",
      "2 rounds of revisions",
      "30-day launch support"
    ],
    excluded: [
      "Marketing website",
      "Customer support system",
      "Complex compliance (SOC2, etc)",
      "Mobile apps",
      "Long-term maintenance contracts"
    ],
    exampleProjects: ["Oceangram VS Code extension", "repo.box CLI tool", "Trading strategy platforms"],
    bestFit: "Bringing validated concepts to production, scaling existing prototypes, launching revenue-generating products",
    tiers: {
      simple: { price: "$5k", description: "Standard web app, basic features, simple hosting" },
      standard: { price: "$10k", description: "Complex features, integrations, advanced security" },
      complex: { price: "$15k", description: "Multi-service architecture, real-time features, enterprise security" }
    }
  }
];

export default function PackagesPage() {
  const pageRef = useRef<HTMLElement>(null);
  const [selectedTier, setSelectedTier] = useState<Record<string, string>>({
    prototype: 'standard',
    automation: 'standard',
    launch: 'standard'
  });

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

  const handleBookSprint = (packageId: string, tier: string) => {
    const pkg = SERVICE_PACKAGES.find(p => p.id === packageId);
    const tierInfo = pkg?.tiers[tier as keyof typeof pkg.tiers];
    
    const context = `
I'm interested in the ${pkg?.name} (${tierInfo?.price}).
Project type: ${pkg?.name}
Complexity: ${tier}
Price range: ${tierInfo?.price}

Please help me get started!`;
    
    const telegramUrl = `https://t.me/ocean_king_bot?start=${encodeURIComponent(context)}`;
    window.open(telegramUrl, '_blank');
  };

  return (
    <>
      <RegMarks />
      <div
        style={{
          maxWidth: 1200,
          margin: "0 auto",
          position: "relative",
          zIndex: 2,
          padding: "80px 40px 100px",
        }}
      >
        {/* Header */}
        <header style={{ marginBottom: 60, textAlign: "center" }}>
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
              href="/hire"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              hire
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
            Service Packages
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 600,
              margin: "0 auto",
            }}
          >
            Fixed-scope sprints with clear deliverables, timelines, and pricing.
            No discovery calls, no ambiguous estimates—just fast, predictable shipping.
          </p>
        </header>

        {/* Packages Grid */}
        <main ref={pageRef} className="reveal" style={{ marginBottom: 60 }}>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(350px, 1fr))",
              gap: 40,
              marginBottom: 60,
            }}
          >
            {SERVICE_PACKAGES.map((pkg) => (
              <div
                key={pkg.id}
                style={{
                  background: "var(--bp-surface)",
                  border: "1px solid var(--bp-border)",
                  borderRadius: 12,
                  padding: 32,
                  height: "fit-content",
                }}
              >
                {/* Package Header */}
                <div style={{ marginBottom: 24 }}>
                  <div
                    style={{
                      fontSize: 20,
                      fontWeight: 700,
                      color: "#ffffff",
                      marginBottom: 8,
                    }}
                  >
                    {pkg.name}
                  </div>
                  <div
                    style={{
                      fontSize: 14,
                      color: "var(--bp-accent)",
                      fontWeight: 600,
                      marginBottom: 8,
                    }}
                  >
                    {pkg.priceRange} • {pkg.timeline}
                  </div>
                  <p
                    style={{
                      fontSize: 14,
                      color: "var(--bp-text)",
                      lineHeight: "20px",
                    }}
                  >
                    {pkg.description}
                  </p>
                </div>

                {/* Tier Selection */}
                <div style={{ marginBottom: 24 }}>
                  <div
                    style={{
                      fontSize: 12,
                      color: "var(--bp-dim)",
                      marginBottom: 12,
                      fontWeight: 600,
                    }}
                  >
                    COMPLEXITY TIER
                  </div>
                  <div style={{ display: "flex", gap: 8 }}>
                    {['simple', 'standard', 'complex'].map((tier) => (
                      <button
                        key={tier}
                        onClick={() => setSelectedTier(prev => ({ ...prev, [pkg.id]: tier }))}
                        style={{
                          flex: 1,
                          padding: "8px 12px",
                          fontSize: 11,
                          fontWeight: 600,
                          textTransform: "uppercase",
                          border: "1px solid var(--bp-border)",
                          borderRadius: 4,
                          background: selectedTier[pkg.id] === tier ? "var(--bp-accent)" : "transparent",
                          color: selectedTier[pkg.id] === tier ? "#000" : "var(--bp-text)",
                          cursor: "pointer",
                          transition: "all 0.2s",
                        }}
                      >
                        {tier}
                      </button>
                    ))}
                  </div>
                  <div
                    style={{
                      fontSize: 12,
                      color: "var(--bp-text)",
                      marginTop: 8,
                      minHeight: 32,
                      lineHeight: "16px",
                    }}
                  >
                    {pkg.tiers[selectedTier[pkg.id] as keyof typeof pkg.tiers]?.description}
                  </div>
                  <div
                    style={{
                      fontSize: 16,
                      fontWeight: 700,
                      color: "var(--bp-accent)",
                      marginTop: 8,
                    }}
                  >
                    {pkg.tiers[selectedTier[pkg.id] as keyof typeof pkg.tiers]?.price}
                  </div>
                </div>

                {/* Best Fit */}
                <div style={{ marginBottom: 24 }}>
                  <div
                    style={{
                      fontSize: 12,
                      color: "var(--bp-dim)",
                      marginBottom: 8,
                      fontWeight: 600,
                    }}
                  >
                    BEST FIT
                  </div>
                  <p
                    style={{
                      fontSize: 12,
                      color: "var(--bp-text)",
                      lineHeight: "16px",
                    }}
                  >
                    {pkg.bestFit}
                  </p>
                </div>

                {/* Included */}
                <div style={{ marginBottom: 24 }}>
                  <div
                    style={{
                      fontSize: 12,
                      color: "var(--bp-dim)",
                      marginBottom: 8,
                      fontWeight: 600,
                    }}
                  >
                    INCLUDED
                  </div>
                  <ul style={{ margin: 0, paddingLeft: 16, listStyle: "disc" }}>
                    {pkg.included.map((item, i) => (
                      <li
                        key={i}
                        style={{
                          fontSize: 12,
                          color: "var(--bp-text)",
                          lineHeight: "16px",
                          marginBottom: 4,
                        }}
                      >
                        {item}
                      </li>
                    ))}
                  </ul>
                </div>

                {/* CTA */}
                <button
                  onClick={() => handleBookSprint(pkg.id, selectedTier[pkg.id])}
                  style={{
                    width: "100%",
                    fontSize: 14,
                    fontWeight: 600,
                    color: "#ffffff",
                    background: "var(--bp-accent)",
                    padding: "16px 24px",
                    borderRadius: 6,
                    border: "none",
                    cursor: "pointer",
                    transition: "opacity 0.2s",
                  }}
                  onMouseOver={(e) => (e.currentTarget.style.opacity = "0.8")}
                  onMouseOut={(e) => (e.currentTarget.style.opacity = "1")}
                >
                  Book This Sprint →
                </button>

                {/* Example Projects */}
                <div style={{ marginTop: 20 }}>
                  <div
                    style={{
                      fontSize: 11,
                      color: "var(--bp-dim)",
                      marginBottom: 8,
                      fontWeight: 600,
                    }}
                  >
                    EXAMPLE PROJECTS
                  </div>
                  <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                    {pkg.exampleProjects.map((project, i) => (
                      <span
                        key={i}
                        style={{
                          fontSize: 11,
                          color: "var(--bp-text)",
                          background: "var(--bp-border)",
                          padding: "4px 8px",
                          borderRadius: 4,
                        }}
                      >
                        {project}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Comparison Table */}
          <div style={{ marginBottom: 60 }}>
            <h2
              style={{
                fontSize: 24,
                fontWeight: 700,
                color: "#ffffff",
                textAlign: "center",
                marginBottom: 32,
              }}
            >
              Package Comparison
            </h2>
            <div
              style={{
                background: "var(--bp-surface)",
                border: "1px solid var(--bp-border)",
                borderRadius: 12,
                overflow: "hidden",
              }}
            >
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr style={{ background: "var(--bp-border)" }}>
                    <th
                      style={{
                        padding: 16,
                        textAlign: "left",
                        fontSize: 12,
                        fontWeight: 600,
                        color: "var(--bp-dim)",
                      }}
                    >
                      FEATURE
                    </th>
                    {SERVICE_PACKAGES.map((pkg) => (
                      <th
                        key={pkg.id}
                        style={{
                          padding: 16,
                          textAlign: "center",
                          fontSize: 14,
                          fontWeight: 600,
                          color: "#ffffff",
                        }}
                      >
                        {pkg.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {[
                    { label: "Timeline", values: SERVICE_PACKAGES.map(p => p.timeline) },
                    { label: "Price Range", values: SERVICE_PACKAGES.map(p => p.priceRange) },
                    { label: "Revisions", values: ["1 round", "Basic", "2 rounds"] },
                    { label: "Hosting Setup", values: ["Guide only", "Not included", "Full setup"] },
                    { label: "Security Audit", values: ["Basic", "Not included", "Full audit"] },
                    { label: "Monitoring", values: ["Not included", "Basic alerts", "Full monitoring"] },
                    { label: "Launch Support", values: ["Not included", "Not included", "30 days"] },
                  ].map((row, i) => (
                    <tr
                      key={i}
                      style={{
                        borderTop: i > 0 ? "1px solid var(--bp-border)" : "none",
                      }}
                    >
                      <td
                        style={{
                          padding: 16,
                          fontSize: 12,
                          fontWeight: 600,
                          color: "var(--bp-dim)",
                        }}
                      >
                        {row.label}
                      </td>
                      {row.values.map((value, j) => (
                        <td
                          key={j}
                          style={{
                            padding: 16,
                            textAlign: "center",
                            fontSize: 12,
                            color: "var(--bp-text)",
                          }}
                        >
                          {value}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Qualification Flow */}
          <div
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 12,
              padding: 40,
              textAlign: "center",
            }}
          >
            <h3
              style={{
                fontSize: 20,
                fontWeight: 700,
                color: "#ffffff",
                marginBottom: 16,
              }}
            >
              Not Sure Which Package Fits?
            </h3>
            <p
              style={{
                fontSize: 14,
                color: "var(--bp-text)",
                lineHeight: "20px",
                marginBottom: 24,
                maxWidth: 500,
                margin: "0 auto 24px",
              }}
            >
              Tell Ocean your project idea, timeline, and budget. Get a recommendation
              with detailed scope and accurate estimate.
            </p>
            <Link
              href="/hire"
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
              Get Custom Recommendation →
            </Link>
          </div>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}