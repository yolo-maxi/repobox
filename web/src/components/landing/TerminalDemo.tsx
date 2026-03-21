"use client";

import { useEffect, useRef, useState } from "react";

interface TerminalLine {
  type: "command" | "output" | "success" | "error";
  text: string;
  delay: number; // ms before this line appears
}

const TERMINAL_SCRIPT: TerminalLine[] = [
  { type: "command", text: "$ repobox init", delay: 0 },
  { type: "output", text: "Creating .repobox/config.yml", delay: 800 },
  { type: "success", text: "✓ Initialized repobox", delay: 600 },
  { type: "command", text: "$ repobox key generate --name codex-agent", delay: 1200 },
  { type: "output", text: "Generated key: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb", delay: 800 },
  { type: "success", text: "✓ Key added to config", delay: 600 },
  { type: "command", text: "$ git push origin feat/new-feature", delay: 1400 },
  { type: "success", text: "✓ Push authorized for codex-agent (branch: feat/*)", delay: 900 },
  { type: "command", text: "$ git push origin main", delay: 1600 },
  { type: "error", text: "✗ Denied: codex-agent cannot push to main", delay: 900 },
];

export function TerminalDemo() {
  const [visibleLines, setVisibleLines] = useState<number>(0);
  const [currentCharIndex, setCurrentCharIndex] = useState<number>(0);
  const animationRef = useRef<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let lastTime = performance.now();
    let cumulativeDelay = 0;
    let lineIndex = 0;
    let charIndex = 0;

    const animate = (currentTime: number) => {
      if (!startTimeRef.current) {
        startTimeRef.current = currentTime;
        lastTime = currentTime;
      }

      const elapsed = currentTime - lastTime;

      if (lineIndex >= TERMINAL_SCRIPT.length) {
        // Animation complete, restart after a pause
        setTimeout(() => {
          setVisibleLines(0);
          setCurrentCharIndex(0);
          startTimeRef.current = null;
          lineIndex = 0;
          charIndex = 0;
          cumulativeDelay = 0;
          animationRef.current = requestAnimationFrame(animate);
        }, 3000);
        return;
      }

      const currentLine = TERMINAL_SCRIPT[lineIndex];

      // Wait for the delay before starting this line
      if (elapsed < currentLine.delay) {
        animationRef.current = requestAnimationFrame(animate);
        return;
      }

      // Type out commands character by character, show others instantly
      if (currentLine.type === "command") {
        const charsPerFrame = 2; // Typing speed
        charIndex += charsPerFrame;

        if (charIndex >= currentLine.text.length) {
          charIndex = currentLine.text.length;
          setCurrentCharIndex(charIndex);
          setVisibleLines(lineIndex + 1);
          lineIndex++;
          charIndex = 0;
          lastTime = currentTime;
        } else {
          setCurrentCharIndex(charIndex);
        }
      } else {
        // Output, success, error - show instantly
        setVisibleLines(lineIndex + 1);
        lineIndex++;
        charIndex = 0;
        lastTime = currentTime;
      }

      animationRef.current = requestAnimationFrame(animate);
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  return (
    <div
      ref={containerRef}
      style={{
        background: "#0a0f1a",
        border: "1px solid rgba(79, 195, 247, 0.2)",
        borderRadius: 8,
        overflow: "hidden",
        fontFamily: "var(--font-mono), monospace",
        fontSize: 13,
        lineHeight: "20px",
        maxWidth: 600,
        margin: "0 auto",
      }}
    >
      {/* Terminal title bar */}
      <div
        style={{
          background: "rgba(6, 14, 26, 0.9)",
          borderBottom: "1px solid rgba(79, 195, 247, 0.15)",
          padding: "8px 16px",
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <div style={{ display: "flex", gap: 6 }}>
          <div
            style={{
              width: 10,
              height: 10,
              borderRadius: "50%",
              background: "#ff5f56",
            }}
          />
          <div
            style={{
              width: 10,
              height: 10,
              borderRadius: "50%",
              background: "#ffbd2e",
            }}
          />
          <div
            style={{
              width: 10,
              height: 10,
              borderRadius: "50%",
              background: "#27c93f",
            }}
          />
        </div>
        <span
          style={{
            fontSize: 11,
            color: "var(--bp-dim)",
            marginLeft: 8,
          }}
        >
          repobox demo
        </span>
      </div>

      {/* Terminal content */}
      <div style={{ padding: "16px 20px", minHeight: 280 }}>
        {TERMINAL_SCRIPT.slice(0, visibleLines).map((line, i) => {
          const isCurrentCommand =
            i === visibleLines - 1 && line.type === "command";
          const displayText = isCurrentCommand
            ? line.text.slice(0, currentCharIndex)
            : line.text;

          let color = "var(--bp-text)";
          if (line.type === "command") color = "#ffffff";
          if (line.type === "success") color = "#27c93f";
          if (line.type === "error") color = "#ff5f56";
          if (line.type === "output") color = "var(--bp-dim)";

          return (
            <div
              key={i}
              style={{
                color,
                marginBottom: line.type === "command" ? 4 : 2,
                fontWeight: line.type === "command" ? 500 : 400,
              }}
            >
              {displayText}
              {isCurrentCommand && (
                <span
                  style={{
                    display: "inline-block",
                    width: 8,
                    height: 16,
                    background: "var(--bp-accent)",
                    marginLeft: 2,
                    animation: "blink 1s step-end infinite",
                  }}
                />
              )}
            </div>
          );
        })}
      </div>

      <style jsx>{`
        @keyframes blink {
          0%,
          50% {
            opacity: 1;
          }
          51%,
          100% {
            opacity: 0;
          }
        }
      `}</style>
    </div>
  );
}
