#!/usr/bin/env node

/**
 * Portfolio Data Generator for REPO-042
 * Scans kanban files and git repos to generate portfolio.json
 * 
 * Usage: node tools/generate-portfolio-data.js
 * Output: public/data/portfolio.json
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Configuration
const KANBAN_DIR = '/home/xiko/kanban-projects';
const REPO_ROOT = '/home/xiko';
const OUTPUT_FILE = path.join(__dirname, '..', 'public', 'data', 'portfolio.json');

// Define repo mappings and their paths
const REPOS = {
  'repobox': '/home/xiko/repobox',
  'sss': '/home/xiko/sss',
  'oceangram': '/home/xiko/oceangram',
  'cabin': '/home/xiko/cabin',
  'botfight': '/home/xiko/botfight',
  'rikai': '/home/xiko/rikai',
  'agentation': '/home/xiko/agentation',
  'streme': '/home/xiko/streme',
  'flow3d': '/home/xiko/flow3d',
  'archipelago': '/home/xiko/archipelago',
  'supstrategy': '/home/xiko/supstrategy',
  'clawd': '/home/xiko/clawd',
};

// Helper: Get git commit count and last commit date for a repo
function getGitStats(repoPath) {
  try {
    if (!fs.existsSync(path.join(repoPath, '.git'))) {
      return { commits: 0, lastCommit: null, weeklyCommits: 0 };
    }

    // Get total commit count
    const totalCommits = execSync(`cd "${repoPath}" && git rev-list --count HEAD 2>/dev/null || echo "0"`, { encoding: 'utf8' }).trim();
    
    // Get last commit date
    const lastCommit = execSync(`cd "${repoPath}" && git log -1 --format="%ci" 2>/dev/null || echo ""`, { encoding: 'utf8' }).trim();
    
    // Get commits in last 7 days
    const weeklyCommits = execSync(`cd "${repoPath}" && git log --since="7 days ago" --oneline 2>/dev/null | wc -l`, { encoding: 'utf8' }).trim();

    return {
      commits: parseInt(totalCommits) || 0,
      lastCommit: lastCommit || null,
      weeklyCommits: parseInt(weeklyCommits) || 0
    };
  } catch (error) {
    console.warn(`Git stats error for ${repoPath}:`, error.message);
    return { commits: 0, lastCommit: null, weeklyCommits: 0 };
  }
}

// Helper: Determine status based on recent activity
function determineStatus(gitStats, manualStatus = null) {
  // If manual status is set, use it
  if (manualStatus && ['shipped', 'paused', 'concept'].includes(manualStatus)) {
    return manualStatus;
  }

  // Auto-determine based on git activity
  if (gitStats.weeklyCommits > 0) {
    return 'active';
  } else if (gitStats.commits > 10) {
    return 'shipped';
  } else if (gitStats.commits > 0) {
    return 'beta';
  } else {
    return 'concept';
  }
}

// Helper: Parse project from kanban files
function parseKanbanProjects() {
  const projects = [];
  
  try {
    const kanbanFiles = fs.readdirSync(KANBAN_DIR).filter(f => f.endsWith('.md'));
    
    for (const file of kanbanFiles) {
      const projectName = path.basename(file, '.md');
      const content = fs.readFileSync(path.join(KANBAN_DIR, file), 'utf8');
      
      // Extract title from first heading
      const titleMatch = content.match(/^# (.+)$/m);
      const title = titleMatch ? titleMatch[1] : projectName;
      
      // Extract first paragraph as description
      const lines = content.split('\n');
      let description = '';
      for (const line of lines) {
        if (line.trim() && !line.startsWith('#') && !line.startsWith('##')) {
          description = line.trim();
          break;
        }
      }
      
      // Use project name as potential repo key
      const repoPath = REPOS[projectName.toLowerCase()] || null;
      const gitStats = repoPath ? getGitStats(repoPath) : { commits: 0, lastCommit: null, weeklyCommits: 0 };
      
      projects.push({
        id: projectName.toLowerCase(),
        name: title,
        description: description || `${title} project from repo.box studio`,
        status: determineStatus(gitStats),
        repoPath,
        gitStats,
        kanbanFile: file
      });
    }
  } catch (error) {
    console.warn('Error parsing kanban files:', error.message);
  }
  
  return projects;
}

// Helper: Add manual project entries for known repos not in kanban
function addManualProjects(projects) {
  const existingIds = new Set(projects.map(p => p.id));
  
  const manualProjects = [
    {
      id: 'clawd',
      name: 'Clawd Workspace',
      description: 'Personal AI assistant workspace and agent memory system',
      status: null, // Auto-determine
      link: null,
      tags: ['workspace', 'ai-assistant', 'memory']
    },
    {
      id: 'repobox-landing',
      name: 'repo.box Website',
      description: 'Marketing website and documentation for repo.box studio',
      status: null,
      link: 'https://repo.box',
      tags: ['website', 'marketing', 'docs']
    }
  ];

  for (const manual of manualProjects) {
    if (!existingIds.has(manual.id)) {
      const repoPath = REPOS[manual.id] || null;
      const gitStats = repoPath ? getGitStats(repoPath) : { commits: 0, lastCommit: null, weeklyCommits: 0 };
      
      projects.push({
        ...manual,
        status: manual.status || determineStatus(gitStats),
        repoPath,
        gitStats,
        tags: manual.tags || []
      });
    }
  }
  
  return projects;
}

// Helper: Add demo links where available
function addDemoLinks(projects) {
  const demoMappings = {
    'repobox': 'https://repo.box',
    'sss': 'https://sss.repo.box',
    'cabin': 'https://cabin.ai',
    'supstrategy': 'https://supstrategy.repo.box',
    'archipelago': 'https://archipelago.repo.box',
    'rikai': 'https://rikai.repo.box',
    'agentation': 'https://agentation.repo.box',
    'streme': 'https://streme.repo.box',
    'flow3d': 'https://flow3d.repo.box',
    'oceangram': 'https://marketplace.visualstudio.com/items?itemName=ocean.oceangram'
  };

  return projects.map(project => ({
    ...project,
    link: project.link || demoMappings[project.id] || null
  }));
}

// Main generation function
function generatePortfolioData() {
  console.log('🔍 Scanning kanban files...');
  let projects = parseKanbanProjects();
  
  console.log('📂 Adding manual projects...');
  projects = addManualProjects(projects);
  
  console.log('🔗 Adding demo links...');
  projects = addDemoLinks(projects);
  
  console.log('📊 Calculating stats...');
  const stats = {
    totalProjects: projects.length,
    activeProjects: projects.filter(p => p.status === 'active').length,
    shippedProjects: projects.filter(p => p.status === 'shipped').length,
    totalCommits: projects.reduce((sum, p) => sum + p.gitStats.commits, 0),
    weeklyCommits: projects.reduce((sum, p) => sum + p.gitStats.weeklyCommits, 0),
    lastGenerated: new Date().toISOString()
  };
  
  const portfolioData = {
    stats,
    projects: projects.map(p => ({
      id: p.id,
      name: p.name,
      description: p.description,
      status: p.status,
      link: p.link,
      lastCommit: p.gitStats.lastCommit,
      totalCommits: p.gitStats.commits,
      weeklyCommits: p.gitStats.weeklyCommits,
      tags: p.tags || []
    }))
  };
  
  // Ensure output directory exists
  const outputDir = path.dirname(OUTPUT_FILE);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  
  // Write output
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(portfolioData, null, 2));
  
  console.log(`✅ Generated portfolio data:`);
  console.log(`   📁 ${stats.totalProjects} projects`);
  console.log(`   🚀 ${stats.activeProjects} active`);
  console.log(`   📈 ${stats.totalCommits} total commits`);
  console.log(`   📊 ${stats.weeklyCommits} commits this week`);
  console.log(`   💾 Output: ${OUTPUT_FILE}`);
  
  return portfolioData;
}

// Run if called directly
if (require.main === module) {
  try {
    generatePortfolioData();
  } catch (error) {
    console.error('❌ Portfolio generation failed:', error);
    process.exit(1);
  }
}

module.exports = { generatePortfolioData };