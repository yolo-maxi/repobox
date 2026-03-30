"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

// Portfolio data interface (matches generate-portfolio-data.js output)
interface PortfolioProject {
  id: string;
  name: string;
  description: string;
  status: "active" | "shipped" | "beta" | "concept";
  link?: string | null;
  lastCommit?: string | null;
  totalCommits: number;
  weeklyCommits: number;
  tags: string[];
}

interface PortfolioStats {
  totalProjects: number;
  activeProjects: number;
  shippedProjects: number;
  totalCommits: number;
  weeklyCommits: number;
  lastGenerated: string;
}

interface PortfolioData {
  stats: PortfolioStats;
  projects: PortfolioProject[];
}

// Status badge styling
function getStatusBadgeColor(status: PortfolioProject["status"]) {
  switch (status) {
    case "active": return "#4ade80"; // Green for active (recent commits)
    case "shipped": return "var(--bp-accent)"; // Blue for shipped/stable
    case "beta": return "#fbbf24"; // Yellow for beta/iterating
    case "concept": return "var(--bp-dim)"; // Gray for concept
    default: return "var(--bp-dim)";
  }
}

function getStatusBadgeBackground(status: PortfolioProject["status"]) {
  switch (status) {
    case "active": return "rgba(74, 222, 128, 0.15)";
    case "shipped": return "rgba(79, 195, 247, 0.15)"; 
    case "beta": return "rgba(251, 191, 36, 0.15)";
    case "concept": return "rgba(90, 122, 148, 0.15)";
    default: return "rgba(90, 122, 148, 0.15)";
  }
}

// Portfolio project card component
function PortfolioCard({ project }: { project: PortfolioProject }) {
  const lastCommitDate = project.lastCommit 
    ? new Date(project.lastCommit).toLocaleDateString() 
    : "No commits";
    
  const daysAgo = project.lastCommit 
    ? Math.floor((Date.now() - new Date(project.lastCommit).getTime()) / (1000 * 60 * 60 * 24))
    : null;
    
  const activityIndicator = daysAgo !== null 
    ? daysAgo === 0 ? "today" 
      : daysAgo === 1 ? "yesterday"
      : daysAgo < 7 ? `${daysAgo}d ago`
      : daysAgo < 30 ? `${Math.floor(daysAgo / 7)}w ago`
      : `${Math.floor(daysAgo / 30)}m ago`
    : "inactive";

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
        transition: "all 0.2s ease",
        cursor: project.link ? "pointer" : "default",
      }}
      onClick={() => project.link && window.open(project.link, '_blank')}
      onMouseOver={(e) => {
        if (project.link) {
          e.currentTarget.style.borderColor = "var(--bp-accent)";
          e.currentTarget.style.transform = "translateY(-2px)";
        }
      }}
      onMouseOut={(e) => {
        e.currentTarget.style.borderColor = "var(--bp-border)";
        e.currentTarget.style.transform = "translateY(0)";
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
        {/* Header with name and status */}
        <div style={{ 
          display: "flex", 
          justifyContent: "space-between", 
          alignItems: "flex-start", 
          marginBottom: 8 
        }}>
          <div style={{ 
            fontWeight: 700, 
            fontSize: 16, 
            lineHeight: "20px", 
            color: "#ffffff",
            flex: 1,
            marginRight: 12
          }}>
            {project.name}
            {project.link && (
              <span style={{ 
                fontSize: 12, 
                color: "var(--bp-dim)", 
                marginLeft: 8 
              }}>
                ↗
              </span>
            )}
          </div>
          <span style={{
            fontSize: 11,
            lineHeight: "18px",
            color: getStatusBadgeColor(project.status),
            background: getStatusBadgeBackground(project.status),
            padding: "2px 8px",
            borderRadius: 2,
            fontWeight: 600,
            textTransform: "uppercase",
            whiteSpace: "nowrap"
          }}>
            {project.status}
          </span>
        </div>

        {/* Description */}
        <div style={{
          fontSize: 12,
          lineHeight: "18px",
          color: "var(--bp-text)",
          marginBottom: 12,
          // Truncate long descriptions
          display: "-webkit-box",
          WebkitLineClamp: 2,
          WebkitBoxOrient: "vertical",
          overflow: "hidden"
        }}>
          {project.description}
        </div>

        {/* Activity stats */}
        <div style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          fontSize: 11,
          color: "var(--bp-dim)"
        }}>
          <div style={{ display: "flex", gap: 12 }}>
            <span>
              {project.totalCommits} commits
            </span>
            {project.weeklyCommits > 0 && (
              <span style={{ color: "#4ade80" }}>
                +{project.weeklyCommits} this week
              </span>
            )}
          </div>
          <span style={{ opacity: 0.8 }}>
            {activityIndicator}
          </span>
        </div>
      </div>
    </div>
  );
}

// Section component for grouping projects
function PortfolioSection({ 
  title, 
  projects, 
  description 
}: { 
  title: string; 
  projects: PortfolioProject[]; 
  description?: string;
}) {
  if (projects.length === 0) return null;
  
  return (
    <section style={{ marginBottom: 48 }}>
      <div style={{ marginBottom: 20 }}>
        <h3 style={{
          fontSize: 14,
          lineHeight: "20px",
          textTransform: "uppercase",
          letterSpacing: "0.12em",
          color: "var(--bp-accent)",
          fontWeight: 600,
          marginBottom: 4
        }}>
          {title} ({projects.length})
        </h3>
        {description && (
          <p style={{
            fontSize: 12,
            color: "var(--bp-dim)",
            margin: 0
          }}>
            {description}
          </p>
        )}
      </div>
      
      <div style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(300px, 1fr))",
        gap: 16
      }}>
        {projects.map((project) => (
          <PortfolioCard key={project.id} project={project} />
        ))}
      </div>
    </section>
  );
}

// Main portfolio page component
export default function PortfolioPage() {
  const pageRef = useRef<HTMLElement>(null);
  const [portfolioData, setPortfolioData] = useState<PortfolioData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load portfolio data
  useEffect(() => {
    fetch('/data/portfolio.json')
      .then(res => {
        if (!res.ok) throw new Error('Failed to load portfolio data');
        return res.json();
      })
      .then(data => {
        setPortfolioData(data);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  // Intersection observer for animations
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

  if (loading) {
    return (
      <div style={{ 
        maxWidth: 720, 
        margin: "0 auto", 
        padding: "80px 40px", 
        textAlign: "center",
        color: "var(--bp-dim)" 
      }}>
        Loading portfolio data...
      </div>
    );
  }

  if (error || !portfolioData) {
    return (
      <div style={{ 
        maxWidth: 720, 
        margin: "0 auto", 
        padding: "80px 40px", 
        textAlign: "center",
        color: "var(--bp-dim)" 
      }}>
        Error loading portfolio: {error || 'Unknown error'}
      </div>
    );
  }

  const { stats, projects } = portfolioData;
  
  // Group projects by status
  const activeProjects = projects.filter(p => p.status === "active");
  const shippedProjects = projects.filter(p => p.status === "shipped");
  const betaProjects = projects.filter(p => p.status === "beta");
  const conceptProjects = projects.filter(p => p.status === "concept");

  return (
    <>
      <RegMarks />
      <div style={{ 
        maxWidth: 900, 
        margin: "0 auto", 
        position: "relative", 
        zIndex: 2, 
        padding: "80px 40px 100px" 
      }}>
        {/* Header */}
        <header style={{ marginBottom: 60 }}>
          <nav style={{
            display: "flex",
            gap: 16,
            marginBottom: 24,
            fontSize: 12,
          }}>
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
              href="/building"
              style={{ color: "var(--bp-dim)", transition: "color 0.2s" }}
              onMouseOver={(e) => (e.currentTarget.style.color = "var(--bp-accent)")}
              onMouseOut={(e) => (e.currentTarget.style.color = "var(--bp-dim)")}
            >
              building
            </Link>
          </nav>
          
          <div className="font-mono font-bold" style={{ 
            fontSize: 36, 
            lineHeight: 1.1, 
            marginBottom: 16 
          }}>
            Live Portfolio Wall
          </div>
          
          <p style={{
            fontSize: 16,
            lineHeight: "24px",
            color: "var(--bp-dim)",
            maxWidth: 600,
            marginBottom: 32
          }}>
            Real-time project status from git activity. 
            Active = commits in last 7 days. Auto-generated from kanban data and repository activity.
          </p>
        </header>

        {/* Stats Summary */}
        <section style={{ 
          background: "var(--bp-surface)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: 24,
          marginBottom: 48
        }}>
          <div style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
            gap: 20,
            textAlign: "center"
          }}>
            <div>
              <div style={{ 
                fontSize: 28, 
                fontWeight: 700, 
                color: "var(--bp-accent)" 
              }}>
                {stats.totalProjects}
              </div>
              <div style={{ 
                fontSize: 12, 
                color: "var(--bp-dim)", 
                textTransform: "uppercase" 
              }}>
                Projects
              </div>
            </div>
            <div>
              <div style={{ 
                fontSize: 28, 
                fontWeight: 700, 
                color: "#4ade80" 
              }}>
                {stats.activeProjects}
              </div>
              <div style={{ 
                fontSize: 12, 
                color: "var(--bp-dim)", 
                textTransform: "uppercase" 
              }}>
                Active
              </div>
            </div>
            <div>
              <div style={{ 
                fontSize: 28, 
                fontWeight: 700, 
                color: "var(--bp-accent)" 
              }}>
                {stats.totalCommits}
              </div>
              <div style={{ 
                fontSize: 12, 
                color: "var(--bp-dim)", 
                textTransform: "uppercase" 
              }}>
                Total Commits
              </div>
            </div>
            <div>
              <div style={{ 
                fontSize: 28, 
                fontWeight: 700, 
                color: "#4ade80" 
              }}>
                {stats.weeklyCommits}
              </div>
              <div style={{ 
                fontSize: 12, 
                color: "var(--bp-dim)", 
                textTransform: "uppercase" 
              }}>
                This Week
              </div>
            </div>
          </div>
          
          <div style={{ 
            textAlign: "center", 
            marginTop: 16, 
            fontSize: 11, 
            color: "var(--bp-dim)" 
          }}>
            Last updated: {new Date(stats.lastGenerated).toLocaleString()}
          </div>
        </section>

        {/* Project Sections */}
        <main ref={pageRef} className="reveal">
          <PortfolioSection 
            title="Active Development" 
            projects={activeProjects}
            description="Projects with commits in the last 7 days"
          />
          
          <PortfolioSection 
            title="Shipped & Stable" 
            projects={shippedProjects}
            description="Production projects with established user base"
          />
          
          <PortfolioSection 
            title="Beta & Iterating" 
            projects={betaProjects}
            description="Deployed but under active development and testing"
          />
          
          <PortfolioSection 
            title="Concept & Planning" 
            projects={conceptProjects}
            description="Early stage projects and documented ideas"
          />
        </main>
        
        {/* Footer note */}
        <footer style={{ 
          marginTop: 60, 
          textAlign: "center",
          fontSize: 11,
          color: "var(--bp-dim)"
        }}>
          <p>
            Portfolio data auto-generated from kanban files and git repository activity.
            <br />
            Status badges reflect real development activity, not manual classification.
          </p>
        </footer>
      </div>
      <BackgroundCanvas />
    </>
  );
}