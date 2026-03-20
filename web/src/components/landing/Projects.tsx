"use client";

import { useEffect, useRef } from "react";

interface ProjectCardProps {
  title: string;
  tag: string;
  description: string;
  href?: string;
}

function ProjectCard({ title, tag, description, href }: ProjectCardProps) {
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
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "baseline",
            marginBottom: 4,
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
            {title}
          </div>
          <span
            style={{
              fontSize: 12,
              lineHeight: "20px",
              color: "var(--bp-accent)",
              background: "rgba(79,195,247,0.15)",
              padding: "0 12px",
              borderRadius: 2,
              fontWeight: 600,
            }}
          >
            {tag}
          </span>
        </div>
        <div
          style={{
            fontSize: 12,
            lineHeight: "20px",
            color: "var(--bp-text)",
          }}
        >
          {description}
        </div>
      </div>
    </div>
  );

  if (href) {
    return (
      <a
        href={href}
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

export function LandingProjects() {
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
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 60 }}>
      <h2
        style={{
          fontSize: 12,
          lineHeight: "20px",
          textTransform: "uppercase",
          letterSpacing: "0.12em",
          color: "var(--bp-dim)",
          fontWeight: 500,
          marginBottom: 20,
        }}
      >
        What repo.box Does
      </h2>

      <ProjectCard
        title="Permission Layer"
        tag="git shim"
        description="Intercepts git commands. Every commit, merge, push is checked against .repobox.yml before it happens."
      />

      <ProjectCard
        title="EVM Identities"
        tag="crypto"
        description="Agents sign commits with secp256k1 keys. No SSH keys, no GPG complexity. Just wallet addresses."
      />

      <ProjectCard
        title="Sandbox by Default"
        tag="security"
        description="Agents get feature branches. They can't touch main, can't edit the config. Structurally impossible to escalate."
      />
    </section>
  );
}
