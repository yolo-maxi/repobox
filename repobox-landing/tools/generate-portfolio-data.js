#!/usr/bin/env node

/**
 * Repository activity generator.
 *
 * REPOBOX-PROOF-001 (2026-09-06). This script previously scanned
 * /home/xiko/kanban-projects and emitted every kanban markdown file as if it
 * were a shipped product, with the file's HTML config comment
 * ("<!-- Config: Last Task ID: 085 -->") as the public description, and it
 * attached a hardcoded demo link to each id whether or not that host still
 * answered. Its output (public/data/portfolio.json) was frozen at 2026-03-30
 * and rendered on the public /portfolio route.
 *
 * It now does one narrow job: report git activity for repositories that
 * actually exist on this host. Project identity — name, description, status,
 * and outbound link — lives in src/data/projects.ts, which is link-swept by
 * hand. /portfolio merges the two. Nothing here invents a project, and nothing
 * here emits a URL.
 *
 * Usage:  node tools/generate-portfolio-data.js
 * Output: public/data/repo-activity.json
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const OUTPUT_FILE = path.join(__dirname, '..', 'public', 'data', 'repo-activity.json');

/**
 * Project id (as used in src/data/projects.ts) -> local checkout path.
 * A project with no local checkout is simply absent from the output; the
 * /portfolio page renders it without activity data rather than with zeros,
 * because "we cannot see the repo from here" is not the same as "no commits".
 */
const REPOS = {
  repobox: '/home/xiko/repobox',
  runyard: '/home/xiko/runyard',
  concierge: '/home/xiko/concierge',
  cabin: '/home/xiko/cabin',
  botfight: '/home/xiko/botfight',
  archipelago: '/home/xiko/archipelago',
};

function getGitStats(repoPath) {
  if (!fs.existsSync(path.join(repoPath, '.git'))) return null;

  const git = (args) =>
    execSync(`git -C ${JSON.stringify(repoPath)} ${args}`, {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();

  try {
    const commits = parseInt(git('rev-list --count HEAD'), 10);
    const lastCommit = git('log -1 --format=%cI') || null;
    const weeklyCommits = git('log --since="7 days ago" --oneline').split('\n').filter(Boolean).length;
    return {
      commits: Number.isFinite(commits) ? commits : 0,
      lastCommit,
      weeklyCommits,
    };
  } catch (error) {
    console.warn(`  ! git stats unavailable for ${repoPath}: ${error.message}`);
    return null;
  }
}

function generateRepoActivity() {
  const repos = {};
  const missing = [];

  for (const [id, repoPath] of Object.entries(REPOS)) {
    const stats = getGitStats(repoPath);
    if (stats) {
      repos[id] = stats;
      console.log(`  ✓ ${id}: ${stats.commits} commits, ${stats.weeklyCommits} this week`);
    } else {
      missing.push(id);
      console.log(`  – ${id}: no checkout at ${repoPath}, omitted`);
    }
  }

  const payload = {
    generatedAt: new Date().toISOString(),
    note:
      'Git activity only. Project identity and outbound links live in src/data/projects.ts. Projects absent from this file have no local checkout on the build host.',
    repos,
  };

  const outputDir = path.dirname(OUTPUT_FILE);
  if (!fs.existsSync(outputDir)) fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(payload, null, 2) + '\n');

  console.log(`\nWrote ${Object.keys(repos).length} repositories to ${OUTPUT_FILE}`);
  if (missing.length) console.log(`Omitted (no local checkout): ${missing.join(', ')}`);

  return payload;
}

if (require.main === module) {
  try {
    generateRepoActivity();
  } catch (error) {
    console.error('Repository activity generation failed:', error);
    process.exit(1);
  }
}

module.exports = { generateRepoActivity };
