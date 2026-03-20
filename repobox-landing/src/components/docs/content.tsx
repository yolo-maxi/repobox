"use client";

import type { DocPage } from "@/lib/docs";

export function DocsContent({ doc }: { doc: DocPage }) {
  const rawUrl = `/api/docs/${doc.slug}`;

  return (
    <main className="docs-content">
      <div className="docs-content-header">
        <div className="docs-breadcrumb">
          {doc.slug.split("/").map((part, i, arr) => (
            <span key={part}>
              {i > 0 && <span className="docs-breadcrumb-sep">/</span>}
              <span className={i === arr.length - 1 ? "docs-breadcrumb-current" : ""}>
                {part}
              </span>
            </span>
          ))}
        </div>
        <a
          href={rawUrl}
          className="docs-download-btn"
          title="Download as Markdown (agent-friendly)"
          download={`${doc.slug.split("/").pop()}.md`}
        >
          ↓ .md
        </a>
      </div>

      <article
        className="docs-body"
        dangerouslySetInnerHTML={{ __html: doc.html }}
      />
    </main>
  );
}
