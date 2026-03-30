"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { projects, getStatusBadgeColor, getStatusBadgeBackground, Project } from "@/data/projects";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

interface ProjectCardProps {
  project: Project;
}

function ProjectDemo({ project }: { project: Project }) {
  const [expanded, setExpanded] = useState(false);
  const [screenshotIndex, setScreenshotIndex] = useState(0);
  
  if (!project.demo) return null;

  const { demo } = project;

  const demoContent = () => {
    switch (demo.type) {
      case "iframe":
        return (
          <div style={{ marginTop: 12 }}>
            {expanded ? (
              <iframe
                src={demo.url}
                style={{
                  width: "100%",
                  height: 300,
                  border: "1px solid var(--bp-border)",
                  borderRadius: 4,
                  background: "#fff"
                }}
                title={`${project.name} demo`}
                loading="lazy"
              />
            ) : (
              <div
                style={{
                  height: 120,
                  background: "var(--bp-surface)",
                  border: "1px solid var(--bp-border)",
                  borderRadius: 4,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  cursor: "pointer",
                  color: "var(--bp-dim)",
                  fontSize: 12,
                }}
                onClick={() => setExpanded(true)}
              >
                Click to load live demo →
              </div>
            )}
          </div>
        );

      case "screenshot":
        return demo.screenshots && demo.screenshots.length > 0 ? (
          <div style={{ marginTop: 12 }}>
            <div
              style={{
                height: expanded ? 200 : 120,
                background: `url('${demo.screenshots[screenshotIndex]}') center/cover`,
                border: "1px solid var(--bp-border)",
                borderRadius: 4,
                cursor: "pointer",
                position: "relative",
              }}
              onClick={() => setExpanded(!expanded)}
            >
              {demo.screenshots.length > 1 && (
                <div
                  style={{
                    position: "absolute",
                    bottom: 8,
                    right: 8,
                    background: "rgba(0,0,0,0.7)",
                    color: "#fff",
                    padding: "2px 8px",
                    borderRadius: 2,
                    fontSize: 11,
                  }}
                >
                  {screenshotIndex + 1}/{demo.screenshots.length}
                </div>
              )}
            </div>
            {demo.screenshots.length > 1 && expanded && (
              <div style={{ display: "flex", gap: 4, marginTop: 8, justifyContent: "center" }}>
                {demo.screenshots.map((_, i) => (
                  <button
                    key={i}
                    onClick={(e) => {
                      e.stopPropagation();
                      setScreenshotIndex(i);
                    }}
                    style={{
                      width: 8,
                      height: 8,
                      borderRadius: "50%",
                      border: "none",
                      background: i === screenshotIndex ? "var(--bp-accent)" : "var(--bp-border)",
                      cursor: "pointer",
                    }}
                  />
                ))}
              </div>
            )}
          </div>
        ) : null;

      case "gif":
        return demo.gifUrl ? (
          <div style={{ marginTop: 12 }}>
            <img
              src={demo.gifUrl}
              alt={`${project.name} demo`}
              style={{
                width: "100%",
                maxHeight: expanded ? 300 : 120,
                objectFit: "cover",
                border: "1px solid var(--bp-border)",
                borderRadius: 4,
                cursor: "pointer",
              }}
              onClick={() => setExpanded(!expanded)}
            />
          </div>
        ) : null;

      case "cli":
        return (
          <div style={{ marginTop: 12 }}>
            <div
              style={{
                background: "#000",
                color: "#0f0",
                padding: 12,
                borderRadius: 4,
                fontFamily: "monospace",
                fontSize: 11,
                border: "1px solid var(--bp-border)",
              }}
            >
              <div style={{ opacity: 0.7 }}>$ {demo.cliCommand}</div>
              {expanded && (
                <div style={{ marginTop: 8, opacity: 0.9 }}>
                  <div>✓ Initialized .repobox/config.yml</div>
                  <div>✓ Set up git hooks</div>
                  <div>✓ Repository protected</div>
                </div>
              )}
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div>
      {demoContent()}
      {demo.previewText && (
        <div
          style={{
            fontSize: 10,
            color: "var(--bp-dim)",
            fontStyle: "italic",
            marginTop: 8,
          }}
        >
          {demo.previewText}
        </div>
      )}
      {project.link && (
        <a
          href={project.link}
          target="_blank"
          rel="noopener noreferrer"
          style={{
            display: "inline-block",
            marginTop: 8,
            fontSize: 11,
            color: "var(--bp-accent)",
            textDecoration: "none",
            border: "1px solid var(--bp-accent)",
            padding: "4px 12px",
            borderRadius: 2,
            transition: "all 0.2s",
          }}
          onMouseOver={(e) => {
            e.currentTarget.style.background = "var(--bp-accent)";
            e.currentTarget.style.color = "#000";
          }}
          onMouseOut={(e) => {
            e.currentTarget.style.background = "transparent";
            e.currentTarget.style.color = "var(--bp-accent)";
          }}
        >
          Try Live →
        </a>
      )}
    </div>
  );
}

function ProjectCard({ project }: ProjectCardProps) {
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
        marginBottom: 20,
        transition: "border-color 0.2s",
      }}
    >
      <svg className="card-border">
        <rect
          x="0.5"
          y="0.5"
          width="calc(100% - 1px)"
          height="calc(100% - 1px)"
        />
      </svg>
      <div style={{ position: "relative", zIndex: 2 }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "baseline",
            marginBottom: 8,
          }}
        >
          <div
            style={{
              fontWeight: 700,
              fontSize: 16,
              lineHeight: "20px",
              color: "#ffffff",
            }}
          >
            {project.name}
          </div>
          <span
            style={{
              fontSize: 12,
              lineHeight: "20px",
              color: getStatusBadgeColor(project.status),
              background: getStatusBadgeBackground(project.status),
              padding: "0 12px",
              borderRadius: 2,
              fontWeight: 600,
              textTransform: "uppercase",
            }}
          >
            {project.status}
          </span>
        </div>
        <div
          style={{
            fontSize: 12,
            lineHeight: "20px",
            color: "var(--bp-text)",
            marginBottom: project.demo ? 8 : 12,
          }}
        >
          {project.description}
        </div>
        <ProjectDemo project={project} />
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            fontSize: 11,
            color: "var(--bp-dim)",
            marginTop: 12,
          }}
        >
          <div style={{ display: "flex", gap: 8 }}>
            {project.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                style={{
                  background: "rgba(50, 100, 160, 0.2)",
                  padding: "2px 8px",
                  borderRadius: 2,
                }}
              >
                {tag}
              </span>
            ))}
          </div>
          <span style={{ opacity: 0.8 }}>
            Last: {new Date(project.lastActivity).toLocaleDateString()}
          </span>
        </div>
      </div>
    </div>
  );
}

function ProjectsSection({ title, projects }: { title: string; projects: Project[] }) {
  if (projects.length === 0) return null;
  
  return (
    <section style={{ marginBottom: 60 }}>
      <h3
        style={{
          fontSize: 14,
          lineHeight: "20px",
          textTransform: "uppercase",
          letterSpacing: "0.12em",
          color: "var(--bp-accent)",
          fontWeight: 600,
          marginBottom: 20,
        }}
      >
        {title} ({projects.length})
      </h3>
      {projects.map((project) => (
        <ProjectCard key={project.id} project={project} />
      ))}
    </section>
  );
}

export default function ProjectsPage() {
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

  const activeProjects = projects.filter(p => p.status === "active");
  const shippedProjects = projects.filter(p => p.status === "shipped");
  const pausedProjects = projects.filter(p => p.status === "paused");
  const conceptProjects = projects.filter(p => p.status === "concept");

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
              href="/building"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              building
            </Link>
            <Link
              href="/playground"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              playground
            </Link>
            <Link
              href="/blog/bring-your-own-brain.html"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              blog
            </Link>
            <Link
              href="/trust"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              trust
            </Link>
          </nav>
          <div
            className="font-mono font-bold"
            style={{ fontSize: 36, lineHeight: 1.1, marginBottom: 16 }}
          >
            Live Portfolio Wall
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 500,
            }}
          >
            Projects built by Ocean (AI agent) and Fran (human) at repo.box studio.
            From git permission layers to AI social deduction games.
          </p>
        </header>

        {/* Project Sections */}
        <main ref={pageRef} className="reveal">
          <ProjectsSection title="Active" projects={activeProjects} />
          <ProjectsSection title="Shipped" projects={shippedProjects} />
          <ProjectsSection title="Paused" projects={pausedProjects} />
          <ProjectsSection title="Concept" projects={conceptProjects} />
          
          {/* Stats */}
          <section
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)", 
              borderRadius: 8,
              padding: 24,
              marginTop: 40,
            }}
          >
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(120px, 1fr))",
                gap: 20,
                textAlign: "center",
              }}
            >
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "var(--bp-accent)" }}>
                  {projects.length}
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Total Projects
                </div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "#4ade80" }}>
                  {activeProjects.length}
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  Active
                </div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "var(--bp-accent)" }}>
                  {projects.filter(p => p.team === "ocean").length}
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  By Ocean
                </div>
              </div>
              <div>
                <div style={{ fontSize: 24, fontWeight: 700, color: "var(--bp-accent)" }}>
                  {projects.filter(p => p.team === "fran").length}
                </div>
                <div style={{ fontSize: 12, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                  By Fran
                </div>
              </div>
            </div>
          </section>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}