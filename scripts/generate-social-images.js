#!/usr/bin/env node

// Generate social media images for blog posts
// This is a placeholder - would use puppeteer or similar in production

const fs = require('fs');
const path = require('path');

const PUBLIC_DIR = path.join(__dirname, '../public');
const IMAGES_DIR = path.join(PUBLIC_DIR, 'images');

// Ensure directories exist
fs.mkdirSync(PUBLIC_DIR, { recursive: true });
fs.mkdirSync(IMAGES_DIR, { recursive: true });

console.log('Social image generation would happen here...');
console.log('For now, create placeholder images manually or use existing ones.');
console.log('Images should be 1200x630 pixels for optimal social sharing.');

// Create basic HTML template for social images
const createSocialImageHTML = (title, subtitle, theme = 'security') => `
<!DOCTYPE html>
<html>
<head>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    width: 1200px;
    height: 630px;
    background: linear-gradient(135deg, #0a0a0a 0%, #111 100%);
    color: #e0e0e0;
    font-family: 'Inter', sans-serif;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }
  .container {
    text-align: center;
    max-width: 900px;
    padding: 40px;
  }
  .title {
    font-size: 52px;
    font-weight: 700;
    line-height: 1.2;
    margin-bottom: 24px;
    color: #fff;
  }
  .subtitle {
    font-size: 28px;
    color: #00ff88;
    margin-bottom: 40px;
  }
  .logo {
    position: absolute;
    bottom: 40px;
    right: 40px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 32px;
    font-weight: 700;
  }
  .logo .dot { color: #00ff88; }
  .accent {
    position: absolute;
    top: 0;
    left: 0;
    width: 6px;
    height: 100%;
    background: #00ff88;
  }
  ${theme === 'security' ? `
  .security-pattern {
    position: absolute;
    top: 20px;
    left: 40px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 14px;
    color: #00ff88;
    opacity: 0.3;
    line-height: 1.4;
  }
  ` : ''}
</style>
</head>
<body>
  <div class="accent"></div>
  ${theme === 'security' ? `
  <div class="security-pattern">
    [SECURITY_BREACH]<br>
    > analyzing_incident...<br>
    > threat_vector: identified<br>
    > mitigation: active<br>
    [SECURED]
  </div>
  ` : ''}
  <div class="container">
    <h1 class="title">${title}</h1>
    <p class="subtitle">${subtitle}</p>
  </div>
  <div class="logo">repo<span class="dot">.</span>box</div>
</body>
</html>
`;

// Generate specific images
const templates = {
  'circleci-breach-social.html': createSocialImageHTML(
    'CircleCI Token Breach Teardown',
    'How Session Hijacking Compromised Developer Pipelines',
    'security'
  ),
  'blog-social.html': createSocialImageHTML(
    'repo.box blog',
    'Irregular thoughts on building things'
  )
};

// Write HTML templates
for (const [filename, content] of Object.entries(templates)) {
  fs.writeFileSync(path.join(IMAGES_DIR, filename), content);
  console.log(`✓ Generated ${filename}`);
}

console.log('\nTo generate PNG images, use:');
console.log('puppeteer or similar to convert HTML → PNG');
console.log('Or use online tools like htmlcsstoimage.com');