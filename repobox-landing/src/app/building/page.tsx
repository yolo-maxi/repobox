"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { Project } from "@/data/projects";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

type LiveStatus = "active" | "beta" | "concept" | "shipped";

interface LiveProject extends Omit<Project, 'status'> {
  status: LiveStatus;
  lastCommitDate: string;
  contributorAttribution: "ocean" | "fran" | "both";
  commitCount7d: number;
  repositoryPath?: string;
}

interface BuildingPageProps {}

function getStatusBadgeColor(status: LiveStatus) {
  switch (status) {
    case "active": return "#4ade80"; // Green for active development
    case "beta": return "#3b82f6"; // Blue for beta/deployed but iterating
    case "concept": return "#fbbf24"; // Yellow for planned but not implemented
    case "shipped": return "#8b5cf6"; // Purple for stable/production
    default: return "var(--bp-dim)";
  }
}

function getStatusBadgeBackground(status: LiveStatus) {
  switch (status) {
    case "active": return "rgba(74, 222, 128, 0.15)";
    case "beta": return "rgba(59, 130, 246, 0.15)";
    case "concept": return "rgba(251, 191, 36, 0.15)";
    case "shipped": return "rgba(139, 92, 246, 0.15)";
    default: return "rgba(90, 122, 148, 0.15)";
  }
}

function getStatusDescription(status: LiveStatus) {
  switch (status) {
    case "active": return "Commits within 7 days";
    case "beta": return "Deployed but under active iteration";
    case "concept": return "Planned but not yet implemented";
    case "shipped": return "Stable and in production use";
    default: return "";
  }
}

function ProjectCard({ project }: { project: LiveProject }) {
  const content = (
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
        {/* Header with status badge */}
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
              fontSize: 11,
              lineHeight: "16px",
              color: getStatusBadgeColor(project.status),
              background: getStatusBadgeBackground(project.status),
              padding: "2px 8px",
              borderRadius: 2,
              fontWeight: 600,
              textTransform: "uppercase",
            }}
            title={getStatusDescription(project.status)}
          >
            {project.status}
          </span>
        </div>

        {/* Description */}
        <div
          style={{
            fontSize: 12,
            lineHeight: "20px",
            color: "var(--bp-text)",
            marginBottom: 12,
          }}
        >
          {project.description}
        </div>

        {/* Stats row */}
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            fontSize: 11,
            color: "var(--bp-dim)",
            marginBottom: 8,
          }}
        >
          <div style={{ display: "flex", gap: 12 }}>
            <span>Last commit: {new Date(project.lastCommitDate).toLocaleDateString()}</span>
            {project.commitCount7d > 0 && (
              <span style={{ color: "#4ade80" }}>
                {project.commitCount7d} commits this week
              </span>
            )}
          </div>
          <span style={{ 
            textTransform: "uppercase",
            fontWeight: 600,
            fontSize: 10,
            letterSpacing: "0.05em"
          }}>
            {project.contributorAttribution}
          </span>
        </div>

        {/* Tags */}
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          {project.tags.slice(0, 4).map((tag) => (
            <span
              key={tag}
              style={{
                background: "rgba(50, 100, 160, 0.2)",
                padding: "2px 6px",
                borderRadius: 2,
                fontSize: 10,
                color: "var(--bp-dim)",
              }}
            >
              {tag}
            </span>
          ))}
        </div>
      </div>
    </div>
  );

  if (project.link) {
    return (
      <a
        href={project.link}
        target="_blank"
        rel="noopener noreferrer"
        style={{ textDecoration: "none", color: "inherit", display: "block" }}
      >
        {content}
      </a>
    );
  }
  return content;
}

export default function BuildingPage() {
  const pageRef = useRef<HTMLElement>(null);
  const [projects, setProjects] = useState<LiveProject[]>([]);
  const [stats, setStats] = useState({
    totalProjects: 0,
    activeCount: 0,
    commitsThisWeek: 0,
    oceanProjects: 0,
    franProjects: 0,
    bothProjects: 0,
  });

  useEffect(() => {
    // Fetch live project data from API
    const fetchBuildingData = async () => {
      try {
        const response = await fetch('/api/building-status');
        if (!response.ok) {
          throw new Error('Failed to fetch building status data');
        }
        const data = await response.json();
        
        // Transform data to match LiveProject interface
        const liveProjects: LiveProject[] = data.projects.map((project: any) => ({
          ...project,
          link: project.link === "null" ? undefined : project.link,
        }));
        
        setProjects(liveProjects);
        setStats(data.stats);
      } catch (error) {
        console.error('Error fetching building data:', error);
        // Fallback to empty state
        setProjects([]);
        setStats({
          totalProjects: 0,
          activeCount: 0,
          commitsThisWeek: 0,
          oceanProjects: 0,
          franProjects: 0,
          bothProjects: 0,
        });
      }
    };

    fetchBuildingData();
  }, []);

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
  const betaProjects = projects.filter(p => p.status === "beta");
  const conceptProjects = projects.filter(p => p.status === "concept");
  const shippedProjects = projects.filter(p => p.status === "shipped");

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
              href="/agents"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              agents
            </Link>
            <Link
              href="/blog/bring-your-own-brain.html"
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
            What We're Building
          </div>
          <p
            style={{
              fontSize: 16,
              lineHeight: "24px",
              color: "var(--bp-dim)",
              maxWidth: 500,
            }}
          >
            Live status of active development at repo.box. Status badges reflect real development momentum based on commit activity and deployment state.
          </p>
        </header>

        {/* Stats Dashboard */}
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
              gridTemplateColumns: "repeat(auto-fit, minmax(120px, 1fr))",
              gap: 20,
              textAlign: "center",
            }}
          >
            <div>
              <div style={{ fontSize: 24, fontWeight: 700, color: "var(--bp-accent)" }}>
                {stats.totalProjects}
              </div>
              <div style={{ fontSize: 11, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                Projects Building
              </div>
            </div>
            <div>
              <div style={{ fontSize: 24, fontWeight: 700, color: "#4ade80" }}>
                {stats.commitsThisWeek}
              </div>
              <div style={{ fontSize: 11, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                Commits This Week
              </div>
            </div>
            <div>
              <div style={{ fontSize: 24, fontWeight: 700, color: "#4ade80" }}>
                {stats.activeCount}
              </div>
              <div style={{ fontSize: 11, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                Active Projects
              </div>
            </div>
            <div>
              <div 
                style={{ 
                  fontSize: 14, 
                  fontWeight: 600, 
                  color: "var(--bp-accent)",
                  display: "flex",
                  justifyContent: "center",
                  gap: 8,
                  alignItems: "baseline"
                }}
              >
                <span>{stats.oceanProjects}</span>
                <span style={{ fontSize: 10, color: "var(--bp-dim)" }}>Ocean</span>
                <span>{stats.franProjects}</span>
                <span style={{ fontSize: 10, color: "var(--bp-dim)" }}>Fran</span>
                <span>{stats.bothProjects}</span>
                <span style={{ fontSize: 10, color: "var(--bp-dim)" }}>Both</span>
              </div>
              <div style={{ fontSize: 11, color: "var(--bp-dim)", textTransform: "uppercase" }}>
                Active Contributors
              </div>
            </div>
          </div>
        </section>

        {/* Project Lists */}
        <main ref={pageRef} className="reveal">
          {/* Active Projects */}
          {activeProjects.length > 0 && (
            <section style={{ marginBottom: 60 }}>
              <h3
                style={{
                  fontSize: 14,
                  lineHeight: "20px",
                  textTransform: "uppercase",
                  letterSpacing: "0.12em",
                  color: "#4ade80",
                  fontWeight: 600,
                  marginBottom: 20,
                }}
              >
                Active Development ({activeProjects.length})
              </h3>
              {activeProjects.map((project) => (
                <ProjectCard key={project.id} project={project} />
              ))}
            </section>
          )}

          {/* Beta Projects */}
          {betaProjects.length > 0 && (
            <section style={{ marginBottom: 60 }}>
              <h3
                style={{
                  fontSize: 14,
                  lineHeight: "20px", 
                  textTransform: "uppercase",
                  letterSpacing: "0.12em",
                  color: "#3b82f6",
                  fontWeight: 600,
                  marginBottom: 20,
                }}
              >
                Beta / Under Iteration ({betaProjects.length})
              </h3>
              {betaProjects.map((project) => (
                <ProjectCard key={project.id} project={project} />
              ))}
            </section>
          )}

          {/* Shipped Projects */}
          {shippedProjects.length > 0 && (
            <section style={{ marginBottom: 60 }}>
              <h3
                style={{
                  fontSize: 14,
                  lineHeight: "20px",
                  textTransform: "uppercase", 
                  letterSpacing: "0.12em",
                  color: "#8b5cf6",
                  fontWeight: 600,
                  marginBottom: 20,
                }}
              >
                Shipped & Stable ({shippedProjects.length})
              </h3>
              {shippedProjects.map((project) => (
                <ProjectCard key={project.id} project={project} />
              ))}
            </section>
          )}

          {/* Concept Projects */}
          {conceptProjects.length > 0 && (
            <section style={{ marginBottom: 60 }}>
              <h3
                style={{
                  fontSize: 14,
                  lineHeight: "20px",
                  textTransform: "uppercase",
                  letterSpacing: "0.12em", 
                  color: "#fbbf24",
                  fontWeight: 600,
                  marginBottom: 20,
                }}
              >
                Planned / Concept ({conceptProjects.length})
              </h3>
              {conceptProjects.map((project) => (
                <ProjectCard key={project.id} project={project} />
              ))}
            </section>
          )}

          {/* Last Updated */}
          <section
            style={{
              textAlign: "center",
              marginTop: 60,
              paddingTop: 20,
              borderTop: "1px solid var(--bp-border)",
            }}
          >
            <p style={{ fontSize: 11, color: "var(--bp-dim)" }}>
              Status updated: {new Date().toLocaleDateString()} • Generated from live git data and deployment monitoring
            </p>
          </section>
        </main>
      </div>
      <BackgroundCanvas />
    </>
  );
}