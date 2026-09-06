// Portfolio Wall data.
//
// MAINTENANCE RULE (REPOBOX-PROOF-001, 2026-09-06): every `link` and every
// `demo.url` in this file must return a real response for the project it
// claims to show. Before adding or reviving an entry, fetch the URL. If a host
// stops answering, move the entry to status "retired" and set `retiredNote`
// with the date and the observed response — do NOT leave it presented as live
// work, and do NOT delete the entry, because the work genuinely happened.
//
// Link sweep performed 2026-09-06:
//   https://repo.box                     200
//   https://runyard.repo.box             200
//   https://frontier.repo.box            200
//   https://frontier-pm.repo.box         200
//   https://github.com/yolo-maxi/frontier-orderbook  200
//   https://github.com/yolo-maxi/concierge           200
//   https://sss.repo.box                 000 (no response)
//   https://archipelago.repo.box         502
//   https://supstrategy.repo.box         000 (no response)
//   https://rikai.repo.box               000 (no response)
//   marketplace.visualstudio.com Oceangram listing   404
//   https://cabin.ai                     200 but a Spaceship.com parking page
//
// `lastActivity` is the last commit date of the local checkout where one
// exists, not a hand-entered date.

export interface Project {
  id: string;
  name: string;
  description: string;
  status: "active" | "shipped" | "paused" | "concept" | "retired";
  link?: string;
  lastActivity: string;
  /** Required for status "retired": what was observed, and when. */
  retiredNote?: string;
  tags: string[];
  team: "ocean" | "fran" | "both";
  demo?: {
    type: "iframe" | "screenshot" | "gif" | "cli";
    url?: string;
    screenshots?: string[];
    gifUrl?: string;
    cliCommand?: string;
    previewText?: string;
  };
}

export const projects: Project[] = [
  {
    id: "runyard",
    name: "RunYard",
    description:
      "Control plane for long-running agent work on your own machines: runs, runners, boards and approvals, so an agent's work is durable and reviewable.",
    status: "active",
    link: "https://runyard.repo.box",
    lastActivity: "2026-09-05",
    tags: ["agents", "orchestration", "infrastructure"],
    team: "both",
  },
  {
    id: "frontier",
    name: "Frontier",
    description:
      "An on-chain order book plus a prediction-market app built on it. Contracts are source-available under BUSL-1.1.",
    status: "active",
    link: "https://frontier.repo.box",
    lastActivity: "2026-09-05",
    tags: ["onchain", "orderbook", "defi"],
    team: "both",
  },
  {
    id: "concierge",
    name: "Concierge",
    description:
      "A deliberately powerless landing-page agent: it answers from one page brief and holds no keys, no tools and no database.",
    status: "active",
    link: "https://github.com/yolo-maxi/concierge",
    lastActivity: "2026-09-05",
    tags: ["agents", "widget", "sandbox"],
    team: "ocean",
  },
  {
    id: "repobox",
    name: "repo.box",
    description:
      "Git permission layer that makes a repository safe to hand to an AI agent: signed commits checked against a config that lives in the repo.",
    status: "active",
    link: "https://repo.box",
    lastActivity: "2026-09-05",
    tags: ["git", "security", "infrastructure"],
    team: "both",
    demo: {
      type: "cli",
      cliCommand: "repobox init && repobox check",
      previewText: "CLI tool that protects git repos from AI agent mistakes",
    },
  },
  {
    id: "botfight",
    name: "BotFight",
    description:
      "AI social deduction arena where agents play Mafia and their strategies evolve between rounds.",
    status: "paused",
    lastActivity: "2026-02-27",
    tags: ["gaming", "ai-behavior", "social"],
    team: "ocean",
  },
  {
    id: "cabin",
    name: "Cabin",
    description:
      "AI group travel agent that searched flights and settled in crypto.",
    status: "retired",
    lastActivity: "2026-04-22",
    retiredNote:
      "cabin.ai is no longer ours — as of 2026-09-06 it resolves to a Spaceship.com domain-for-sale page. No hosted instance remains.",
    tags: ["travel", "ai-agent", "crypto"],
    team: "ocean",
  },
  {
    id: "sss",
    name: "Semi-Sentient Society",
    description:
      "Verified-agent DAO experiment with on-chain reputation and a corvée task system.",
    status: "retired",
    lastActivity: "2026-03-19",
    retiredNote:
      "sss.repo.box has not responded since at least 2026-09-05 (no HTTP response on 2026-09-06). No source checkout remains on the build host.",
    tags: ["dao", "verification", "on-chain"],
    team: "ocean",
  },
  {
    id: "oceangram",
    name: "Oceangram",
    description: "Telegram surface for VS Code.",
    status: "retired",
    lastActivity: "2026-03-19",
    retiredNote:
      "The VS Code Marketplace listing returns 404 as of 2026-09-06 and no source checkout remains on the build host.",
    tags: ["vscode", "telegram", "developer-tools"],
    team: "ocean",
  },
  {
    id: "archipelago",
    name: "Archipelago",
    description:
      "Real-time multi-topic visibility dashboard for Telegram teams.",
    status: "retired",
    lastActivity: "2026-03-28",
    retiredNote:
      "archipelago.repo.box returns 502 as of 2026-09-06. The source checkout still exists locally; the deployment does not.",
    tags: ["dashboard", "telegram", "collaboration"],
    team: "ocean",
  },
  {
    id: "supstrategy",
    name: "SUPStrategy",
    description: "Superfluid token trading monitor with generated signals.",
    status: "retired",
    lastActivity: "2026-03-08",
    retiredNote:
      "supstrategy.repo.box gives no HTTP response as of 2026-09-06 and no source checkout remains on the build host.",
    tags: ["trading", "superfluid", "defi"],
    team: "ocean",
  },
  {
    id: "rikai",
    name: "Rikai",
    description:
      "Interactive language reading assistant with inline vocabulary help.",
    status: "retired",
    lastActivity: "2026-03-07",
    retiredNote:
      "rikai.repo.box gives no HTTP response as of 2026-09-06 and no source checkout remains on the build host.",
    tags: ["education", "language", "reading"],
    team: "fran",
  },
];

export function getProjectsByStatus(status: Project["status"]) {
  return projects.filter(p => p.status === status);
}

export function getProjectsByTeam(team: Project["team"]) {
  return projects.filter(p => p.team === team);
}

export function getActiveProjects() {
  return getProjectsByStatus("active");
}

export function getStatusBadgeColor(status: Project["status"]) {
  switch (status) {
    case "active": return "var(--bp-accent)";
    case "shipped": return "#4ade80";
    case "paused": return "#fbbf24";
    case "retired": return "var(--bp-dim)";
    case "concept": return "var(--bp-dim)";
    default: return "var(--bp-dim)";
  }
}

export function getStatusBadgeBackground(status: Project["status"]) {
  switch (status) {
    case "active": return "rgba(79, 195, 247, 0.15)";
    case "shipped": return "rgba(74, 222, 128, 0.15)";
    case "paused": return "rgba(251, 191, 36, 0.15)";
    case "retired": return "rgba(90, 122, 148, 0.15)";
    case "concept": return "rgba(90, 122, 148, 0.15)";
    default: return "rgba(90, 122, 148, 0.15)";
  }
}
