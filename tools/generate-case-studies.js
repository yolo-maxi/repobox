#!/usr/bin/env node

/**
 * Generate case study data for repo.box homepage
 * 
 * This script generates the transformation stories section content based on
 * existing project artifacts and maintains factual accuracy by referencing
 * real commits, releases, and URLs.
 * 
 * Usage: node tools/generate-case-studies.js
 */

const caseStudies = [
  {
    name: 'Oceangram',
    tag: 'telegram',
    animation: 'wave',
    before: 'Single-surface chat prototype',
    after: 'Universal client + developer tools',
    summary: 'From basic chat idea → universal Telegram client with VS Code extension, Mac tray app, and 60+ API endpoints',
    proofSignal: {
      type: 'release',
      url: 'https://github.com/yolo-maxi/oceangram',
      text: 'v0.9.0 release'
    },
    description: 'Universal Telegram client powered by centralized daemon, embeddable in any surface'
  },
  {
    name: 'Cabin',
    tag: 'travel',
    animation: 'network',
    before: 'Manual flight coordination',
    after: 'Automated group booking in chat',
    summary: 'From group travel pain point → AI flight agent with real search API, multi-language support, and USDC payments',
    proofSignal: {
      type: 'live-site',
      url: 'https://cabin.team',
      text: 'cabin.team'
    },
    description: 'AI-powered group travel agent in Telegram — find and book optimal flights for groups'
  },
  {
    name: 'SSS',
    tag: 'crypto',
    animation: 'bricks',
    before: 'Agent identity problem',
    after: 'Verified agent ecosystem + DeFi',
    summary: 'From AI verification concept → complete agent registry with smart contracts, verification flows, and token launches',
    proofSignal: {
      type: 'docs',
      url: '/blog/agent-skills.html',
      text: 'design docs'
    },
    description: 'Curated registry of verified AI agents gating access to exclusive crypto events'
  }
];

function generateCaseStudyHTML() {
  const sectionStart = `  <!-- Case Studies -->
  <section class="reveal" style="margin-bottom:60px;">
    <h2 style="font-size:12px;line-height:20px;text-transform:uppercase;letter-spacing:0.12em;color:#5a7a94;font-weight:500;margin-bottom:20px;">Transformation Stories</h2>

`;

  const cards = caseStudies.map(study => `    <div class="project-card ascii-bg" data-ascii-anim="${study.animation}" style="position:relative;overflow:hidden;background:#0d1f35;border:1px solid rgba(50,100,160,0.25);border-radius:8px;padding:20px;margin-bottom:20px;transition:border-color 0.2s;">
      <canvas class="ascii-canvas"></canvas>
      <svg class="card-border"><rect x="0.5" y="0.5" width="calc(100% - 1px)" height="calc(100% - 1px)"/></svg>
      <div style="position:relative;z-index:2;">
        <div style="display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px;">
          <div style="font-weight:700;font-size:16px;line-height:20px;color:#ffffff;">${study.name}</div>
          <span style="font-size:12px;line-height:20px;color:#4fc3f7;background:rgba(79,195,247,0.15);padding:0 12px;border-radius:2px;font-weight:600;">${study.tag}</span>
        </div>
        <div style="font-size:12px;line-height:20px;color:#b8d4e3;margin-bottom:12px;">
          ${study.summary}
        </div>
        <div style="display:flex;justify-content:space-between;align-items:center;font-size:11px;line-height:16px;">
          <div style="color:#5a7a94;">
            <span style="color:#4fc3f7;">Before:</span> ${study.before}
            <span style="color:#5a7a94;margin:0 8px;">→</span>
            <span style="color:#4fc3f7;">After:</span> ${study.after}
          </div>
          <a href="${study.proofSignal.url}" target="_blank" style="color:#81d4fa;font-weight:600;">${study.proofSignal.text}</a>
        </div>
      </div>
    </div>`).join('\n\n');

  const sectionEnd = `
  </section>`;

  return sectionStart + cards + sectionEnd;
}

function generateDataFile() {
  return {
    meta: {
      generated: new Date().toISOString(),
      version: '1.0.0',
      description: 'Case study data for repo.box transformation stories'
    },
    caseStudies
  };
}

if (require.main === module) {
  console.log('📊 Case Studies Data:');
  console.log(JSON.stringify(generateDataFile(), null, 2));
  console.log('\n📝 HTML Section:');
  console.log(generateCaseStudyHTML());
}

module.exports = { generateCaseStudyHTML, generateDataFile, caseStudies };
