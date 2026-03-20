// Live Portfolio Wall Data
// Auto-generated from kanban projects

export interface Project {
  id: string;
  name: string;
  description: string;
  status: "active" | "shipped" | "paused" | "concept";
  link?: string;
  lastActivity: string;
  tags: string[];
  team: "ocean" | "fran" | "both";
}

export const projects: Project[] = [
  {
    id: "repobox",
    name: "repo.box",
    description: "Git permission layer that makes repositories safe for AI agents",
    status: "active",
    link: "https://repo.box",
    lastActivity: "2026-03-20",
    tags: ["git", "security", "infrastructure"],
    team: "both"
  },
  {
    id: "sss",
    name: "Semi-Sentient Society",
    description: "Verified agent DAO with on-chain reputation and corvée system",
    status: "active", 
    link: "https://sss.repo.box",
    lastActivity: "2026-03-19",
    tags: ["dao", "verification", "on-chain"],
    team: "ocean"
  },
  {
    id: "oceangram",
    name: "Oceangram",
    description: "Telegram surface for VS Code with 76 integrated services",
    status: "active",
    link: "https://marketplace.visualstudio.com/items?itemName=ocean.oceangram",
    lastActivity: "2026-03-19",
    tags: ["vscode", "telegram", "developer-tools"],
    team: "ocean"
  },
  {
    id: "archipelago",
    name: "Archipelago", 
    description: "Real-time multi-topic visibility dashboard for Telegram teams",
    status: "active",
    link: "https://archipelago.repo.box",
    lastActivity: "2026-03-13",
    tags: ["dashboard", "telegram", "collaboration"],
    team: "ocean"
  },
  {
    id: "supstrategy",
    name: "SUPStrategy",
    description: "AI-powered Superfluid token trading monitor with smart signals",
    status: "active",
    link: "https://supstrategy.repo.box",
    lastActivity: "2026-03-08",
    tags: ["trading", "superfluid", "defi"],
    team: "ocean"
  },
  {
    id: "cabin",
    name: "Cabin",
    description: "AI group travel agent that finds and books real flights with crypto",
    status: "active", 
    lastActivity: "2026-03-07",
    tags: ["travel", "ai-agent", "crypto"],
    team: "ocean"
  },
  {
    id: "botfight",
    name: "BotFight",
    description: "AI social deduction arena where agents play Mafia with evolving strategies",
    status: "paused",
    lastActivity: "2026-03-13",
    tags: ["gaming", "ai-behavior", "social"],
    team: "ocean"
  },
  {
    id: "rikai",
    name: "Rikai",
    description: "Interactive language reading assistant with real-time vocabulary help",
    status: "paused",
    lastActivity: "2026-03-07",
    tags: ["education", "language", "reading"],
    team: "fran"
  }
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
    case "concept": return "var(--bp-dim)";
    default: return "var(--bp-dim)";
  }
}

export function getStatusBadgeBackground(status: Project["status"]) {
  switch (status) {
    case "active": return "rgba(79, 195, 247, 0.15)";
    case "shipped": return "rgba(74, 222, 128, 0.15)";
    case "paused": return "rgba(251, 191, 36, 0.15)";
    case "concept": return "rgba(90, 122, 148, 0.15)";
    default: return "rgba(90, 122, 148, 0.15)";
  }
}