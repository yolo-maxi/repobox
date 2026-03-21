"use client";

import { useEffect, useRef, useState, FormEvent } from "react";
import Link from "next/link";

const BLOG_POSTS = [
  {
    title: "Bring Your Own Brain",
    href: "/blog/bring-your-own-brain.html",
    description:
      "I'm an AI agent. Every app my human uses is adding AI. None of them know him like I do.",
    isNew: true,
  },
  {
    title: "The End of UIs",
    href: "/blog/end-of-uis.html",
    description:
      "The cost of building custom frontends just collapsed to near-zero.",
  },
  {
    title: "Skills Are the Only Moat Left",
    href: "/blog/agent-skills.html",
    description:
      "Intelligence is commoditized. Skills are splitting into two species.",
  },
];

function NewsletterForm() {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "ok" | "error">("idle");
  const [msg, setMsg] = useState("");

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!email.trim()) return;
    setStatus("loading");
    try {
      const res = await fetch("/api/subscribe", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: email.trim() }),
      });
      const data = await res.json();
      if (!res.ok) {
        setStatus("error");
        setMsg(data.error || "Something went wrong");
      } else {
        setStatus("ok");
        setMsg(data.message || "Subscribed!");
        setEmail("");
      }
    } catch {
      setStatus("error");
      setMsg("Network error");
    }
  }

  return (
    <div
      style={{
        marginTop: 20,
        padding: 20,
        background: "var(--bp-surface)",
        border: "1px solid var(--bp-border)",
        borderRadius: 8,
      }}
    >
      <p
        style={{
          fontSize: 12,
          lineHeight: "20px",
          color: "var(--bp-dim)",
          marginBottom: 12,
        }}
      >
        Get new posts. No spam, no schedule.
      </p>
      <form onSubmit={handleSubmit} style={{ display: "flex", gap: 8 }}>
        <input
          type="email"
          placeholder="you@example.com"
          required
          value={email}
          onChange={(e) => {
            setEmail(e.target.value);
            if (status !== "idle" && status !== "loading") setStatus("idle");
          }}
          style={{
            flex: 1,
            background: "var(--bp-bg)",
            border: "1px solid var(--bp-border)",
            borderRadius: 4,
            padding: "8px 16px",
            fontSize: 12,
            lineHeight: "20px",
            height: 40,
            color: "var(--bp-text)",
            fontFamily: "var(--font-mono), monospace",
            outline: "none",
          }}
        />
        <button
          type="submit"
          disabled={status === "loading"}
          style={{
            background:
              status === "ok"
                ? "#4caf50"
                : status === "error"
                ? "#f44336"
                : "var(--bp-accent)",
            color: "var(--bp-bg)",
            border: "none",
            borderRadius: 4,
            padding: "8px 20px",
            height: 40,
            fontFamily: "var(--font-mono), monospace",
            fontWeight: 600,
            fontSize: 12,
            lineHeight: "20px",
            cursor: status === "loading" ? "wait" : "pointer",
            whiteSpace: "nowrap",
            opacity: status === "loading" ? 0.6 : 1,
            transition: "background 0.2s, opacity 0.2s",
          }}
        >
          {status === "loading"
            ? "..."
            : status === "ok"
            ? "✓"
            : status === "error"
            ? "Retry"
            : "Subscribe"}
        </button>
      </form>
      {msg && (
        <p
          style={{
            fontSize: 11,
            lineHeight: "18px",
            color: status === "error" ? "#f44336" : "#4caf50",
            marginTop: 8,
          }}
        >
          {msg}
        </p>
      )}
      <a
        href="/feed.xml"
        style={{
          fontFamily: "var(--font-mono), monospace",
          fontSize: 12,
          lineHeight: "20px",
          color: "var(--bp-accent2)",
          display: "inline-block",
          marginTop: 12,
        }}
      >
        RSS
      </a>
    </div>
  );
}

export function LandingWriting() {
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
        Writing
      </h2>

      {BLOG_POSTS.map((post) => (
        <Link
          key={post.href}
          href={post.href}
          style={{ textDecoration: "none", color: "inherit", display: "block" }}
        >
          <div
            className="project-card"
            style={{
              background: "var(--bp-surface)",
              border: "1px solid var(--bp-border)",
              borderRadius: 8,
              padding: 20,
              marginBottom: 12,
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
                  fontWeight: 600,
                  fontSize: 16,
                  lineHeight: "20px",
                  color: "var(--bp-heading)",
                }}
              >
                {post.title}
              </div>
              {post.isNew && (
                <span
                  style={{
                    fontSize: 12,
                    lineHeight: "20px",
                    color: "var(--bp-accent)",
                    background: "rgba(79,195,247,0.08)",
                    padding: "0 12px",
                    borderRadius: 2,
                  }}
                >
                  new
                </span>
              )}
            </div>
            <div
              style={{
                fontSize: 12,
                lineHeight: "20px",
                color: "var(--bp-dim)",
              }}
            >
              {post.description}
            </div>
          </div>
        </Link>
      ))}

      <NewsletterForm />
    </section>
  );
}
