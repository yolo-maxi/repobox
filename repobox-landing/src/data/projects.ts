// The studio's current work. One list, rendered by the homepage "What we're
// building" section and by /projects. Nothing else on the site lists projects.
//
// RULES (2026-09-11, supersedes the retired-entries rule of 2026-09-06):
//   * Only work that is being built or run right now belongs here. Retired,
//     paused and prototype work is removed from this file, not relabelled;
//     the case studies on /proof carry the history.
//   * Every href below must answer for the project it claims to show. Fetch
//     it before adding it. If it stops answering, remove the link (or the
//     entry) rather than leaving a dead receipt.
//   * Keep it short. This is a curated surface, not an inventory.
//
// Link sweep performed 2026-09-11:
//   https://runyard.repo.box                          200
//   https://concierge.repo.box                        200
//   https://github.com/yolo-maxi/concierge            200 (public)
//   https://repo.box                                  200
//   https://frontier.repo.box                         200
//   https://frontier-pm.repo.box                      200
//   https://github.com/yolo-maxi/frontier-orderbook   200 (public)

export type ProjectStatus = "active" | "shipped";

export interface ProjectLink {
  label: string;
  href: string;
}

export interface Project {
  id: string;
  name: string;
  /** active: being built right now. shipped: live and maintained, not changing daily. */
  status: ProjectStatus;
  summary: string;
  links: ProjectLink[];
  tags: string[];
}

export const STATUS_LABEL: Record<ProjectStatus, string> = {
  active: "active",
  shipped: "shipped",
};

export const projects: Project[] = [
  {
    id: "runyard",
    name: "RunYard",
    status: "active",
    summary:
      "A control plane for long-running agent work on your own machines. Runs, runners, boards and approvals, so an agent's work is durable and reviewable instead of a chat log you lost.",
    links: [{ label: "runyard.repo.box", href: "https://runyard.repo.box" }],
    tags: ["agents", "orchestration", "infrastructure"],
  },
  {
    id: "concierge",
    name: "Concierge",
    status: "active",
    summary:
      "A deliberately powerless landing-page agent. It answers from one page brief and holds no keys, no tools and no database; the interesting engineering is everything it is not allowed to do.",
    links: [
      { label: "concierge.repo.box", href: "https://concierge.repo.box" },
      { label: "source", href: "https://github.com/yolo-maxi/concierge" },
    ],
    tags: ["agents", "widget", "sandbox"],
  },
  {
    id: "repobox",
    name: "repo.box",
    status: "active",
    summary:
      "The thing this domain is named after: a permission layer that makes a git repository safe to hand to an agent. Every commit is signed and checked against a config file that lives in the repo.",
    links: [
      { label: "what it does", href: "/git" },
      { label: "try the config", href: "/playground" },
    ],
    tags: ["git", "security", "infrastructure"],
  },
  {
    id: "frontier",
    name: "Frontier",
    status: "shipped",
    summary:
      "An on-chain order book, plus a prediction-market app on top of it. Contracts are source-available under BUSL-1.1 rather than described in a pitch deck.",
    links: [
      { label: "frontier.repo.box", href: "https://frontier.repo.box" },
      { label: "the app", href: "https://frontier-pm.repo.box" },
      { label: "source", href: "https://github.com/yolo-maxi/frontier-orderbook" },
    ],
    tags: ["onchain", "orderbook", "defi"],
  },
];
