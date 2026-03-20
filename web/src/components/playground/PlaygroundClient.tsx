"use client";

import { useState, useRef, useCallback } from "react";
import Link from "next/link";
import {
  REPOBOX_SYSTEM_PROMPT,
  VENICE_ENDPOINT,
  VENICE_MODEL,
  EXPLAIN_EXAMPLES,
} from "@/lib/repobox-prompt";

type Mode = "generate" | "explain";

const GENERATE_EXAMPLES = [
  {
    label: "team + agents",
    text: "Two founders with full access. Three AI agents that can only work on feature branches. Nobody except founders can touch the config file.",
  },
  {
    label: "solo + codex",
    text: "Solo developer. One Codex agent that can push to any branch except main, and can only append to the config.",
  },
  {
    label: "open source",
    text: "Open source repo. Maintainers can merge to main. External contributors can only push to their own branches (contributor/*). Contracts folder is locked to maintainers.",
  },
  {
    label: "multi-agent",
    text: "Five agents, five keys, one repo. Each agent gets its own feature namespace. Only the orchestrator agent can merge. Nobody can force-push.",
  },
];

const EXPLAIN_LABELS = ["minimal", "file-locked", "multi-agent"];

function highlightYaml(text: string): string {
  text = text.replace(/^```ya?ml\n?/gm, "").replace(/^```\n?/gm, "");

  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/(#.*$)/gm, '<span style="color:#3a5a72;">$1</span>')
    .replace(
      /^(\s*)([\w-]+)(:)/gm,
      '$1<span style="color:#4fc3f7;">$2</span><span style="color:#5a7a94;">$3</span>'
    )
    .replace(
      /(&quot;[^&]*&quot;|'[^']*')/g,
      '<span style="color:#81d4fa;">$1</span>'
    )
    .replace(/^(\s*)(- )/gm, '$1<span style="color:#5a7a94;">- </span>')
    .replace(
      /(evm:0x[\w.]+)/g,
      '<span style="color:#b8d4e3;opacity:0.7;">$1</span>'
    )
    .replace(
      /(\.\/[\w.*\/-]+)/g,
      '<span style="color:#81d4fa;">$1</span>'
    );
}

function highlightExplanation(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(
      /\*\*([^*]+)\*\*/g,
      '<span style="color:#e8f4fd;font-weight:600;">$1</span>'
    )
    .replace(
      /`([^`]+)`/g,
      '<span style="color:#4fc3f7;background:rgba(79,195,247,0.08);padding:0 4px;border-radius:2px;">$1</span>'
    )
    .replace(
      /^(\s*[-•])/gm,
      '<span style="color:#4fc3f7;">$1</span>'
    )
    .replace(/✅/g, '<span style="color:#4caf50;">✅</span>')
    .replace(/❌/g, '<span style="color:#f44336;">❌</span>');
}

export function PlaygroundClient() {
  const [mode, setMode] = useState<Mode>("generate");
  const [generateInput, setGenerateInput] = useState("");
  const [explainInput, setExplainInput] = useState("");
  const [output, setOutput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [statusText, setStatusText] = useState("");
  const abortRef = useRef<AbortController | null>(null);
  const outputRef = useRef<HTMLDivElement>(null);

  const run = useCallback(async () => {
    const input = mode === "generate" ? generateInput.trim() : explainInput.trim();
    if (!input) return;

    if (abortRef.current) abortRef.current.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    setOutput("");
    setIsStreaming(true);
    setStatusText("thinking...");

    const userMessage =
      mode === "generate"
        ? `Generate a .repobox.yml for this scenario:\n\n${input}\n\nOutput ONLY the YAML. No explanation, no code fences.`
        : `Explain this .repobox.yml in plain English. What can each group do? What are they denied?\n\n${input}`;

    try {
      const res = await fetch(VENICE_ENDPOINT, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${process.env.NEXT_PUBLIC_VENICE_API_KEY}`,
        },
        body: JSON.stringify({
          model: VENICE_MODEL,
          stream: true,
          messages: [
            { role: "system", content: REPOBOX_SYSTEM_PROMPT },
            { role: "user", content: userMessage },
          ],
          temperature: 0.3,
          max_tokens: 2000,
          venice_parameters: {
            include_venice_system_prompt: false,
            disable_thinking: true,
          },
        }),
        signal: controller.signal,
      });

      if (!res.ok) throw new Error(`API error: ${res.status}`);

      setStatusText("streaming...");

      const reader = res.body!.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      let fullText = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop()!;

        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          const data = line.slice(6).trim();
          if (data === "[DONE]") continue;

          try {
            const json = JSON.parse(data);
            const delta = json.choices?.[0]?.delta?.content;
            if (delta) {
              fullText += delta;
              setOutput(fullText);
              if (outputRef.current) {
                outputRef.current.scrollTop = outputRef.current.scrollHeight;
              }
            }
          } catch {}
        }
      }
    } catch (err: any) {
      if (err.name !== "AbortError") {
        setOutput(`Error: ${err.message}`);
      }
    } finally {
      setIsStreaming(false);
      setStatusText("");
      abortRef.current = null;
    }
  }, [mode, generateInput, explainInput]);

  const copyOutput = useCallback(() => {
    const text = outputRef.current?.innerText || output;
    navigator.clipboard.writeText(text);
  }, [output]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        run();
      }
    },
    [run]
  );

  const highlightedOutput = output
    ? mode === "generate"
      ? highlightYaml(output)
      : highlightExplanation(output)
    : "";

  return (
    <div
      className="max-w-[760px] mx-auto"
      style={{ padding: "40px 20px 100px" }}
      onKeyDown={handleKeyDown}
    >
      {/* Header */}
      <header style={{ marginBottom: 48 }}>
        <div
          style={{
            display: "flex",
            alignItems: "baseline",
            justifyContent: "space-between",
            marginBottom: 24,
          }}
        >
          <Link
            href="/"
            style={{ fontWeight: 700, fontSize: 24, lineHeight: 1.1 }}
          >
            repo<span className="logo-dot">.</span>box
          </Link>
          <span
            style={{
              fontSize: 12,
              color: "var(--bp-dim)",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
            }}
          >
            playground
          </span>
        </div>
        <p
          style={{
            fontSize: 14,
            lineHeight: "22px",
            color: "var(--bp-dim)",
            maxWidth: 560,
          }}
        >
          Try the config generator. Describe your repo permissions in English and
          get a{" "}
          <code
            style={{
              color: "var(--bp-accent)",
              background: "rgba(79,195,247,0.08)",
              padding: "1px 6px",
              borderRadius: 3,
            }}
          >
            .repobox.yml
          </code>{" "}
          — or paste a config to understand what it does.
        </p>
      </header>

      {/* Mode toggle */}
      <div style={{ display: "flex", gap: 8, marginBottom: 24 }}>
        <button
          className={`playground-mode-btn ${mode === "generate" ? "active" : ""}`}
          onClick={() => setMode("generate")}
        >
          English → Config
        </button>
        <button
          className={`playground-mode-btn ${mode === "explain" ? "active" : ""}`}
          onClick={() => setMode("explain")}
        >
          Config → English
        </button>
      </div>

      {/* Generate mode */}
      {mode === "generate" && (
        <div>
          <label className="playground-label">Describe your permissions</label>
          <textarea
            className="playground-textarea"
            rows={6}
            value={generateInput}
            onChange={(e) => setGenerateInput(e.target.value)}
            placeholder="e.g. Three founders can do anything. Two AI agents can only push to feature branches and can't touch the config file."
          />
          <div className="playground-chips">
            {GENERATE_EXAMPLES.map((ex) => (
              <button
                key={ex.label}
                className="playground-chip"
                onClick={() => setGenerateInput(ex.text)}
              >
                {ex.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Explain mode */}
      {mode === "explain" && (
        <div>
          <label className="playground-label">Paste a .repobox.yml</label>
          <textarea
            className="playground-textarea"
            rows={10}
            value={explainInput}
            onChange={(e) => setExplainInput(e.target.value)}
            placeholder={`groups:\n  founders:\n    - evm:0xAAA...\n  agents:\n    - evm:0xBBB...\n\npermissions:\n  default: allow\n  rules:\n    - founders push >*\n    - founders merge >*\n    - agents push >feature/**\n    - agents create >feature/**`}
          />
          <div className="playground-chips">
            {EXPLAIN_LABELS.map((label, i) => (
              <button
                key={label}
                className="playground-chip"
                onClick={() => setExplainInput(EXPLAIN_EXAMPLES[i])}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Go */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          marginBottom: 8,
        }}
      >
        <button
          className="playground-go-btn"
          onClick={run}
          disabled={isStreaming}
        >
          {mode === "generate" ? "Generate" : "Explain"}
        </button>
        {statusText && (
          <span style={{ fontSize: 11, color: "var(--bp-dim)", display: "inline-flex", alignItems: "center", gap: 6 }}>
            <span
              style={{
                width: 6,
                height: 6,
                borderRadius: "50%",
                background: "var(--bp-accent)",
                animation: "pulse 2s infinite",
                display: "inline-block",
              }}
            />
            {statusText}
          </span>
        )}
      </div>

      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#2a4a62",
          fontSize: 20,
          padding: "8px 0",
        }}
      >
        ↓
      </div>

      {/* Output */}
      <div
        style={{
          display: "flex",
          alignItems: "baseline",
          justifyContent: "space-between",
          marginBottom: 8,
        }}
      >
        <label className="playground-label">
          {mode === "generate" ? "Generated config" : "Explanation"}
        </label>
        <button className="playground-copy-btn" onClick={copyOutput}>
          copy
        </button>
      </div>
      <div
        ref={outputRef}
        className={`playground-output ${isStreaming ? "streaming" : ""}`}
        dangerouslySetInnerHTML={
          highlightedOutput
            ? { __html: highlightedOutput }
            : { __html: '<span style="color:#2a4a62;">Output will appear here...</span>' }
        }
      />
    </div>
  );
}
