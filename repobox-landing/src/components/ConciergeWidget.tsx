"use client";

import Script from "next/script";

/**
 * Concierge on repo.box, briefed on repo.box (packet page "repobox").
 *
 * The embed is served by the same proxy Frontier and concierge.repo.box use,
 * reverse-proxied at /concierge/* by Caddy. Mounting manually (instead of
 * data-* attributes) lets the widget take the site's exact palette and font
 * through themeVars, so it reads as part of repo.box rather than the default
 * midnight preset. The widget holds no keys: it only knows /concierge/chat.
 */
const SITE_THEME = {
  "--cc-bg": "#0a1628",
  "--cc-surface": "#0d1f35",
  "--cc-surface-raised": "#12294a",
  "--cc-border": "rgba(50, 100, 160, 0.25)",
  "--cc-text": "#e8f4fd",
  "--cc-text-muted": "#7a9ab4",
  "--cc-accent": "#4fc3f7",
  "--cc-accent-2": "#81d4fa",
  "--cc-accent-ink": "#06121f",
  "--cc-radius-panel": "8px",
  "--cc-radius-bubble": "8px",
  "--cc-radius-launcher": "6px",
  "--cc-radius-control": "6px",
  "--cc-font-family": "var(--font-mono), 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace",
};

declare global {
  interface Window {
    Concierge?: { mount: (props: Record<string, unknown>) => void };
  }
}

export default function ConciergeWidget() {
  return (
    <Script
      id="concierge-embed"
      src="/concierge/embed.js"
      strategy="afterInteractive"
      onLoad={() => {
        window.Concierge?.mount({
          endpoint: "/concierge/chat",
          pageId: "repobox",
          brandName: "repo.box",
          tagline: "What we build, and how to work with us",
          greeting:
            "Ask what repo.box actually does, what is running right now, or how to work with us. Serious enquiries go to /hire.",
          suggestions: [
            "What do you people actually do?",
            "What is running right now?",
            "How do I hire you?",
            "What is Concierge?",
          ],
          launcher: "pill",
          launcherLabel: "ask repo.box",
          avatar: "none",
          position: "bottom-right",
          themeVars: SITE_THEME,
          creditText: "Concierge · answers from this site only",
        });
      }}
    />
  );
}
