"use client";

import type { CSSProperties, ReactNode } from "react";

export const sectionTitleStyle: CSSProperties = {
  fontSize: 12,
  lineHeight: "20px",
  textTransform: "uppercase",
  letterSpacing: "0.12em",
  color: "var(--bp-dim)",
  fontWeight: 500,
  marginBottom: 20,
};

export function CardBorder() {
  return (
    <svg className="card-border" aria-hidden="true">
      <rect x="0.5" y="0.5" width="calc(100% - 1px)" height="calc(100% - 1px)" />
    </svg>
  );
}

export function ExternalAnchor({
  href,
  children,
}: {
  href: string;
  children: ReactNode;
}) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      style={{ textDecoration: "none", color: "inherit", display: "block" }}
    >
      {children}
    </a>
  );
}
