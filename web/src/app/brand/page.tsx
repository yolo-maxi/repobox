'use client';

import { SiteNav } from '@/components/SiteNav';
import { useState } from 'react';

function Swatch({ color, label, value }: { color: string; label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <button onClick={copy} className="brand-swatch">
      <div className="brand-swatch-color" style={{ background: color }} />
      <div className="brand-swatch-info">
        <span className="brand-swatch-label">{label}</span>
        <code className="brand-swatch-value">{copied ? 'Copied!' : value}</code>
      </div>
    </button>
  );
}

function TypeSample({ size, weight, label, sample }: { size: string; weight: number; label: string; sample?: string }) {
  return (
    <div className="brand-type-row">
      <span className="brand-type-label">{label}</span>
      <span style={{ fontSize: size, fontWeight: weight, color: 'var(--bp-heading)' }}>
        {sample || 'repo.box — Git permissions for AI agents'}
      </span>
    </div>
  );
}

export default function BrandPage() {
  return (
    <div className="brand-root">
      <SiteNav />

      <div className="brand-content">
        {/* Hero */}
        <div className="brand-hero">
          <h1>Brand Guidelines</h1>
          <p>Visual identity and design system for repo.box</p>
        </div>

        {/* Logo */}
        <section className="brand-section">
          <h2>Logo</h2>
          <div className="brand-logo-showcase">
            <div className="brand-logo-card dark">
              <span className="brand-logo-text">
                repo<span style={{ color: 'var(--bp-accent)' }}>.</span>box
              </span>
              <span className="brand-logo-context">Primary — dark background</span>
            </div>
            <div className="brand-logo-card light">
              <span className="brand-logo-text" style={{ color: '#0a1628' }}>
                repo<span style={{ color: '#0288d1' }}>.</span>box
              </span>
              <span className="brand-logo-context" style={{ color: '#666' }}>Inverted — light background</span>
            </div>
          </div>
          <div className="brand-rules">
            <h3>Rules</h3>
            <ul>
              <li>The dot is always in accent color — it's the brand mark</li>
              <li>Always lowercase: <code>repo.box</code>, never "Repo.Box" or "REPO.BOX"</li>
              <li>Monospace font only (JetBrains Mono preferred)</li>
              <li>Minimum clear space: 1× the height of the "r" on all sides</li>
              <li>Do not add taglines, icons, or symbols to the logo</li>
            </ul>
          </div>
        </section>

        {/* Colors */}
        <section className="brand-section">
          <h2>Colors</h2>

          <h3>Core Palette</h3>
          <div className="brand-swatches">
            <Swatch color="#0a1628" label="Background" value="#0a1628" />
            <Swatch color="#0d1f35" label="Surface" value="#0d1f35" />
            <Swatch color="rgba(20, 40, 65, 0.6)" label="Card" value="rgba(20, 40, 65, 0.6)" />
            <Swatch color="#4fc3f7" label="Accent" value="#4fc3f7" />
            <Swatch color="#81d4fa" label="Accent Light" value="#81d4fa" />
          </div>

          <h3>Text</h3>
          <div className="brand-swatches">
            <Swatch color="#e8f4fd" label="Heading" value="#e8f4fd" />
            <Swatch color="#b8d4e3" label="Body" value="#b8d4e3" />
            <Swatch color="#7a9ab4" label="Dim / Secondary" value="#7a9ab4" />
          </div>

          <h3>Borders & Overlays</h3>
          <div className="brand-swatches">
            <Swatch color="rgba(50, 100, 160, 0.25)" label="Border" value="rgba(50, 100, 160, 0.25)" />
            <Swatch color="rgba(60, 120, 180, 0.2)" label="Card Border" value="rgba(60, 120, 180, 0.2)" />
            <Swatch color="rgba(79, 195, 247, 0.1)" label="Accent Tint" value="rgba(79, 195, 247, 0.1)" />
            <Swatch color="rgba(79, 195, 247, 0.03)" label="Hover" value="rgba(79, 195, 247, 0.03)" />
          </div>

          <h3>Gold Accent (Interactive / Emphasis)</h3>
          <div className="brand-swatches">
            <Swatch color="#f0b860" label="Gold" value="#f0b860" />
            <Swatch color="rgba(240, 184, 96, 0.15)" label="Gold Tint" value="rgba(240, 184, 96, 0.15)" />
          </div>

          <h3>Card Levels</h3>
          <div className="brand-swatches">
            <Swatch color="rgba(20, 40, 65, 0.6)" label="Card (default)" value="rgba(20, 40, 65, 0.6)" />
            <Swatch color="rgba(25, 50, 80, 0.7)" label="Card Elevated" value="rgba(25, 50, 80, 0.7)" />
          </div>

          <h3>Semantic</h3>
          <div className="brand-swatches">
            <Swatch color="#3fb950" label="Success / Green" value="#3fb950" />
            <Swatch color="#ff8a65" label="Warning / Orange" value="#ff8a65" />
            <Swatch color="#f44336" label="Error / Red" value="#f44336" />
          </div>
        </section>

        {/* Typography */}
        <section className="brand-section">
          <h2>Typography</h2>
          <div className="brand-type-card">
            <div className="brand-type-family">
              <h3>JetBrains Mono</h3>
              <p>Primary typeface for all UI and content. Monospace everywhere — reflecting the developer-first, code-native identity of repo.box.</p>
            </div>
            <TypeSample size="24px" weight={700} label="H1 — 24px / 700" />
            <TypeSample size="16px" weight={600} label="H2 — 16px / 600" />
            <TypeSample size="13px" weight={500} label="Body — 13px / 500" sample="Every commit EVM-signed. Every push permission-checked." />
            <TypeSample size="12px" weight={400} label="Small — 12px / 400" sample="0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048" />
            <TypeSample size="10px" weight={600} label="Label — 10px / 600 / uppercase" sample="RECENT ACTIVITY" />
          </div>
        </section>

        {/* Components */}
        <section className="brand-section">
          <h2>Components</h2>

          <h3>Cards</h3>
          <div className="brand-component-grid">
            <div className="brand-component-demo">
              <div style={{
                background: 'rgba(20, 40, 65, 0.6)',
                border: '1px solid rgba(60, 120, 180, 0.2)',
                borderRadius: 8,
                padding: 16,
              }}>
                <div style={{ fontSize: 10, fontWeight: 600, textTransform: 'uppercase' as const, letterSpacing: '0.1em', color: 'var(--bp-dim)', marginBottom: 12 }}>Card Title</div>
                <p style={{ fontSize: 12, color: 'var(--bp-text)', margin: 0 }}>Card content with subtle frosted background that separates from the page surface.</p>
              </div>
              <code className="brand-component-spec">background: rgba(20, 40, 65, 0.6)<br/>border: 1px solid rgba(60, 120, 180, 0.2)<br/>border-radius: 8px<br/>padding: 16px</code>
            </div>
          </div>

          <h3>Buttons & Pills</h3>
          <div className="brand-component-grid">
            <div className="brand-button-row">
              <button className="brand-demo-btn primary">Primary Action</button>
              <button className="brand-demo-btn solid">Solid Accent</button>
              <button className="brand-demo-btn outline">Outline</button>
              <span className="brand-demo-pill">badge</span>
              <span className="brand-demo-pill highlight">⬡ 401</span>
            </div>
          </div>

          <h3>Navigation Tab</h3>
          <div className="brand-component-grid">
            <div className="brand-tab-row">
              <span className="brand-demo-tab active">Files <span className="brand-demo-tab-count">46</span></span>
              <span className="brand-demo-tab">Commits <span className="brand-demo-tab-count">401</span></span>
              <span className="brand-demo-tab">Users <span className="brand-demo-tab-count">4</span></span>
              <span className="brand-demo-tab">Config</span>
            </div>
          </div>
        </section>

        {/* Spacing */}
        <section className="brand-section">
          <h2>Spacing & Layout</h2>
          <div className="brand-spacing-list">
            <div className="brand-spacing-row"><span>Page max-width</span><code>1280px</code></div>
            <div className="brand-spacing-row"><span>Page padding</span><code>24px 32px</code></div>
            <div className="brand-spacing-row"><span>Card padding</span><code>16px</code></div>
            <div className="brand-spacing-row"><span>Card gap</span><code>16px</code></div>
            <div className="brand-spacing-row"><span>Grid gap</span><code>24px</code></div>
            <div className="brand-spacing-row"><span>Sidebar width</span><code>280px</code></div>
            <div className="brand-spacing-row"><span>Border radius (cards)</span><code>8px</code></div>
            <div className="brand-spacing-row"><span>Border radius (buttons)</span><code>4–6px</code></div>
            <div className="brand-spacing-row"><span>Border radius (pills)</span><code>10–20px</code></div>
          </div>
        </section>

        {/* Iconography */}
        <section className="brand-section">
          <h2>Iconography</h2>
          <p>SVG icons via <a href="https://lucide.dev" target="_blank" rel="noopener" style={{ color: 'var(--bp-accent)' }}>lucide-react</a> — clean, consistent 24×24 stroke icons. Tree-shakeable, 300+ icons available. Used at 16px / 1.5 stroke width throughout.</p>

          <h3>File Type Icons</h3>
          <p style={{ color: 'var(--bp-dim)', fontSize: 13, marginBottom: 16 }}>Each file type maps to a specific lucide icon + language color. See <code style={{ color: 'var(--bp-accent)', fontSize: 12 }}>components/explore/FileIcon.tsx</code> for the full mapping.</p>
          <div className="brand-icon-grid">
            {[
              ['Folder', '#4fc3f7', 'Directory'],
              ['FileCode', '#dea584', 'Rust (.rs)'],
              ['FileCode', '#3178c6', 'TypeScript (.ts)'],
              ['FileCode', '#f7df1e', 'JavaScript (.js)'],
              ['Braces', '#627eea', 'Solidity (.sol)'],
              ['FileCog', '#8fb0c8', 'Config (yml/toml)'],
              ['FileTerminal', '#89e051', 'Shell (.sh)'],
              ['Lock', '#636c76', 'Lock file'],
              ['FileText', '#8fb0c8', 'Markdown (.md)'],
              ['FileJson', '#8fb0c8', 'JSON'],
              ['Globe', '#e34c26', 'HTML'],
              ['Palette', '#563d7c', 'CSS'],
              ['Image', '#8fb0c8', 'Image'],
              ['Package', '#dea584', 'Cargo.toml'],
            ].map(([icon, color, label]) => (
              <div key={label} className="brand-icon-item">
                <span className="brand-icon-emoji" style={{ color, fontSize: 16 }}>●</span>
                <span>{label}</span>
              </div>
            ))}
          </div>

          <h3>Navigation Icons</h3>
          <p style={{ color: 'var(--bp-dim)', fontSize: 13, marginBottom: 16 }}>Used in the mobile hamburger tray and UI elements.</p>
          <div className="brand-icon-grid">
            {[
              ['Home', 'home'], ['Compass', 'explorer'], ['BookOpen', 'docs'],
              ['Play', 'playground'], ['Palette', 'brand'], ['Menu', 'menu'],
              ['X', 'close'], ['GitBranch', 'git'],
            ].map(([icon, label]) => (
              <div key={label} className="brand-icon-item">
                <span className="brand-icon-emoji" style={{ color: 'var(--bp-accent)', fontSize: 14 }}>◆</span>
                <span>{label}</span>
              </div>
            ))}
          </div>
        </section>

        {/* Voice & Tone */}
        <section className="brand-section">
          <h2>Voice & Tone</h2>
          <div className="brand-voice-grid">
            <div className="brand-voice-card do">
              <h3>✅ Do</h3>
              <ul>
                <li>Be technical and precise</li>
                <li>Use developer language naturally</li>
                <li>Keep copy short — let the product speak</li>
                <li>"Every commit EVM-signed"</li>
                <li>"Git permissions for AI agents"</li>
              </ul>
            </div>
            <div className="brand-voice-card dont">
              <h3>❌ Don't</h3>
              <ul>
                <li>Use marketing buzzwords</li>
                <li>"Revolutionary blockchain-powered solution"</li>
                <li>Over-explain — our users are developers</li>
                <li>Use title case in UI labels</li>
                <li>Add unnecessary decorative elements</li>
              </ul>
            </div>
          </div>
        </section>

        {/* CSS Variables Reference */}
        <section className="brand-section">
          <h2>CSS Variables</h2>
          <pre className="brand-code">{`/* Core theme — use these, not raw values */
--bp-bg:      #0a1628;
--bp-surface: #0d1f35;
--bp-border:  rgba(50, 100, 160, 0.25);
--bp-text:    #b8d4e3;
--bp-heading: #e8f4fd;
--bp-dim:     #7a9ab4;
--bp-accent:  #4fc3f7;
--bp-accent2: #81d4fa;
--bp-gold:    #f0b860;
--bp-gold-dim: rgba(240, 184, 96, 0.15);

/* Card levels */
--bp-card:               rgba(20, 40, 65, 0.6);
--bp-card-border:        rgba(60, 120, 180, 0.2);
--bp-card-elevated:      rgba(25, 50, 80, 0.7);
--bp-card-elevated-border: rgba(70, 140, 200, 0.25);

/* Font */
--font-mono: 'JetBrains Mono', 'Fira Code', monospace;`}</pre>
        </section>
      </div>

      <style>{`
        .brand-root {
          min-height: 100vh;
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono, 'JetBrains Mono', monospace);
          font-size: 13px;
        }
        .brand-content {
          max-width: 960px;
          margin: 0 auto;
          padding: 0 32px 80px;
        }
        .brand-hero {
          padding: 48px 0 40px;
          border-bottom: 1px solid var(--bp-border);
          margin-bottom: 48px;
        }
        .brand-hero h1 {
          font-size: 28px;
          font-weight: 700;
          color: var(--bp-heading);
          letter-spacing: -0.5px;
          margin-bottom: 8px;
        }
        .brand-hero p {
          font-size: 14px;
          color: var(--bp-dim);
        }

        .brand-section {
          margin-bottom: 56px;
        }
        .brand-section h2 {
          font-size: 18px;
          font-weight: 700;
          color: var(--bp-heading);
          margin-bottom: 24px;
          padding-bottom: 12px;
          border-bottom: 1px solid var(--bp-border);
        }
        .brand-section h3 {
          font-size: 12px;
          font-weight: 600;
          color: var(--bp-dim);
          text-transform: uppercase;
          letter-spacing: 0.1em;
          margin: 24px 0 12px;
        }
        .brand-section p {
          font-size: 13px;
          color: var(--bp-dim);
          line-height: 1.6;
          margin-bottom: 16px;
        }
        .brand-section ul {
          list-style: none;
          padding: 0;
        }
        .brand-section li {
          padding: 4px 0;
          font-size: 12px;
          color: var(--bp-text);
        }
        .brand-section li::before {
          content: '— ';
          color: var(--bp-dim);
        }
        .brand-rules li::before {
          content: '→ ';
          color: var(--bp-accent);
        }
        .brand-section code {
          color: var(--bp-accent);
          background: rgba(79, 195, 247, 0.08);
          padding: 1px 6px;
          border-radius: 3px;
          font-size: 12px;
        }

        /* Logo */
        .brand-logo-showcase {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
        }
        .brand-logo-card {
          padding: 48px 32px;
          border-radius: 8px;
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
        }
        .brand-logo-card.dark {
          background: var(--bp-bg);
          border: 1px solid var(--bp-border);
        }
        .brand-logo-card.light {
          background: #f5f7fa;
          border: 1px solid #e0e4e8;
        }
        .brand-logo-text {
          font-size: 36px;
          font-weight: 700;
          letter-spacing: -1px;
        }
        .brand-logo-context {
          font-size: 11px;
          color: var(--bp-dim);
        }

        /* Swatches */
        .brand-swatches {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }
        .brand-swatch {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 6px 12px 6px 6px;
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 6px;
          cursor: pointer;
          font-family: inherit;
          transition: border-color 0.15s;
        }
        .brand-swatch:hover {
          border-color: rgba(79, 195, 247, 0.3);
        }
        .brand-swatch-color {
          width: 32px;
          height: 32px;
          border-radius: 4px;
          border: 1px solid rgba(255,255,255,0.1);
          flex-shrink: 0;
        }
        .brand-swatch-info {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }
        .brand-swatch-label {
          font-size: 11px;
          font-weight: 500;
          color: var(--bp-heading);
        }
        .brand-swatch-value {
          font-size: 10px;
          color: var(--bp-dim);
          background: none;
          padding: 0;
        }

        /* Typography */
        .brand-type-card {
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 8px;
          padding: 24px;
        }
        .brand-type-family {
          margin-bottom: 24px;
          padding-bottom: 16px;
          border-bottom: 1px solid var(--bp-border);
        }
        .brand-type-family h3 {
          font-size: 20px;
          color: var(--bp-heading);
          text-transform: none;
          letter-spacing: -0.3px;
          margin: 0 0 8px;
        }
        .brand-type-family p {
          font-size: 12px;
          margin: 0;
        }
        .brand-type-row {
          display: flex;
          align-items: baseline;
          gap: 20px;
          padding: 10px 0;
          border-bottom: 1px solid rgba(50, 100, 160, 0.1);
        }
        .brand-type-row:last-child { border-bottom: none; }
        .brand-type-label {
          width: 200px;
          font-size: 10px;
          color: var(--bp-dim);
          flex-shrink: 0;
        }

        /* Components */
        .brand-component-grid {
          margin-bottom: 16px;
        }
        .brand-component-demo {
          display: flex;
          gap: 20px;
          align-items: flex-start;
        }
        .brand-component-demo > div:first-child {
          flex: 1;
        }
        .brand-component-spec {
          font-size: 10px;
          color: var(--bp-dim);
          background: none;
          padding: 0;
          line-height: 1.8;
          flex-shrink: 0;
          width: 300px;
        }

        .brand-button-row {
          display: flex;
          align-items: center;
          gap: 12px;
          flex-wrap: wrap;
        }
        .brand-demo-btn {
          padding: 8px 16px;
          border-radius: 6px;
          font-size: 12px;
          font-family: inherit;
          cursor: pointer;
          transition: all 0.15s;
        }
        .brand-demo-btn.primary {
          background: rgba(79, 195, 247, 0.15);
          color: var(--bp-accent);
          border: 1px solid rgba(79, 195, 247, 0.25);
          font-weight: 600;
        }
        .brand-demo-btn.primary:hover {
          background: rgba(79, 195, 247, 0.25);
        }
        .brand-demo-btn.solid {
          background: var(--bp-accent);
          color: #0a1628;
          border: none;
          font-weight: 600;
        }
        .brand-demo-btn.outline {
          background: transparent;
          color: var(--bp-dim);
          border: 1px solid var(--bp-border);
        }
        .brand-demo-pill {
          display: inline-flex;
          align-items: center;
          gap: 4px;
          padding: 2px 8px;
          background: rgba(50, 100, 160, 0.15);
          border-radius: 10px;
          font-size: 11px;
          font-weight: 500;
          color: var(--bp-dim);
        }
        .brand-demo-pill.highlight {
          color: var(--bp-accent);
          background: rgba(79, 195, 247, 0.12);
        }

        .brand-tab-row {
          display: flex;
          gap: 2px;
          border-bottom: 1px solid var(--bp-border);
          padding-bottom: 0;
        }
        .brand-demo-tab {
          padding: 8px 16px;
          font-size: 12px;
          font-weight: 500;
          color: var(--bp-dim);
          border-bottom: 2px solid transparent;
          margin-bottom: -1px;
        }
        .brand-demo-tab.active {
          color: var(--bp-accent);
          border-bottom-color: var(--bp-accent);
        }
        .brand-demo-tab-count {
          margin-left: 4px;
          font-size: 10px;
          background: rgba(79, 195, 247, 0.1);
          color: var(--bp-accent);
          padding: 1px 6px;
          border-radius: 10px;
        }

        /* Spacing */
        .brand-spacing-list {
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 8px;
          overflow: hidden;
        }
        .brand-spacing-row {
          display: flex;
          justify-content: space-between;
          padding: 10px 16px;
          border-bottom: 1px solid var(--bp-border);
          font-size: 12px;
        }
        .brand-spacing-row:last-child { border-bottom: none; }
        .brand-spacing-row code {
          background: none;
          padding: 0;
        }

        /* Icons */
        .brand-icon-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
          gap: 8px;
        }
        .brand-icon-item {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 6px;
          font-size: 12px;
        }
        .brand-icon-emoji { font-size: 16px; }

        /* Voice */
        .brand-voice-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
        }
        .brand-voice-card {
          padding: 20px;
          border-radius: 8px;
        }
        .brand-voice-card.do {
          background: rgba(63, 185, 80, 0.08);
          border: 1px solid rgba(63, 185, 80, 0.2);
        }
        .brand-voice-card.dont {
          background: rgba(244, 67, 54, 0.06);
          border: 1px solid rgba(244, 67, 54, 0.15);
        }
        .brand-voice-card h3 {
          text-transform: none;
          letter-spacing: normal;
          color: var(--bp-heading);
          font-size: 14px;
          margin: 0 0 12px;
        }

        /* Code */
        .brand-code {
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 8px;
          padding: 20px;
          font-size: 12px;
          line-height: 1.8;
          color: var(--bp-text);
          overflow-x: auto;
        }

        @media (max-width: 700px) {
          .brand-content { padding: 0 16px 60px; }
          .brand-logo-showcase { grid-template-columns: 1fr; }
          .brand-voice-grid { grid-template-columns: 1fr; }
          .brand-component-demo { flex-direction: column; }
          .brand-component-spec { width: auto; }
          .brand-type-row { flex-direction: column; gap: 4px; }
          .brand-type-label { width: auto; }
        }
      `}</style>
    </div>
  );
}
