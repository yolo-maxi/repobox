"use client";

import { useEffect, useRef } from "react";

export function LandingHireCTA() {
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
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 80 }}>
      <div
        style={{
          background: "rgba(6, 14, 26, 0.85)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: "40px 36px",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          textAlign: "center",
        }}
      >
        <h2
          style={{
            fontSize: 28,
            lineHeight: "36px",
            color: "var(--bp-heading)",
            fontWeight: 700,
            marginBottom: 8,
          }}
        >
          Work with us
        </h2>
        <p
          style={{
            fontSize: 16,
            lineHeight: "24px",
            color: "var(--bp-dim)",
            marginBottom: 32,
            maxWidth: 480,
            margin: "0 auto 32px",
          }}
        >
          Tell us what you are trying to build. It goes straight to us, not to a
          queue — we read every one and reply by email, usually within a day.
        </p>

        {/* Primary CTA Button.
            Text is --bp-bg (near-black) rather than white: white on the light
            --bp-accent measures 2.0:1, which fails WCAG AA for 16px body text.
            The dark-on-accent pairing clears AA comfortably. */}
        <div style={{ marginBottom: 24 }}>
          <a
            href="/hire"
            style={{
              display: "inline-flex",
              alignItems: "center",
              justifyContent: "center",
              gap: 12,
              background: "var(--bp-accent)",
              color: "var(--bp-bg)",
              padding: "16px 32px",
              borderRadius: 8,
              textDecoration: "none",
              fontWeight: 600,
              fontSize: 16,
              transition: "all 0.2s",
              border: "2px solid var(--bp-accent)",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.color = "var(--bp-accent)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "var(--bp-accent)";
              e.currentTarget.style.color = "var(--bp-bg)";
            }}
          >
            Tell us about your project
          </a>
        </div>

      </div>
    </section>
  );
}
