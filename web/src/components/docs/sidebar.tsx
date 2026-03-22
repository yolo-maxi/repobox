"use client";

import Link from "next/link";
import { useState } from "react";
import type { Manifest } from "@/lib/docs";

export function DocsSidebar({
  manifest,
  currentSlug,
}: {
  manifest: Manifest;
  currentSlug: string;
}) {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <>
      {/* Mobile toggle */}
      <button
        className="docs-mobile-toggle"
        onClick={() => setMobileOpen(!mobileOpen)}
        aria-label="Toggle navigation"
      >
        {mobileOpen ? "✕" : "☰"} Docs
      </button>

      <nav className={`docs-sidebar ${mobileOpen ? "open" : ""}`}>
        <div className="docs-sidebar-header">
          <span className="docs-label">Documentation</span>
        </div>

        {manifest.sections.map((section) => (
          <div key={section.title} className="docs-nav-section">
            <div className="docs-nav-heading">{section.title}</div>
            {section.items.map((item) => (
              <Link
                key={item.slug}
                href={`/docs/${item.slug}`}
                className={`docs-nav-item ${
                  currentSlug === item.slug ? "active" : ""
                }`}
                onClick={() => setMobileOpen(false)}
              >
                {item.title}
              </Link>
            ))}
          </div>
        ))}

        <div className="docs-nav-section" style={{ marginTop: "auto" }}>
          <Link href="/blog" className="docs-nav-item">
            ← Blog
          </Link>
        </div>
      </nav>
    </>
  );
}
