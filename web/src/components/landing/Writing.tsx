"use client";

import { useEffect, useRef, useState } from "react";
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

export function LandingWriting() {
  const sectionRef = useRef<HTMLElement>(null);
  const [email, setEmail] = useState("");
  const [message, setMessage] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim()) return;

    setIsSubmitting(true);
    setMessage("");

    try {
      const res = await fetch("/api/subscribe", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: email.trim() }),
      });

      if (res.ok) {
        setMessage("You're in. First dispatch coming soon.");
        setEmail("");
      } else {
        const data = await res.json().catch(() => ({}));
        throw new Error(data.error || "Something went wrong");
      }
    } catch (err: any) {
      setMessage(err.message);
    } finally {
      setIsSubmitting(false);
    }
  };

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

      {/* Subscribe */}
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
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="you@example.com"
            required
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
            disabled={isSubmitting}
            style={{
              background: "var(--bp-accent)",
              color: "var(--bp-bg)",
              border: "none",
              borderRadius: 4,
              padding: "8px 20px",
              height: 40,
              fontFamily: "var(--font-mono), monospace",
              fontWeight: 600,
              fontSize: 12,
              lineHeight: "20px",
              cursor: isSubmitting ? "not-allowed" : "pointer",
              whiteSpace: "nowrap",
              opacity: isSubmitting ? 0.6 : 1,
            }}
          >
            Subscribe
          </button>
        </form>
        {message && (
          <div
            style={{
              fontSize: 12,
              lineHeight: "20px",
              marginTop: 8,
              color: message.includes("in") ? "var(--bp-accent)" : "#ff4444",
            }}
          >
            {message}
          </div>
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
    </section>
  );
}
