import type { Metadata } from "next";
import Link from "next/link";

import { projects } from "@/data/projects";
import { ProjectCard } from "@/components/ProjectCard";
import { RegMarks } from "@/components/RegMarks";
import { BackgroundCanvas } from "@/components/BackgroundCanvas";

export const metadata: Metadata = {
  title: "Projects — repo.box",
  description:
    "What the repo.box studio is building and running right now. Current work only, every entry with something you can open.",
  alternates: { canonical: "https://repo.box/projects" },
};

const navLinkStyle = { color: "var(--bp-dim)", fontSize: 12 } as const;

// The one place the site lists what we work on. /portfolio, /building and
// /projects/<slug> used to be separate views of overlapping data and now
// redirect here (see next.config.ts). Retired work is not listed; /proof
// carries the case studies.
export default function ProjectsPage() {
  return (
    <>
      <RegMarks />
      <div
        style={{ maxWidth: 720, margin: "0 auto", position: "relative", zIndex: 2, padding: "80px 40px 100px" }}
      >
        <header style={{ marginBottom: 48 }}>
          <nav style={{ display: "flex", gap: 16, marginBottom: 24 }}>
            <Link href="/" style={navLinkStyle}>← home</Link>
            <Link href="/proof" style={navLinkStyle}>proof</Link>
            <Link href="/blog/" style={navLinkStyle}>blog</Link>
            <Link href="/hire" style={navLinkStyle}>hire us</Link>
          </nav>
          <h1 className="font-mono font-bold" style={{ fontSize: 36, lineHeight: 1.1, margin: "0 0 16px" }}>
            Projects
          </h1>
          <p style={{ fontSize: 16, lineHeight: "24px", color: "var(--bp-dim)", maxWidth: 520, margin: 0 }}>
            What the studio is building and running right now. Every entry has something you can
            open; work we have stopped is not listed here.
          </p>
        </header>

        <main>
          {projects.map((project) => (
            <ProjectCard key={project.id} project={project} showTags />
          ))}
        </main>

        <p style={{ fontSize: 12, lineHeight: "20px", color: "var(--bp-dim)", marginTop: 40 }}>
          Looking for the history?{" "}
          <Link href="/proof" style={{ color: "var(--bp-accent2)" }}>
            The proof page
          </Link>{" "}
          keeps the case studies, including the ones we retired.
        </p>
      </div>
      <BackgroundCanvas />
    </>
  );
}
