"use client";

import { useEffect, useRef } from "react";
import Link from "next/link";

interface FeedEntry {
  hash: string;
  author: string;
  message: string;
  time: string;
}

function truncateAddr(addr: string): string {
  if (addr.startsWith("0x") && addr.length > 12) {
    return addr.slice(0, 6) + "…" + addr.slice(-4);
  }
  return addr;
}

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diff = now - then;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export function LiveFeed({ entries }: { entries: FeedEntry[] }) {
  const sectionRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (es) => es.forEach((e) => { if (e.isIntersecting) e.target.classList.add("visible"); }),
      { threshold: 0.1 }
    );
    if (sectionRef.current) observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, []);

  if (!entries.length) return null;

  return (
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 40 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "baseline",
          marginBottom: 16,
        }}
      >
        <h2
          style={{
            fontSize: 14,
            lineHeight: "20px",
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-dim)",
            fontWeight: 500,
            margin: 0,
            display: "inline-flex",
            alignItems: "center",
            gap: 8,
          }}
        >
          <span
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              background: "#22c55e",
              animation: "pulse 2s infinite",
              flexShrink: 0,
            }}
          />
          Recent pushes
        </h2>
        <Link
          href="/explore"
          style={{
            fontFamily: "var(--font-mono), monospace",
            fontSize: 12,
            color: "var(--bp-accent)",
            textDecoration: "none",
          }}
        >
          view all →
        </Link>
      </div>

      <div
        style={{
          background: "rgba(0, 0, 0, 0.25)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          overflow: "hidden",
        }}
      >
        {entries.map((entry, i) => (
          <div
            key={entry.hash}
            style={{
              display: "flex",
              alignItems: "baseline",
              gap: 12,
              padding: "12px 20px",
              borderBottom:
                i < entries.length - 1
                  ? "1px solid rgba(50, 100, 160, 0.1)"
                  : "none",
              fontSize: 13,
              lineHeight: "20px",
              fontFamily: "var(--font-mono), monospace",
            }}
          >
            <Link
              href="/explore/0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048/wall"
              style={{
                color: "var(--bp-accent2)",
                fontSize: 11,
                flexShrink: 0,
                minWidth: 48,
                textDecoration: "none",
              }}
            >
              {entry.hash}
            </Link>
            <Link
              href={`/explore/${entry.author}`}
              style={{
                color: "var(--bp-accent)",
                fontWeight: 500,
                flexShrink: 0,
                minWidth: 80,
                fontSize: 12,
                textDecoration: "none",
              }}
            >
              {truncateAddr(entry.author)}
            </Link>
            <Link
              href="/explore/0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048/wall"
              style={{
                color: "var(--bp-text)",
                flex: 1,
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                fontSize: 12,
                textDecoration: "none",
              }}
            >
              {entry.message}
            </Link>
            <span
              style={{
                color: "var(--bp-dim)",
                fontSize: 11,
                flexShrink: 0,
                opacity: 0.7,
              }}
            >
              {timeAgo(entry.time)}
            </span>
          </div>
        ))}
      </div>
    </section>
  );
}
