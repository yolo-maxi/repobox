"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";

/* Simple YAML syntax highlighter */
function HighlightedYaml({ code }: { code: string }) {
  const lines = code.split("\n");
  return (
    <>
      {lines.map((line, i) => (
        <div key={i}>{colorize(line)}</div>
      ))}
    </>
  );
}

function colorize(line: string): React.ReactNode {
  // Comments
  if (line.trimStart().startsWith("#")) {
    return <span style={{ color: "#546e7a" }}>{line}</span>;
  }

  // Key: value lines
  const keyMatch = line.match(/^(\s*)([\w-]+)(:)(.*)/);
  if (keyMatch) {
    const [, indent, key, colon, rest] = keyMatch;
    return (
      <>
        {indent}
        <span style={{ color: "#4fc3f7" }}>{key}</span>
        <span style={{ color: "#546e7a" }}>{colon}</span>
        {colorizeValue(rest)}
      </>
    );
  }

  // List items
  const listMatch = line.match(/^(\s*)(- )(.*)/);
  if (listMatch) {
    const [, indent, dash, rest] = listMatch;
    // Check for inline comment
    const commentIdx = rest.indexOf("#");
    if (commentIdx > 0) {
      const value = rest.slice(0, commentIdx);
      const comment = rest.slice(commentIdx);
      return (
        <>
          {indent}
          <span style={{ color: "#546e7a" }}>{dash}</span>
          <span style={{ color: "#c3e88d" }}>{value}</span>
          <span style={{ color: "#546e7a" }}>{comment}</span>
        </>
      );
    }
    return (
      <>
        {indent}
        <span style={{ color: "#546e7a" }}>{dash}</span>
        {colorizeValue(" " + rest)}
      </>
    );
  }

  return line;
}

function colorizeValue(val: string): React.ReactNode {
  const trimmed = val.trim();
  // Keywords
  if (["deny", "allow"].includes(trimmed)) {
    return <span style={{ color: trimmed === "deny" ? "#ef5350" : "#4caf50" }}> {trimmed}</span>;
  }
  // Rule strings (verb >target)
  if (trimmed.includes(">")) {
    const parts = trimmed.split(/(\s+)/);
    return (
      <span>
        {" "}
        {parts.map((p, i) => {
          if (p.startsWith(">")) return <span key={i} style={{ color: "#ffcb6b" }}>{p}</span>;
          if (["own", "push", "read", "merge", "edit", "not"].includes(p))
            return <span key={i} style={{ color: p === "not" ? "#ef5350" : "#c792ea" }}>{p}</span>;
          return <span key={i} style={{ color: "#c3e88d" }}>{p}</span>;
        })}
      </span>
    );
  }
  // ENS / addresses
  if (trimmed.startsWith("ens:") || trimmed.startsWith("evm:") || trimmed.startsWith("resolver:")) {
    return <span style={{ color: "#c3e88d" }}>{val}</span>;
  }
  return <span style={{ color: "#e0e0e0" }}>{val}</span>;
}

const CONFIG_YAML = `# .repobox/config.yml
resolver: onchain

groups:
  founders:
    - ens:0xfran.eth
  agents:
    - evm:0xDbbA...2048   # deep-blue-kraken
    - evm:0x8224...fA09   # yolo-shipper

permissions:
  default: deny
  rules:
    - founders own >main
    - agents push >feature/**
    - agents read >*
    - agents not edit .repobox/config.yml`;

export function ConfigExample() {
  const sectionRef = useRef<HTMLElement>(null);
  const [copied, setCopied] = useState(false);

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

  const handleCopy = () => {
    navigator.clipboard.writeText(CONFIG_YAML);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section ref={sectionRef} className="reveal" style={{ marginBottom: 60 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 20 }}>
        <h2
          style={{
            fontSize: 14,
            lineHeight: "20px",
            textTransform: "uppercase",
            letterSpacing: "0.12em",
            color: "var(--bp-accent)",
            fontWeight: 500,
            margin: 0,
          }}
        >
          Minimal Config
        </h2>
        <Link
          href="/playground"
          style={{
            fontFamily: "var(--font-mono), monospace",
            fontSize: 12,
            color: "var(--bp-accent)",
            textDecoration: "none",
            display: "inline-flex",
            alignItems: "center",
            gap: 5,
          }}
        >
          <span style={{ fontSize: 13 }}>▶</span> Try in Playground
        </Link>
      </div>
      <div style={{ position: "relative" }}>
        <pre
          style={{
            background: "rgba(0, 0, 0, 0.4)",
            border: "1px solid var(--bp-border)",
            borderRadius: 6,
            padding: 20,
            overflow: "auto",
            fontFamily: "var(--font-mono), monospace",
            fontSize: 13,
            lineHeight: "22px",
            margin: 0,
          }}
        >
          <code>
            <HighlightedYaml code={CONFIG_YAML} />
          </code>
        </pre>
        <button
          onClick={handleCopy}
          style={{
            position: "absolute",
            top: 12,
            right: 12,
            color: copied ? "var(--bp-accent)" : "var(--bp-dim)",
            fontFamily: "var(--font-mono), monospace",
            fontSize: 12,
            background: "transparent",
            border: `1px solid ${copied ? "var(--bp-accent)" : "var(--bp-border)"}`,
            padding: "6px 14px",
            borderRadius: 4,
            cursor: "pointer",
            transition: "color 0.2s, border-color 0.2s",
          }}
        >
          {copied ? "copied!" : "copy"}
        </button>
      </div>

    </section>
  );
}
