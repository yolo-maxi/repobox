"use client";

import Link from "next/link";

import { CardBorder } from "@/components/landing/shared";
import { STATUS_LABEL, type Project } from "@/data/projects";

const linkStyle = {
  fontFamily: "var(--font-mono), monospace",
  fontSize: 12,
  lineHeight: "20px",
  color: "var(--bp-accent2)",
} as const;

function StatusPill({ status }: { status: Project["status"] }) {
  return (
    <span
      style={{
        fontSize: 11,
        lineHeight: "20px",
        color: "var(--bp-accent)",
        background: "rgba(79,195,247,0.15)",
        padding: "0 10px",
        borderRadius: 2,
        fontWeight: 600,
        whiteSpace: "nowrap",
        textTransform: "uppercase",
      }}
    >
      {STATUS_LABEL[status]}
    </span>
  );
}

/** One project, as shown on the homepage and on /projects. */
export function ProjectCard({ project, showTags = false }: { project: Project; showTags?: boolean }) {
  return (
    <div
      className="project-card"
      style={{
        position: "relative",
        overflow: "hidden",
        background: "var(--bp-surface)",
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
        padding: 20,
        marginBottom: 16,
        transition: "border-color 0.2s",
      }}
    >
      <CardBorder />
      <div style={{ position: "relative", zIndex: 2 }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "baseline",
            gap: 12,
            marginBottom: 8,
          }}
        >
          <div style={{ fontWeight: 700, fontSize: 16, lineHeight: "22px", color: "#ffffff" }}>
            {project.name}
          </div>
          <StatusPill status={project.status} />
        </div>

        <p style={{ fontSize: 13, lineHeight: "21px", color: "var(--bp-text)", margin: "0 0 14px" }}>
          {project.summary}
        </p>

        <div style={{ display: "flex", gap: 14, flexWrap: "wrap", alignItems: "center" }}>
          {project.links.map((link) =>
            link.href.startsWith("http") ? (
              <a key={link.href} href={link.href} target="_blank" rel="noopener noreferrer" style={linkStyle}>
                {link.label} →
              </a>
            ) : (
              <Link key={link.href} href={link.href} style={linkStyle}>
                {link.label} →
              </Link>
            )
          )}
          {showTags && (
            <span style={{ display: "flex", gap: 8, marginLeft: "auto" }}>
              {project.tags.map((tag) => (
                <span
                  key={tag}
                  style={{
                    fontSize: 11,
                    lineHeight: "20px",
                    color: "var(--bp-dim)",
                    background: "rgba(50, 100, 160, 0.2)",
                    padding: "0 8px",
                    borderRadius: 2,
                  }}
                >
                  {tag}
                </span>
              ))}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
