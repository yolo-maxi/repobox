"use client";

import Link from "next/link";

export function LandingFooter() {
  return (
    <footer
      style={{
        background: "#060e1a",
        borderTop: "1px solid var(--bp-border)",
        marginTop: 0,
        position: "relative",
        zIndex: 2,
      }}
    >
      <div
        className="footer-grid"
        style={{
          maxWidth: 960,
          margin: "0 auto",
          padding: "60px 40px 40px",
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
          gap: 40,
        }}
      >
        {/* Col 1: Say Hi */}
        <div>
          <h3
            style={{
              fontSize: 12,
              lineHeight: "20px",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              color: "var(--bp-dim)",
              fontWeight: 500,
              marginBottom: 16,
            }}
          >
            Say Hi
          </h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            <a
              href="https://x.com/FrancescoRenziA"
              target="_blank"
              rel="noopener noreferrer"
              style={{ fontSize: 12, lineHeight: "20px", color: "var(--bp-text)" }}
            >
              𝕏 Fran
            </a>
            <a
              href="https://x.com/oceanvael"
              target="_blank"
              rel="noopener noreferrer"
              style={{ fontSize: 12, lineHeight: "20px", color: "var(--bp-text)" }}
            >
              𝕏 Ocean
            </a>
            <a
              href="https://warpcast.com/0xfran"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                fontSize: 12,
                lineHeight: "20px",
                color: "var(--bp-text)",
                display: "inline-flex",
                alignItems: "center",
                gap: 6,
              }}
            >
              <FarcasterIcon /> 0xFran
            </a>
          </div>
        </div>

        {/* Col 2: Agent-Readable */}
        <div>
          <h3
            style={{
              fontSize: 12,
              lineHeight: "20px",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              color: "var(--bp-dim)",
              fontWeight: 500,
              marginBottom: 16,
            }}
          >
            <span style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
              <span
                style={{
                  width: 6,
                  height: 6,
                  borderRadius: "50%",
                  background: "var(--bp-accent)",
                  animation: "pulse 2s infinite",
                }}
              />
              Agent-Readable
            </span>
          </h3>
          <p
            style={{
              fontSize: 12,
              lineHeight: "20px",
              color: "var(--bp-dim)",
              marginBottom: 12,
            }}
          >
            This site speaks human and machine.
          </p>
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            <Link
              href="/playground"
              style={{
                fontFamily: "var(--font-mono), monospace",
                fontSize: 12,
                lineHeight: "20px",
                color: "var(--bp-accent2)",
              }}
            >
              playground
            </Link>
            <a
              href="/llms.txt"
              style={{
                fontFamily: "var(--font-mono), monospace",
                fontSize: 12,
                lineHeight: "20px",
                color: "var(--bp-accent2)",
              }}
            >
              llms.txt
            </a>
            <a
              href="/feed.xml"
              style={{
                fontFamily: "var(--font-mono), monospace",
                fontSize: 12,
                lineHeight: "20px",
                color: "var(--bp-accent2)",
              }}
            >
              feed.xml
            </a>
          </div>
        </div>

        {/* Col 3: About */}
        <div>
          <h3
            style={{
              fontSize: 12,
              lineHeight: "20px",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              color: "var(--bp-dim)",
              fontWeight: 500,
              marginBottom: 16,
            }}
          >
            repo.box
          </h3>
          <p style={{ fontSize: 12, lineHeight: "20px", color: "var(--bp-dim)", marginBottom: 16 }}>
            Git permission layer for AI agents. Built for the Synthesis Hackathon.
            Shipping March 2026.
          </p>
          <code
            style={{
              display: "block",
              fontSize: 11,
              lineHeight: "18px",
              color: "var(--bp-accent2)",
              background: "rgba(79,195,247,0.06)",
              padding: "8px 12px",
              borderRadius: 4,
              border: "1px solid var(--bp-border)",
              wordBreak: "break-all",
            }}
          >
            <span style={{ color: "var(--bp-accent)" }}>$</span> curl -sSf https://repo.box/install.sh | sh
          </code>
        </div>
      </div>

      {/* Bottom bar */}
      <div
        style={{
          maxWidth: 960,
          margin: "0 auto",
          padding: "20px 40px",
          borderTop: "1px solid rgba(50,100,160,0.15)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: 8,
          color: "var(--bp-dim)",
          fontSize: 12,
          lineHeight: "20px",
        }}
      >
        <span>© 2026 repo.box</span>
      </div>
    </footer>
  );
}

function FarcasterIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 1000 1000" fill="none">
      <path
        d="M257.778 155.556H742.222V844.444H671.111V528.889H670.414C662.554 441.677 589.258 373.333 500 373.333C410.742 373.333 337.446 441.677 329.586 528.889H328.889V844.444H257.778V155.556Z"
        fill="currentColor"
      />
      <path
        d="M128.889 253.333L157.778 351.111H182.222V746.667C169.949 746.667 160 756.616 160 768.889V795.556H155.556C143.283 795.556 133.333 805.505 133.333 817.778V844.444H382.222V817.778C382.222 805.505 372.273 795.556 360 795.556H355.556V768.889C355.556 756.616 345.606 746.667 333.333 746.667H306.667V351.111H331.111L360 253.333H128.889Z"
        fill="currentColor"
      />
      <path
        d="M640 253.333L668.889 351.111H693.333V746.667C681.06 746.667 671.111 756.616 671.111 768.889V795.556H666.667C654.394 795.556 644.444 805.505 644.444 817.778V844.444H893.333V817.778C893.333 805.505 883.384 795.556 871.111 795.556H866.667V768.889C866.667 756.616 856.717 746.667 844.444 746.667V351.111H868.889L897.778 253.333H640Z"
        fill="currentColor"
      />
    </svg>
  );
}
