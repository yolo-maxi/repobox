#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { marked } = require('marked');
const matter = require('gray-matter');

const POSTS_DIR = path.join(__dirname, 'posts');
const OUT_DIR = '/var/www/repo.box/subdomains/root/blog';
const FEED_PATH = '/var/www/repo.box/subdomains/root/feed.xml';
const SITE_URL = 'https://repo.box';

// Ensure output dir
fs.mkdirSync(OUT_DIR, { recursive: true });

// Read and parse all posts
const posts = fs.readdirSync(POSTS_DIR)
  .filter(f => f.endsWith('.md'))
  .map(f => {
    const raw = fs.readFileSync(path.join(POSTS_DIR, f), 'utf8');
    const { data, content } = matter(raw);
    const slug = f.replace(/\.md$/, '').replace(/^\d{4}-\d{2}-\d{2}-/, '');
    return {
      slug,
      title: data.title || slug,
      date: data.date ? new Date(data.date) : new Date(),
      description: data.description || '',
      tags: data.tags || [],
      html: marked(content),
      filename: f,
    };
  })
  .sort((a, b) => b.date - a.date);

const formatDate = d => d.toISOString().split('T')[0];
const rfcDate = d => d.toUTCString();

// Shared styles
const STYLES = `
*{margin:0;padding:0;box-sizing:border-box}
:root{--bg:#0a0a0a;--surface:#111;--border:#1a1a1a;--text:#e0e0e0;--dim:#666;--accent:#00ff88;--accent2:#00ccff}
body{background:var(--bg);color:var(--text);font-family:'Inter',system-ui,sans-serif;line-height:1.6;min-height:100vh}
a{color:var(--accent);text-decoration:none;transition:opacity .2s}
a:hover{opacity:.8}
.container{max-width:720px;margin:0 auto;padding:3rem 1.5rem 4rem}
.back{font-family:'JetBrains Mono',monospace;font-size:.85rem;color:var(--dim);display:inline-block;margin-bottom:2rem;transition:color .2s}
.back:hover{color:var(--accent)}
.logo{font-family:'JetBrains Mono',monospace;font-size:1.8rem;font-weight:700;margin-bottom:.5rem}
.logo .dot{color:var(--accent)}
h2.section{font-size:.85rem;text-transform:uppercase;letter-spacing:.12em;color:var(--dim);margin-bottom:1.25rem;font-weight:500}
footer{margin-top:4rem;padding-top:1.5rem;border-top:1px solid var(--border);color:var(--dim);font-size:.8rem;display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:.5rem}
@media(max-width:500px){
  .container{padding:2rem 1.25rem 3rem}
  .logo{font-size:1.5rem}
  footer{flex-direction:column;align-items:flex-start}
}
`;

const INDEX_STYLES = `
.post-item{background:var(--surface);border:1px solid var(--border);border-radius:10px;padding:1.25rem 1.5rem;margin-bottom:.75rem;transition:border-color .2s;display:block;color:inherit}
.post-item:hover{border-color:#333;opacity:1}
.post-header{display:flex;justify-content:space-between;align-items:baseline;margin-bottom:.25rem}
.post-title{font-weight:600;font-size:1.05rem;color:var(--text)}
.post-date{color:var(--dim);font-size:.75rem;font-family:'JetBrains Mono',monospace}
.post-desc{color:#999;font-size:.9rem}
.post-tags{margin-top:.4rem;display:flex;gap:.4rem;flex-wrap:wrap}
.post-tag{color:var(--accent);font-size:.7rem;font-family:'JetBrains Mono',monospace;background:rgba(0,255,136,.08);padding:.1rem .4rem;border-radius:3px}
.intro{color:#bbb;font-size:1.05rem;margin-bottom:2rem;max-width:600px}
`;

const POST_STYLES = `
.post-meta{color:var(--dim);font-size:.85rem;font-family:'JetBrains Mono',monospace;margin-bottom:2rem}
.post-body{max-width:680px}
.post-body h1{font-size:1.8rem;font-weight:700;margin-bottom:.5rem;line-height:1.2}
.post-body h2{font-size:1.3rem;font-weight:600;margin:2rem 0 .75rem;color:var(--text)}
.post-body h3{font-size:1.1rem;font-weight:600;margin:1.5rem 0 .5rem;color:var(--text)}
.post-body p{margin-bottom:1rem;color:#ccc;font-size:1rem;line-height:1.7}
.post-body ul,.post-body ol{margin:0 0 1rem 1.5rem;color:#ccc}
.post-body li{margin-bottom:.4rem;line-height:1.6}
.post-body blockquote{border-left:3px solid var(--accent);padding:.5rem 1rem;margin:1rem 0;color:#999;background:rgba(0,255,136,.03);border-radius:0 6px 6px 0}
.post-body code{font-family:'JetBrains Mono',monospace;font-size:.85em;background:var(--surface);padding:.15rem .4rem;border-radius:4px;border:1px solid var(--border)}
.post-body pre{background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:1.25rem;margin:1rem 0;overflow-x:auto}
.post-body pre code{background:none;border:none;padding:0;font-size:.85rem;line-height:1.5}
.post-body img{max-width:100%;border-radius:8px;margin:1rem 0}
.post-body a{color:var(--accent);border-bottom:1px solid rgba(0,255,136,.2)}
.post-body a:hover{border-bottom-color:var(--accent)}
.post-body hr{border:none;border-top:1px solid var(--border);margin:2rem 0}
`;

const htmlHead = (title, description) => `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>${title}</title>
<meta name="description" content="${description}">
<link rel="alternate" type="application/rss+xml" title="repo.box blog" href="/feed.xml">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">`;

const htmlFooter = `<footer>
  <span>© 2026 repo.box</span>
  <span>Built with caffeine and curiosity</span>
</footer>`;

// --- Generate index page ---
const indexHtml = `${htmlHead('Blog — repo.box', 'Irregular thoughts on building things.')}
<style>${STYLES}${INDEX_STYLES}</style>
</head>
<body>
<div class="container">
<a href="/" class="back">← repo.box</a>
<header>
  <div class="logo">repo<span class="dot">.</span>box <span style="font-size:.5em;color:var(--dim);font-weight:400">/ blog</span></div>
</header>
<p class="intro">Irregular thoughts on building things. Unpolished, unfiltered, occasionally useful.</p>

<section>
  <h2 class="section">Posts</h2>
  ${posts.map(p => `<a href="/blog/${p.slug}.html" class="post-item">
    <div class="post-header">
      <div class="post-title">${p.title}</div>
      <span class="post-date">${formatDate(p.date)}</span>
    </div>
    ${p.description ? `<div class="post-desc">${p.description}</div>` : ''}
    ${p.tags.length ? `<div class="post-tags">${p.tags.map(t => `<span class="post-tag">${t}</span>`).join('')}</div>` : ''}
  </a>`).join('\n  ')}
</section>

${htmlFooter}
</div>
</body>
</html>`;

fs.writeFileSync(path.join(OUT_DIR, 'index.html'), indexHtml);
console.log('✓ blog/index.html');

// --- Generate individual post pages ---
for (const p of posts) {
  const postHtml = `${htmlHead(`${p.title} — repo.box`, p.description)}
<style>${STYLES}${POST_STYLES}</style>
</head>
<body>
<div class="container">
<a href="/blog/" class="back">← blog</a>
<article class="post-body">
  <h1>${p.title}</h1>
  <div class="post-meta">${formatDate(p.date)}${p.tags.length ? ' · ' + p.tags.join(', ') : ''}</div>
  ${p.html}
</article>
${htmlFooter}
</div>
</body>
</html>`;

  fs.writeFileSync(path.join(OUT_DIR, `${p.slug}.html`), postHtml);
  console.log(`✓ blog/${p.slug}.html`);
}

// --- Generate RSS feed ---
const feedXml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>repo.box blog</title>
  <link>${SITE_URL}/blog/</link>
  <description>Irregular thoughts on building things.</description>
  <language>en-us</language>
  <lastBuildDate>${rfcDate(new Date())}</lastBuildDate>
  <atom:link href="${SITE_URL}/feed.xml" rel="self" type="application/rss+xml"/>
  ${posts.map(p => `<item>
    <title>${p.title}</title>
    <link>${SITE_URL}/blog/${p.slug}.html</link>
    <guid isPermaLink="true">${SITE_URL}/blog/${p.slug}.html</guid>
    <pubDate>${rfcDate(p.date)}</pubDate>
    <description><![CDATA[${p.description || p.html.slice(0, 500)}]]></description>
  </item>`).join('\n  ')}
</channel>
</rss>`;

fs.writeFileSync(FEED_PATH, feedXml);
console.log('✓ feed.xml');

console.log(`\nBuilt ${posts.length} post(s) → ${OUT_DIR}`);
