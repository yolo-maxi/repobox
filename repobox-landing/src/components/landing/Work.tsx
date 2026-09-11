"use client";

import Link from "next/link";

import { projects } from "@/data/projects";
import { ProjectCard } from "@/components/ProjectCard";
import { useReveal } from "./useReveal";
import { sectionTitleStyle } from "./shared";

// Renders the same registry as /projects (src/data/projects.ts), so the
// homepage cannot drift from the projects page.
export function LandingWork() {
  const sectionRef = useReveal<HTMLElement>();

  return (
    <section id="work" ref={sectionRef} className="reveal" style={{ marginBottom: 72 }}>
      <h2 style={sectionTitleStyle}>What we&apos;re building</h2>

      <p
        style={{
          fontSize: 14,
          lineHeight: "23px",
          color: "var(--bp-dim)",
          maxWidth: 560,
          margin: "0 0 24px",
        }}
      >
        All of it open to inspection. We would rather you clicked through and
        formed your own opinion than read an adjective about it.
      </p>

      {projects.map((project) => (
        <ProjectCard key={project.id} project={project} />
      ))}

      <p style={{ fontSize: 12, lineHeight: "20px", color: "var(--bp-dim)", margin: "8px 0 0" }}>
        <Link href="/projects" style={{ color: "var(--bp-accent2)" }}>
          All current projects →
        </Link>
      </p>
    </section>
  );
}
