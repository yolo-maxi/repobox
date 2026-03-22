"use client";

import { useState, useRef, useCallback } from "react";
import Link from "next/link";
import { SiteNav } from "@/components/SiteNav";
import {
  REPOBOX_SYSTEM_PROMPT,
  VENICE_ENDPOINT,
  VENICE_MODEL,
  EXPLAIN_EXAMPLES,
} from "@/lib/repobox-prompt";
import { YamlHighlighter, ExplanationHighlighter } from "./SyntaxHighlighter";
import { TestRunner } from "./TestRunner";

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

// Note: Syntax highlighting functions moved to SyntaxHighlighter.tsx for better organization

export function PlaygroundClient() {
  const [mode, setMode] = useState<Mode>("generate");
  const [generateInput, setGenerateInput] = useState("");
  const [explainInput, setExplainInput] = useState("");
  const [output, setOutput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [statusText, setStatusText] = useState("");
  const [showTests, setShowTests] = useState(false);
  const abortRef = useRef<AbortController | null>(null);
  const outputRef = useRef<HTMLDivElement>(null);

  const runGeneration = useCallback(async (inputText: string, targetMode: Mode): Promise<string> => {
    if (abortRef.current) abortRef.current.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    const userMessage =
      targetMode === "generate"
        ? `Generate a .repobox/config.yml for this scenario:\n\n${inputText}\n\nOutput ONLY the YAML. No explanation, no code fences.`
        : `Explain this .repobox/config.yml in plain English. What can each group do? What are they denied?\n\n${inputText}`;

    try {
      const res = await fetch(VENICE_ENDPOINT, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${process.env.NEXT_PUBLIC_VENICE_API_KEY}`,
        },
        body: JSON.stringify({
          model: VENICE_MODEL,
          stream: false, // Non-streaming for test runs
          messages: [
            { role: "system", content: REPOBOX_SYSTEM_PROMPT },
            { role: "user", content: userMessage },
          ],
          temperature: 0.1,
          max_tokens: 1500,
          ...(VENICE_MODEL.includes('claude') ? {
            anthropic_parameters: {
              include_anthropic_system_prompt: false,
            }
          } : {
            venice_parameters: {
              include_venice_system_prompt: false,
              disable_thinking: true,
            }
          }),
        }),
        signal: controller.signal,
      });

      if (!res.ok) throw new Error(`API error: ${res.status}`);
      
      const data = await res.json();
      return data.choices?.[0]?.message?.content || "";
    } finally {
      abortRef.current = null;
    }
  }, []);

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
        ? `Generate a .repobox/config.yml for this scenario:\n\n${input}\n\nOutput ONLY the YAML. No explanation, no code fences.`
        : `Explain this .repobox/config.yml in plain English. What can each group do? What are they denied?\n\n${input}`;

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
          temperature: 0.1, // Reduced for more consistent structured output
          max_tokens: 1500,  // Reduced for faster responses
          // Model-specific optimizations
          ...(VENICE_MODEL.includes('claude') ? {
            anthropic_parameters: {
              include_anthropic_system_prompt: false,
            }
          } : {
            venice_parameters: {
              include_venice_system_prompt: false,
              disable_thinking: true,
            }
          }),
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

  return (
    <div className="pg-root">
      <SiteNav />

      {/* Header */}
      <div className="pg-header">
        <div className="pg-header-inner">
          <div className="pg-breadcrumb">
            <Link href="/">home</Link>
            <span>/</span>
            <span className="pg-breadcrumb-current">playground</span>
          </div>
          <h1 className="pg-title">Config Playground</h1>
          <p className="pg-description">
            Try the config generator. Describe your repo permissions in English and get a{" "}
            <code>.repobox/config.yml</code> — or paste a config to understand what it does.
          </p>
        </div>
      </div>

      {/* Content */}
      <div className="pg-content" onKeyDown={handleKeyDown}>
        {/* Main playground */}
        <div className="pg-panel">
          {/* Mode toggle */}
          <div className="pg-mode-toggles">
            <button
              className={`pg-mode-btn ${mode === "generate" ? "active" : ""}`}
              onClick={() => setMode("generate")}
            >
              English → Config
            </button>
            <button
              className={`pg-mode-btn ${mode === "explain" ? "active" : ""}`}
              onClick={() => setMode("explain")}
            >
              Config → English
            </button>
            {/* Test suite hidden — internal dev tool, not user-facing */}
          </div>

          {/* Generate mode */}
          {mode === "generate" && (
            <div className="pg-input-section">
              <label className="pg-label">Describe your permissions</label>
              <textarea
                className="pg-textarea"
                rows={6}
                value={generateInput}
                onChange={(e) => setGenerateInput(e.target.value)}
                placeholder="e.g. Three founders can do anything. Two AI agents can only push to feature branches and can't touch the config file."
              />
              <div className="pg-chips">
                {GENERATE_EXAMPLES.map((ex) => (
                  <button
                    key={ex.label}
                    className="pg-chip"
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
            <div className="pg-input-section">
              <label className="pg-label">Paste a .repobox/config.yml</label>
              <textarea
                className="pg-textarea"
                rows={10}
                value={explainInput}
                onChange={(e) => setExplainInput(e.target.value)}
                placeholder={`groups:\n  founders:\n    - evm:0xAAA...\n  agents:\n    - evm:0xBBB...\n\npermissions:\n  default: allow\n  rules:\n    - founders push >*\n    - founders merge >*\n    - agents push >feature/**\n    - agents create >feature/**`}
              />
              <div className="pg-chips">
                {EXPLAIN_LABELS.map((label, i) => (
                  <button
                    key={label}
                    className="pg-chip"
                    onClick={() => setExplainInput(EXPLAIN_EXAMPLES[i])}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Run button */}
          <div className="pg-run-section">
            <button
              className="pg-go-btn"
              onClick={run}
              disabled={isStreaming}
            >
              {mode === "generate" ? "Generate" : "Explain"}
            </button>
            {statusText && (
              <span className="pg-status">
                <span className="pg-status-dot" />
                {statusText}
              </span>
            )}
          </div>

          <div className="pg-arrow">↓</div>

          {/* Output */}
          <div className="pg-output-section">
            <div className="pg-output-header">
              <label className="pg-label">
                {mode === "generate" ? "Generated config" : "Explanation"}
              </label>
              <button className="pg-copy-btn" onClick={copyOutput}>
                copy
              </button>
            </div>
            <div
              ref={outputRef}
              className={`pg-output ${isStreaming ? "streaming" : ""}`}
            >
              {output ? (
                mode === "generate" ? (
                  <YamlHighlighter content={output} />
                ) : (
                  <ExplanationHighlighter content={output} />
                )
              ) : (
                <span className="pg-output-placeholder">Output will appear here...</span>
              )}
            </div>
          </div>

          {/* Test Runner — hidden from UI, kept for dev use */}
        </div>

        {/* Right sidebar */}
        <aside className="pg-sidebar">
          {/* About */}
          <div className="pg-sidebar-card">
            <div className="pg-sidebar-label">About Playground</div>
            <p className="pg-sidebar-text">
              Experiment with repo.box config generation and explanation. Use natural language to create permission rules or understand existing configurations.
            </p>
          </div>

          {/* Shortcuts */}
          <div className="pg-sidebar-card">
            <div className="pg-sidebar-label">Keyboard Shortcuts</div>
            <div className="pg-shortcut">
              <code>Ctrl + Enter</code>
              <span>Run generation</span>
            </div>
          </div>

          {/* Links */}
          <div className="pg-sidebar-card">
            <div className="pg-sidebar-label">Documentation</div>
            <Link href="/docs/config" className="pg-sidebar-link" style={{ display: 'block', marginBottom: 6 }}>
              Config Reference
            </Link>
            <Link href="/docs/permissions" className="pg-sidebar-link" style={{ display: 'block', marginBottom: 6 }}>
              Permission Rules
            </Link>
            <Link href="/docs/examples" className="pg-sidebar-link" style={{ display: 'block' }}>
              Example Configs
            </Link>
          </div>

          {/* Model info */}
          <div className="pg-sidebar-card">
            <div className="pg-sidebar-label">Model</div>
            <div className="pg-stat-row">
              <span>Provider</span><strong>Venice.ai</strong>
            </div>
            <div className="pg-stat-row">
              <span>Model</span><strong>Llama 3.3 70B</strong>
            </div>
            <div className="pg-stat-row">
              <span>Temperature</span><strong>0.1</strong>
            </div>
          </div>
        </aside>
      </div>

      <style jsx>{`
        .pg-root {
          min-height: 100vh;
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono, 'JetBrains Mono', monospace);
          font-size: 13px;
        }

        /* Header */
        .pg-header {
          border-bottom: 1px solid var(--bp-border);
          padding: 20px 32px;
        }
        .pg-header-inner { 
          max-width: 1280px; 
          margin: 0 auto; 
        }
        .pg-breadcrumb {
          display: flex; 
          align-items: center; 
          gap: 6px;
          font-size: 12px; 
          color: var(--bp-dim); 
          margin-bottom: 12px;
        }
        .pg-breadcrumb a { 
          color: var(--bp-accent); 
          text-decoration: none; 
        }
        .pg-breadcrumb a:hover { 
          opacity: 0.8; 
        }
        .pg-breadcrumb-current { 
          color: var(--bp-heading); 
          font-weight: 600; 
        }
        .pg-title {
          font-size: 24px; 
          font-weight: 700; 
          color: var(--bp-heading);
          letter-spacing: -0.5px;
          margin-bottom: 8px;
        }
        .pg-description {
          font-size: 14px;
          line-height: 22px;
          color: var(--bp-dim);
          margin: 0;
        }
        .pg-description code {
          color: var(--bp-accent);
          background: rgba(79,195,247,0.08);
          padding: 1px 6px;
          border-radius: 3px;
        }

        /* Content grid */
        .pg-content {
          max-width: 1280px; 
          margin: 0 auto;
          padding: 24px 32px;
          display: grid;
          grid-template-columns: 1fr 280px;
          gap: 24px;
        }

        /* Main panel */
        .pg-panel {
          min-width: 0;
        }

        /* Mode toggles */
        .pg-mode-toggles {
          display: flex; 
          gap: 8px; 
          margin-bottom: 24px;
        }
        .pg-mode-btn {
          padding: 8px 16px;
          border: 1px solid var(--bp-border);
          background: var(--bp-surface);
          color: var(--bp-text);
          font-family: inherit;
          font-size: 12px;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.15s;
        }
        .pg-mode-btn:hover {
          border-color: rgba(79, 195, 247, 0.3);
        }
        .pg-mode-btn.active {
          background: rgba(79, 195, 247, 0.1);
          border-color: var(--bp-accent);
          color: var(--bp-accent);
        }
        .pg-test-btn {
          padding: 8px 16px;
          border: 1px solid var(--bp-border);
          background: transparent;
          color: var(--bp-dim);
          font-family: inherit;
          font-size: 12px;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.15s;
        }
        .pg-test-btn:hover {
          color: var(--bp-accent);
          border-color: rgba(79, 195, 247, 0.3);
        }

        /* Input sections */
        .pg-input-section {
          margin-bottom: 24px;
        }
        .pg-label {
          display: block;
          font-size: 12px;
          font-weight: 600;
          color: var(--bp-heading);
          margin-bottom: 8px;
        }
        .pg-textarea {
          width: 100%;
          padding: 12px;
          border: 1px solid var(--bp-border);
          border-radius: 6px;
          background: var(--bp-surface);
          color: var(--bp-text);
          font-family: inherit;
          font-size: 13px;
          line-height: 1.5;
          resize: vertical;
          margin-bottom: 12px;
        }
        .pg-textarea:focus {
          outline: none;
          border-color: var(--bp-accent);
        }
        .pg-chips {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }
        .pg-chip {
          padding: 4px 12px;
          border: 1px solid var(--bp-border);
          background: transparent;
          color: var(--bp-dim);
          font-family: inherit;
          font-size: 11px;
          border-radius: 16px;
          cursor: pointer;
          transition: all 0.15s;
        }
        .pg-chip:hover {
          background: rgba(79, 195, 247, 0.08);
          color: var(--bp-accent);
          border-color: rgba(79, 195, 247, 0.3);
        }

        /* Run section */
        .pg-run-section {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 8px;
        }
        .pg-go-btn {
          padding: 10px 20px;
          border: none;
          background: var(--bp-accent);
          color: white;
          font-family: inherit;
          font-size: 13px;
          font-weight: 600;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.15s;
        }
        .pg-go-btn:hover:not(:disabled) {
          background: rgba(79, 195, 247, 0.8);
        }
        .pg-go-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
        .pg-status {
          font-size: 11px;
          color: var(--bp-dim);
          display: inline-flex;
          align-items: center;
          gap: 6px;
        }
        .pg-status-dot {
          width: 6px;
          height: 6px;
          border-radius: 50%;
          background: var(--bp-accent);
          animation: pulse 2s infinite;
        }

        .pg-arrow {
          display: flex;
          align-items: center;
          justify-content: center;
          color: #2a4a62;
          font-size: 20px;
          padding: 8px 0;
          margin-bottom: 24px;
        }

        /* Output */
        .pg-output-section {
          margin-bottom: 24px;
        }
        .pg-output-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 8px;
        }
        .pg-copy-btn {
          padding: 4px 12px;
          border: 1px solid var(--bp-border);
          background: transparent;
          color: var(--bp-dim);
          font-family: inherit;
          font-size: 11px;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.15s;
        }
        .pg-copy-btn:hover {
          color: var(--bp-accent);
          border-color: rgba(79, 195, 247, 0.3);
        }
        .pg-output {
          min-height: 200px;
          padding: 16px;
          border: 1px solid var(--bp-border);
          border-radius: 6px;
          background: var(--bp-surface);
          overflow-y: auto;
          max-height: 400px;
        }
        .pg-output.streaming {
          border-color: rgba(79, 195, 247, 0.3);
        }
        .pg-output-placeholder {
          color: #2a4a62;
          font-style: italic;
        }

        /* Sidebar */
        .pg-sidebar {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }
        .pg-sidebar-card {
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          padding: 16px;
        }
        .pg-sidebar-label {
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          color: var(--bp-dim);
          margin-bottom: 12px;
        }
        .pg-sidebar-text {
          font-size: 12px;
          color: var(--bp-dim);
          line-height: 1.6;
          margin: 0;
        }
        .pg-shortcut {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-size: 12px;
          color: var(--bp-dim);
          margin-bottom: 6px;
        }
        .pg-shortcut:last-child {
          margin-bottom: 0;
        }
        .pg-shortcut code {
          background: rgba(79, 195, 247, 0.1);
          color: var(--bp-accent);
          padding: 2px 6px;
          border-radius: 3px;
          font-size: 11px;
        }
        .pg-sidebar-link {
          display: block;
          color: var(--bp-accent);
          text-decoration: none;
          font-size: 12px;
          margin-bottom: 6px;
          transition: opacity 0.15s;
        }
        .pg-sidebar-link:hover {
          opacity: 0.8;
        }
        .pg-sidebar-link:last-child {
          margin-bottom: 0;
        }
        .pg-stat-row {
          display: flex;
          justify-content: space-between;
          padding: 4px 0;
          font-size: 12px;
          color: var(--bp-dim);
        }
        .pg-stat-row strong {
          color: var(--bp-heading);
          font-variant-numeric: tabular-nums;
        }

        /* Mobile */
        @media (max-width: 900px) {
          .pg-content {
            grid-template-columns: 1fr;
            padding: 16px;
          }
          .pg-sidebar {
            order: -1;
            grid-column: 1;
          }
          .pg-header {
            padding: 16px;
          }
          .pg-title {
            font-size: 20px;
          }
        }

        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  );
}
