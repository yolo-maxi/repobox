# README Rendering Polish Specification

**Version:** 1.0  
**Author:** PM Agent  
**Date:** March 21, 2026  
**Status:** Draft

## Overview

This specification defines the requirements and implementation approach for upgrading the README tab in the repo.box explorer from basic ReactMarkdown rendering to GitHub-style polished markdown with syntax highlighting, proper table rendering, image support, anchor links, and professional styling.

## Current State Analysis

### Existing Implementation
- **Location:** `/web/src/app/explore/[address]/[name]/page.tsx`
- **Current Tech Stack:**
  - `react-markdown` v10.1.0
  - `remark-gfm` v4.0.1 (GitHub Flavored Markdown)
  - Basic custom components for code/tables/links
  - Minimal CSS classes (explore-* prefixed)

### Current Limitations
1. **No syntax highlighting** - Code blocks render as plain text
2. **Basic table styling** - No GitHub-like table appearance
3. **No anchor links** - Headings aren't navigable
4. **Inconsistent styling** - Doesn't match GitHub's README aesthetic
5. **Poor code block UX** - No copy buttons, language indicators
6. **Limited image handling** - No zoom, captions, or proper responsive behavior

## Technical Approach

### 1. Syntax Highlighting Solution

**Recommended:** **Shiki** (over Prism)

**Rationale:**
- **Shiki advantages:**
  - Server-side rendering compatible (crucial for Next.js)
  - Uses real VS Code themes and grammars
  - Better TypeScript support
  - Smaller runtime bundle (highlighting done at build time)
  - More accurate syntax highlighting
  - No client-side JavaScript required

- **Implementation:**
  ```bash
  pnpm add shiki
  pnpm add @types/hast
  ```

**Configuration:**
- **Themes:** Use `github-dark` to match the current dark theme
- **Languages:** Support 50+ common languages (JS, TS, Python, Rust, Go, etc.)
- **Fallback:** Plain text for unsupported languages

### 2. Component Architecture Changes

#### 2.1 New Component Structure

```
src/components/markdown/
├── MarkdownRenderer.tsx          # Main renderer component
├── CodeBlock.tsx                 # Enhanced code block with copy button
├── TableRenderer.tsx             # GitHub-style table component
├── HeadingRenderer.tsx           # Headings with anchor links
├── ImageRenderer.tsx             # Enhanced image handling
├── LinkRenderer.tsx              # External link styling
└── styles/
    ├── markdown.css              # Markdown-specific styles
    └── syntax-highlighting.css   # Code highlighting themes
```

#### 2.2 MarkdownRenderer Component

```typescript
interface MarkdownRendererProps {
  content: string;
  baseUrl?: string; // For relative image/link resolution
  className?: string;
}

interface CodeHighlightingOptions {
  theme: 'github-dark' | 'github-light';
  showLineNumbers: boolean;
  showLanguage: boolean;
  enableCopy: boolean;
}
```

### 3. Styling Requirements

#### 3.1 Design System Compliance

**Color Scheme Integration:**
- Use existing CSS variables from `globals.css`
- Maintain consistency with repo.box design language
- Dark theme optimization

**Typography:**
- Headings: Use `--bp-heading` color
- Body text: Use `--bp-text` color  
- Code: Use `--font-mono` family
- Links: Use `--bp-accent` color

#### 3.2 GitHub-Style Table Rendering

**Features:**
- Zebra striping for rows
- Proper column alignment (left, center, right)
- Responsive horizontal scrolling
- Consistent padding and borders

**CSS Requirements:**
```css
.markdown-table {
  border-collapse: collapse;
  margin: 1.5rem 0;
  width: 100%;
  overflow-x: auto;
}

.markdown-table th {
  background: var(--bp-surface);
  border: 1px solid var(--bp-border);
  padding: 8px 16px;
  text-align: left;
  font-weight: 600;
}

.markdown-table td {
  border: 1px solid var(--bp-border);
  padding: 8px 16px;
}

.markdown-table tbody tr:nth-child(even) {
  background: rgba(79, 195, 247, 0.03);
}
```

#### 3.3 Code Block Enhancements

**Required Features:**
1. **Language indicator badge** (top-right corner)
2. **Copy to clipboard button**
3. **Line numbers** (optional, configurable)
4. **Syntax highlighting** with proper theme
5. **Horizontal scrolling** for long lines

### 4. Anchor Links for Headings

**Implementation Approach:**
- Generate slug-based IDs from heading text
- Add clickable anchor icons (🔗) on hover
- Smooth scroll behavior for navigation
- Update browser URL with fragment

**Slug Generation:**
```typescript
function generateHeadingId(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .trim();
}
```

### 5. Enhanced Image Support

#### 5.1 Image Features
1. **Responsive sizing** - Fit container width
2. **Click to zoom** - Modal overlay for large images  
3. **Loading states** - Skeleton while loading
4. **Error fallbacks** - Broken image handling
5. **Alt text display** - Accessibility compliance

#### 5.2 Image Resolution Strategy
- **Relative paths:** Resolve against repository root
- **Absolute URLs:** Direct rendering
- **GitHub blob URLs:** Proxy through repo.box API if needed

### 6. Advanced Markdown Features

#### 6.1 GitHub-Flavored Markdown Support
- [x] **Task lists** with interactive checkboxes (read-only)
- **Strikethrough** text
- **Tables** with alignment
- **Footnotes** support
- **Math expressions** (LaTeX) - future enhancement

#### 6.2 Callouts and Admonitions
Support GitHub-style alerts:
```markdown
> [!NOTE]
> This is a note

> [!WARNING]  
> This is a warning
```

## Implementation Details

### 7. Component Integration

#### 7.1 ReactMarkdown Configuration

```typescript
const markdownComponents = {
  code: ({ className, children, inline, ...props }) => {
    if (inline) {
      return <InlineCode>{children}</InlineCode>;
    }
    const language = className?.replace('language-', '') || 'text';
    return (
      <CodeBlock 
        language={language} 
        code={String(children)} 
        {...props} 
      />
    );
  },
  table: ({ children }) => (
    <TableRenderer>{children}</TableRenderer>
  ),
  h1: ({ children }) => (
    <HeadingRenderer level={1}>{children}</HeadingRenderer>
  ),
  h2: ({ children }) => (
    <HeadingRenderer level={2}>{children}</HeadingRenderer>
  ),
  // ... other heading levels
  img: ({ src, alt }) => (
    <ImageRenderer src={src} alt={alt} />
  ),
  a: ({ href, children }) => (
    <LinkRenderer href={href}>{children}</LinkRenderer>
  ),
};
```

#### 7.2 Integration into Explorer Page

**Changes to `/web/src/app/explore/[address]/[name]/page.tsx`:**

1. Replace current ReactMarkdown implementation
2. Import new MarkdownRenderer component
3. Update CSS class names to match new structure
4. Add proper error boundaries

```typescript
// Replace current README rendering section
{activeTab === 'readme' && (
  <div className="explore-readme">
    {repo.readme_content ? (
      <MarkdownRenderer 
        content={repo.readme_content}
        baseUrl={`/api/explorer/repos/${address}/${name}/blob/`}
        className="explore-readme-content"
      />
    ) : (
      <div className="explore-empty">
        <p>No README found</p>
      </div>
    )}
  </div>
)}
```

### 8. Performance Considerations

#### 8.1 Bundle Size Optimization
- **Shiki:** Load only required languages and themes
- **Tree shaking:** Ensure unused markdown components are eliminated
- **Code splitting:** Lazy load heavy components

#### 8.2 Caching Strategy
- **Syntax highlighting:** Cache highlighted code blocks
- **Image processing:** Implement proper caching headers
- **Markdown parsing:** Memoize expensive operations

## Edge Cases & Error Handling

### 9. Content Security

#### 9.1 XSS Prevention
- **HTML sanitization:** Strip dangerous HTML elements
- **Link validation:** Restrict javascript: and data: URLs
- **Image sources:** Validate and proxy external images

#### 9.2 Content Limits
- **File size:** Limit README rendering to reasonable size (1MB max)
- **Nesting depth:** Prevent deeply nested markdown from breaking layout
- **Table size:** Handle very wide tables with horizontal scrolling

### 10. Accessibility Requirements

#### 10.1 WCAG 2.1 AA Compliance
- **Color contrast:** Ensure 4.5:1 minimum ratio
- **Keyboard navigation:** All interactive elements accessible via keyboard
- **Screen reader support:** Proper heading structure and alt text
- **Focus indicators:** Visible focus states for all interactive elements

#### 10.2 Semantic HTML
- Use proper heading hierarchy (h1 → h2 → h3)
- Table headers properly associated with data
- Image alt text requirements
- Link descriptions for screen readers

### 11. Browser Compatibility

**Target Support:**
- Chrome 90+
- Firefox 88+  
- Safari 14+
- Edge 90+

**Progressive Enhancement:**
- Syntax highlighting gracefully degrades to unstyled code
- Copy buttons fail silently if clipboard API unavailable
- Image zoom falls back to new tab opening

## Testing Strategy

### 12. Test Coverage Areas

#### 12.1 Unit Tests
- **Component rendering:** Each markdown component renders correctly
- **Slug generation:** Heading ID generation is deterministic
- **Code highlighting:** Shiki integration works properly
- **Table parsing:** Complex tables render correctly

#### 12.2 Integration Tests
- **README rendering:** Full README files render without errors
- **Image handling:** Relative and absolute image paths work
- **Link processing:** Internal and external links behave correctly
- **Responsive behavior:** Tables and images adapt to screen sizes

#### 12.3 Visual Regression Tests
- **Before/after comparisons:** Ensure visual consistency
- **Cross-browser testing:** Verify rendering across browsers
- **Mobile responsiveness:** Test on various screen sizes

#### 12.4 Performance Tests
- **Bundle size:** Ensure new dependencies don't bloat bundle significantly
- **Rendering speed:** Large README files render within acceptable time
- **Memory usage:** No memory leaks from syntax highlighting

### 13. Test Cases Matrix

| Feature | Input | Expected Output | Edge Case |
|---------|--------|----------------|-----------|
| Syntax Highlighting | ```js\nconsole.log('hi')``` | Highlighted JavaScript code | Unsupported language → plain text |
| Tables | GitHub markdown table | Styled table with borders | Very wide table → horizontal scroll |
| Headings | # Heading with spaces | Anchor link with slug ID | Special characters → sanitized slug |
| Images | ![alt](image.png) | Responsive image with zoom | Broken image → fallback placeholder |
| Links | [text](url) | Styled external link | Invalid URL → disabled link |

## Implementation Timeline

### 14. Development Phases

#### Phase 1: Foundation (Week 1)
- Set up Shiki integration
- Create basic MarkdownRenderer component
- Implement CodeBlock with syntax highlighting
- Basic styling framework

#### Phase 2: Core Features (Week 2)  
- Enhanced table rendering
- Heading anchor links
- Image handling improvements
- Link processing upgrades

#### Phase 3: Polish & Testing (Week 3)
- Accessibility improvements
- Performance optimizations  
- Comprehensive testing
- Cross-browser validation

#### Phase 4: Integration & Launch (Week 4)
- Integration into explorer page
- User acceptance testing
- Production deployment
- Monitoring and metrics

## Success Metrics

### 15. Acceptance Criteria

#### Functional Requirements
- [ ] Code blocks display syntax highlighting for 20+ languages
- [ ] Tables render with GitHub-style appearance
- [ ] All headings have clickable anchor links
- [ ] Images are responsive and zoomable
- [ ] External links open in new tabs with proper security
- [ ] Copy-to-clipboard works on code blocks

#### Non-Functional Requirements  
- [ ] Page load time increase < 10% from baseline
- [ ] Bundle size increase < 50KB gzipped
- [ ] WCAG 2.1 AA accessibility compliance
- [ ] Works on mobile devices (320px+ width)
- [ ] No JavaScript errors in production

#### User Experience
- [ ] README files look visually similar to GitHub
- [ ] Navigation via anchor links feels smooth
- [ ] Code is easy to read and copy
- [ ] Tables are scannable and well-formatted
- [ ] Images enhance rather than distract from content

## Migration Strategy

### 16. Rollout Plan

#### 16.1 Feature Flag Implementation
- Implement feature toggle for new markdown renderer
- A/B test with subset of users
- Gradual rollout based on performance metrics

#### 16.2 Backward Compatibility
- Maintain current ReactMarkdown as fallback
- Graceful degradation for unsupported content
- Clear migration path for custom extensions

#### 16.3 Monitoring & Alerting
- Track rendering performance metrics
- Monitor JavaScript error rates
- User feedback collection mechanism

## Future Enhancements

### 17. Post-Launch Improvements

#### 17.1 Advanced Features
- **Math rendering:** LaTeX equation support via KaTeX
- **Mermaid diagrams:** Flowchart and diagram rendering
- **Interactive elements:** Live code playgrounds for certain languages
- **Table of contents:** Auto-generated navigation for long READMEs

#### 17.2 Performance Optimizations
- **Web Workers:** Move syntax highlighting to background thread
- **Virtual scrolling:** For very long documents
- **Progressive loading:** Render above-fold content first

---

## Appendix

### A. Dependencies

**New Dependencies:**
```json
{
  "shiki": "^1.0.0",
  "rehype-slug": "^6.0.0", 
  "rehype-autolink-headings": "^7.0.0",
  "rehype-external-links": "^3.0.0"
}
```

### B. File Structure Changes

```
web/src/
├── components/
│   └── markdown/           # New directory
│       ├── MarkdownRenderer.tsx
│       ├── CodeBlock.tsx
│       ├── TableRenderer.tsx  
│       ├── HeadingRenderer.tsx
│       ├── ImageRenderer.tsx
│       ├── LinkRenderer.tsx
│       └── styles/
│           ├── markdown.css
│           └── syntax-themes.css
├── app/explore/[address]/[name]/
│   └── page.tsx           # Modified
└── styles/
    └── globals.css        # Updated with markdown styles
```

### C. CSS Class Naming Convention

**Prefix:** `md-` (markdown) to distinguish from explore-specific styles

**Examples:**
- `.md-container` - Main markdown wrapper
- `.md-code-block` - Code block container
- `.md-table` - Table styling
- `.md-heading-anchor` - Heading anchor links
- `.md-image-zoom` - Image zoom overlay

---

**This specification serves as the complete technical blueprint for implementing GitHub-style README rendering in repo.box. All implementation details, edge cases, and success criteria are defined to ensure a high-quality, accessible, and performant upgrade to the current markdown rendering system.**