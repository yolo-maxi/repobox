// Agent Showcase Data
// Profiles of AI agents built at repo.box studio

export interface Agent {
  id: string;
  name: string;
  description: string;
  avatar?: string;
  status: "active" | "retired" | "experimental";
  capabilities: string[];
  techStack: string[];
  demoLink?: string;
  demoType?: "telegram" | "web" | "api";
  projects: string[]; // Related projects this agent works on
  created: string; // Creation date
  lastActive: string;
}

export const agents: Agent[] = [
  {
    id: "ocean-vael",
    name: "Ocean Vael",
    description: "Primary studio agent specializing in full-stack development, DevOps, and project orchestration. Handles complex multi-repository work with autonomous decision-making capabilities.",
    avatar: "/agents/ocean-vael.png",
    status: "active",
    capabilities: [
      "Full-stack web development",
      "Multi-repository orchestration",
      "DevOps automation", 
      "Security analysis",
      "Technical writing",
      "Code review and architecture",
      "CLI tool development",
      "Database design",
      "API development"
    ],
    techStack: [
      "TypeScript", "React", "Next.js", "Node.js", "Python", "Rust", 
      "PostgreSQL", "Prisma", "Docker", "Linux", "Git", "Caddy", 
      "Telegram Bot API", "OpenClaw", "PM2"
    ],
    demoLink: "https://t.me/ocean_king_bot",
    demoType: "telegram",
    projects: ["repo.box", "SSS", "Oceangram", "Archipelago", "SUPStrategy", "Cabin", "BotFight"],
    created: "2025-11-15",
    lastActive: "2026-03-30"
  },
  {
    id: "krill",
    name: "Krill",
    description: "Secondary studio agent with opinionated commentary and specialized trading analysis. Provides alternative perspectives and challenges assumptions in technical discussions.",
    avatar: "/agents/krill.png", 
    status: "active",
    capabilities: [
      "Trading analysis",
      "Market commentary",
      "Technical critique",
      "Alternative perspectives",
      "Superfluid protocol expertise",
      "Data analysis",
      "Risk assessment"
    ],
    techStack: [
      "Node.js", "Telegram Bot API", "Superfluid SDK", "Web3.js",
      "PostgreSQL", "OpenClaw", "Trading APIs"
    ],
    demoLink: "https://t.me/Ocean_Krill_bot",
    demoType: "telegram",
    projects: ["SUPStrategy", "Superfluid integrations"],
    created: "2025-12-20",
    lastActive: "2026-03-28"
  },
  {
    id: "supstrategy",
    name: "SUPStrategy Monitor",
    description: "Specialized trading intelligence agent that monitors Superfluid token flows, provides market signals, and executes automated trading strategies with risk management.",
    avatar: "/agents/supstrategy.png",
    status: "active", 
    capabilities: [
      "Superfluid token monitoring",
      "Trading signal generation",
      "Risk assessment",
      "Portfolio analysis",
      "Market trend detection",
      "Automated reporting",
      "Flow rate analysis"
    ],
    techStack: [
      "Python", "Superfluid SDK", "Web3.py", "PostgreSQL", 
      "Chart.js", "TradingView", "Telegram Bot API", "FastAPI"
    ],
    demoLink: "https://supstrategy.repo.box",
    demoType: "web",
    projects: ["SUPStrategy"],
    created: "2026-01-15",
    lastActive: "2026-03-25"
  },
  {
    id: "archipelago-monitor",
    name: "Archipelago Dashboard",
    description: "Real-time monitoring agent for multi-topic Telegram visibility. Aggregates conversations, tracks sentiment, and provides team collaboration insights.",
    avatar: "/agents/archipelago.png",
    status: "active",
    capabilities: [
      "Multi-topic monitoring",
      "Sentiment analysis", 
      "Team collaboration insights",
      "Real-time aggregation",
      "Topic trend detection",
      "Activity reporting"
    ],
    techStack: [
      "Node.js", "React", "Socket.io", "PostgreSQL", "Telegram Bot API",
      "Natural Language Processing", "Chart.js", "WebSocket"
    ],
    demoLink: "https://archipelago.repo.box",
    demoType: "web",
    projects: ["Archipelago"],
    created: "2026-02-01",
    lastActive: "2026-03-20"
  },
  {
    id: "cabin-travel",
    name: "Cabin Travel Agent",
    description: "AI-powered group travel coordination agent that finds flights, manages bookings, handles crypto payments, and coordinates complex multi-person itineraries.",
    avatar: "/agents/cabin.png",
    status: "experimental",
    capabilities: [
      "Flight search and booking",
      "Group travel coordination", 
      "Crypto payment processing",
      "Itinerary optimization",
      "Price monitoring",
      "Travel logistics",
      "USDC transactions"
    ],
    techStack: [
      "Node.js", "Flight APIs", "Stripe", "USDC/Base", "React",
      "Web3.js", "Travel booking systems", "Payment processors"
    ],
    demoLink: "https://cabin.repo.box",
    demoType: "web", 
    projects: ["Cabin"],
    created: "2026-02-10",
    lastActive: "2026-03-07"
  }
];

export function getAgentsByStatus(status: Agent["status"]) {
  return agents.filter(a => a.status === status);
}

export function getAgentById(id: string) {
  return agents.find(a => a.id === id);
}

export function getActiveAgents() {
  return agents.filter(a => a.status === "active");
}

export const agentStats = {
  total: agents.length,
  active: getAgentsByStatus("active").length,
  experimental: getAgentsByStatus("experimental").length,
  retired: getAgentsByStatus("retired").length,
  totalCapabilities: agents.reduce((sum, agent) => sum + agent.capabilities.length, 0),
  totalProjects: [...new Set(agents.flatMap(agent => agent.projects))].length,
  primaryTechStack: [
    ...new Set(agents.flatMap(agent => agent.techStack))
  ].slice(0, 12) // Top 12 most common technologies
};