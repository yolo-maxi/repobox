# repo.box Blog System

Static blog generator for repo.box with built-in SEO optimization and social sharing support.

## Quick Start

```bash
# Add new blog post
cd repobox-blog/posts
# Create: YYYY-MM-DD-slug.md

# Build and deploy
./build.sh
```

## File Structure

```
repobox-blog/
├── posts/                  # Markdown blog posts
├── build.js               # Static site generator
├── build.sh               # Build script
├── package.json           # Dependencies
└── README.md              # This file

Generated output → /var/www/repo.box/subdomains/root/blog/
```

## Writing Posts

### Frontmatter Format

```markdown
---
title: "Your Post Title"
date: 2026-03-30
description: "SEO-optimized description (max 160 chars)"
tags: [security, incident-response, access-control]
---

Post content in Markdown...
```

### Filename Convention

- **Format:** `YYYY-MM-DD-slug.md`
- **Examples:**
  - `2026-03-30-circleci-token-breach-teardown.md`
  - `2026-04-15-supply-chain-security-lessons.md`

### Special Formatting

#### Visual Diagrams
```markdown
<div class="visual-break">
<pre class="diagram">
ASCII art diagrams go here
System A → System B → Compromise
    ↓         ↓           ↓
[Detail]  [Detail]   [Impact]
</pre>
</div>
```

#### Code Blocks
Standard markdown fenced code blocks are supported.

## SEO Features

### Automatic Meta Tags
- Open Graph (Facebook)
- Twitter Cards  
- SEO keywords from tags
- Canonical URLs
- Schema.org structured data (planned)

### Social Images
- Custom social sharing images per post
- Fallback to generic blog image
- 1200x630 px optimal size

### RSS Feed
Automatically generated at `/feed.xml` with all posts.

## Incident Teardown Posts

### Template Location
`/content/templates/incident-teardown.md` - comprehensive template for security incident analysis.

### Required Sections
1. **Incident Summary** - Timeline and scope
2. **Access Control Analysis** - Technical breakdown  
3. **Blast Radius Assessment** - Impact analysis
4. **Resolution & Prevention** - Response and fixes
5. **repo.box Prevention Architecture** - Our solution
6. **Industry Impact** - Broader implications

### SEO Targeting
- Primary: "[Company] security breach analysis"
- Secondary: "access control failure prevention"  
- Long-tail: "[Specific incident] lessons learned"

## Build Process

### Dependencies
```json
{
  "marked": "^17.0.2",    // Markdown parsing
  "gray-matter": "^4.0.3" // Frontmatter parsing
}
```

### Build Output
- **HTML pages:** Individual post pages + index
- **RSS feed:** XML feed for all posts
- **Social optimization:** Meta tags and structured data
- **Mobile responsive:** CSS optimized for all devices

### Deployment
Static files deployed to `/var/www/repo.box/subdomains/root/blog/`

Access at: `https://repo.box/blog/`

## Content Guidelines

### Writing Style
- **Technical depth:** Assume engineering audience
- **Word count:** 1200-1500 words for teardowns, 800+ for regular posts  
- **Tone:** Analytical, not inflammatory
- **Evidence:** Only publicly documented information

### Security Content
- **No speculation** on unreported details
- **No confidential information** from any source
- **Attribution required** for all technical claims
- **Actionable insights** - what can readers learn/apply

### repo.box Integration
- **Always end with CTA** to /trust or /packages
- **Specific comparisons** - not generic "we're better"
- **Technical accuracy** - engineering team must review security claims
- **Value focus** - how does our approach solve the problem

## Performance

### Optimization Features
- **Minimal CSS:** Single concatenated stylesheet
- **No JavaScript:** Pure HTML/CSS for fast loading
- **Font optimization:** Google Fonts with preconnect
- **Image optimization:** WebP support (planned)

### Analytics Integration
Ready for Google Analytics, Plausible, or similar (not included by default).

## Publishing Workflow

### Regular Posts
1. Write post in `posts/` directory
2. Run `./build.sh` to generate static files
3. Commit changes and push

### Incident Teardowns
1. Research phase using public sources only
2. Use incident teardown template
3. Technical review by engineering team
4. Generate custom social images
5. SEO optimization check
6. Publish and promote

## Maintenance

### Monthly Tasks
- Review post performance
- Update SEO metadata as needed
- Check for broken links
- Plan upcoming content

### Quarterly Tasks  
- Template improvements based on feedback
- Performance optimization
- Social sharing analysis
- Content strategy review

## Future Enhancements

### Planned Features
- **Interactive diagrams:** D3.js visualizations
- **Comment system:** GitHub-based discussions
- **Search functionality:** Client-side search
- **Related posts:** Automatic content suggestions
- **Newsletter signup:** Email list integration

### Technical Debt
- **Image optimization:** Automated WebP conversion
- **Build performance:** Incremental builds for large post counts
- **Content validation:** Automated checks for required sections
- **Social image generation:** Automated from post metadata

---

*This blog system prioritizes speed, SEO, and technical content quality. Keep it simple, keep it fast, keep it focused.*