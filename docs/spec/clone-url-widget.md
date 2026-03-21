# Clone URL Copy Widget + Credential Helper Instructions Specification

**Version:** 1.0  
**Date:** March 21, 2026  
**Author:** PM Agent (`0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b`)  
**Status:** Draft  

## 1. Executive Summary

This specification defines the implementation of a prominent clone URL copy widget with credential helper setup instructions on repository detail pages. The feature provides one-click copying of HTTPS and SSH clone URLs, plus an expandable section with step-by-step instructions for configuring the repobox credential helper for authenticated clones.

## 2. User Stories

### 2.1 Primary User Stories
- **US-001**: As a developer, I want to quickly copy the clone URL with one click so that I can efficiently clone repositories
- **US-002**: As a developer, I want access to both HTTPS and SSH clone variants so that I can choose my preferred authentication method  
- **US-003**: As a developer, I want clear instructions for setting up the repobox credential helper so that I can clone private repos without manual authentication
- **US-004**: As a developer, I want a one-line install command so that I can quickly set up repobox on my machine
- **US-005**: As a developer, I want the widget to be visually prominent on the repo page so that cloning is the obvious next action

### 2.2 Secondary User Stories  
- **US-006**: As a mobile user, I want the clone URLs to remain accessible and readable on small screens
- **US-007**: As a developer, I want visual feedback when copying URLs so that I know the action succeeded
- **US-008**: As a developer, I want to understand the difference between regular git and authenticated repobox cloning

## 3. Current State Analysis

### 3.1 Existing Implementation
The current repo detail page at `/home/xiko/repobox/web/src/app/explore/[address]/[name]/page.tsx` includes:

```tsx
<div className="explore-repo-detail-clone">
  <div className="explore-clone-label">HTTPS</div>
  <div className="explore-clone-input-group">
    <input 
      type="text" 
      value={cloneUrl}
      readOnly
      className="explore-clone-input"
    />
    <button 
      onClick={handleCopyClone} 
      className="explore-clone-copy-btn"
      title="Copy clone command"
    >
      {/* Copy icon with state */}
    </button>
  </div>
</div>
```

### 3.2 Current Styling
The CSS classes in `globals.css` provide:
- `.explore-clone-label`: 12px font, uppercase, letter-spacing
- `.explore-clone-input-group`: Flexbox container with border 
- `.explore-clone-input`: Monospace font, flex-1, background/border
- `.explore-clone-copy-btn`: Icon button with hover states

### 3.3 Gaps in Current Implementation
1. **No SSH variant**: Only HTTPS clone URL is shown
2. **No credential helper instructions**: No guidance for authenticated cloning
3. **Not prominent enough**: Clone section is small compared to other elements
4. **No install instructions**: No curl|sh snippet provided
5. **Limited educational content**: Users don't understand EVM-authenticated git workflow

## 4. Feature Design

### 4.1 Widget Layout

The new clone widget will be prominently placed after the repository header and before the stats cards:

```
┌─ Repository Header ─────────────────────────┐
│ repo-name                    [Star] [Fork]  │
│ 0xOwner...Address           [Copy Address]  │
└─────────────────────────────────────────────┘
┌─ CLONE WIDGET ──────────────────────────────┐  ← NEW
│ Clone Repository                            │
│ ┌─ HTTPS ────────────────────────┐ [Copy]   │
│ │ https://git.repo.box/...       │         │
│ └────────────────────────────────┘         │
│ ┌─ SSH ──────────────────────────┐ [Copy]   │  ← NEW
│ │ git@git.repo.box:...           │         │
│ └────────────────────────────────┘         │
│ ▼ Need authenticated access? Set up repobox │  ← NEW
│   [COLLAPSED SECTION]                      │
└─────────────────────────────────────────────┘
┌─ Repository Stats Cards ────────────────────┐
│ Languages | Contributors | Lines of Code   │
└─────────────────────────────────────────────┘
```

### 4.2 Expandable Instructions Section

When clicked, the expandable section reveals:

```
▲ Need authenticated access? Set up repobox
┌─ Installation & Setup ─────────────────────┐
│ 1. Install repobox CLI                    │
│ ┌───────────────────────────────────────┐   │
│ │ curl -sSf repo.box/install.sh | sh    │ [Copy]
│ └───────────────────────────────────────┘   │
│                                            │
│ 2. Generate EVM identity                   │
│ $ repobox keys generate                    │
│                                            │
│ 3. Configure git credential helper        │  ← NEW
│ $ git config --global credential.helper \ │
│     "!repobox credential-helper"           │
│                                            │
│ 4. Clone with automatic authentication    │
│ $ git clone [URL from above]              │
│                                            │
│ ℹ  Your EVM identity will be used for     │
│    push authentication. No passwords      │
│    required - everything is cryptographic │
│    signatures.                            │
└─────────────────────────────────────────────┘
```

### 4.3 Visual Hierarchy

1. **High prominence**: Clone widget sits prominently after repo header
2. **Clear CTAs**: Both HTTPS and SSH have large, obvious copy buttons  
3. **Progressive disclosure**: Setup instructions hidden by default, expandable
4. **Scan-friendly**: Numbered steps, code blocks, copy buttons on snippets

### 4.4 Mobile Considerations

On mobile (< 768px):
- Clone URLs switch to vertical stacking (HTTPS above SSH)
- Code snippets remain horizontally scrollable
- Copy buttons maintain 44px minimum touch target
- Expandable section uses full width

## 5. Technical Implementation

### 5.1 Component Structure

#### 5.1.1 New CloneUrlWidget Component

```tsx
// /components/CloneUrlWidget.tsx
interface CloneUrlWidgetProps {
  ownerAddress: string;
  repoName: string;
  className?: string;
}

export default function CloneUrlWidget({ 
  ownerAddress, 
  repoName, 
  className 
}: CloneUrlWidgetProps) {
  const [httpsUrl, sshUrl] = generateCloneUrls(ownerAddress, repoName);
  const [expandedHelp, setExpandedHelp] = useState(false);
  const [copiedItem, setCopiedItem] = useState<string | null>(null);

  const handleCopy = async (text: string, itemId: string) => {
    await copyToClipboard(text);
    setCopiedItem(itemId);
    setTimeout(() => setCopiedItem(null), 2000);
  };

  return (
    <div className={`clone-url-widget ${className}`}>
      {/* Implementation details below */}
    </div>
  );
}
```

#### 5.1.2 Helper Functions

```tsx
function generateCloneUrls(ownerAddress: string, repoName: string) {
  const httpsUrl = `https://git.repo.box/${ownerAddress}/${repoName}.git`;
  const sshUrl = `git@git.repo.box:${ownerAddress}/${repoName}.git`;
  return [httpsUrl, sshUrl];
}

async function copyToClipboard(text: string): Promise<void> {
  if (navigator.clipboard) {
    await navigator.clipboard.writeText(text);
  } else {
    // Fallback for older browsers
    const textarea = document.createElement('textarea');
    textarea.value = text;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
  }
}
```

### 5.2 CSS Styling

#### 5.2.1 Widget Container

```css
/* Clone URL Widget */
.clone-url-widget {
  background: var(--explore-surface);
  border: 1px solid var(--explore-border);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 24px;
}

.clone-url-widget-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--explore-heading);
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.clone-url-widget-title svg {
  width: 20px;
  height: 20px;
  color: var(--explore-accent);
}
```

#### 5.2.2 URL Input Groups

```css
.clone-url-group {
  margin-bottom: 12px;
}

.clone-url-group:last-of-type {
  margin-bottom: 20px;
}

.clone-url-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--explore-text);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.clone-url-input-container {
  display: flex;
  border: 1px solid var(--explore-border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--explore-bg);
}

.clone-url-input {
  flex: 1;
  background: transparent;
  border: none;
  padding: 10px 12px;
  color: var(--explore-text);
  font-family: var(--font-mono), monospace;
  font-size: 14px;
  min-width: 0;
}

.clone-url-input:focus {
  outline: none;
}

.clone-url-copy-btn {
  background: var(--explore-surface-hover);
  border: none;
  border-left: 1px solid var(--explore-border);
  color: var(--explore-text-secondary);
  padding: 10px 14px;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 44px; /* Touch-friendly */
}

.clone-url-copy-btn:hover {
  background: var(--explore-border);
  color: var(--explore-text);
}

.clone-url-copy-btn.copied {
  color: var(--explore-accent);
  background: rgba(79, 195, 247, 0.1);
}
```

#### 5.2.3 Expandable Help Section

```css
.clone-help-toggle {
  background: none;
  border: none;
  color: var(--explore-text-secondary);
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  transition: color 0.15s ease;
  width: 100%;
  text-align: left;
}

.clone-help-toggle:hover {
  color: var(--explore-text);
}

.clone-help-chevron {
  transition: transform 0.2s ease;
}

.clone-help-toggle[aria-expanded="true"] .clone-help-chevron {
  transform: rotate(180deg);
}

.clone-help-content {
  border-top: 1px solid var(--explore-border);
  padding-top: 20px;
  margin-top: 12px;
}

.clone-help-step {
  margin-bottom: 20px;
}

.clone-help-step:last-child {
  margin-bottom: 0;
}

.clone-help-step-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--explore-heading);
  margin-bottom: 8px;
}

.clone-help-code {
  background: var(--explore-bg);
  border: 1px solid var(--explore-border);
  border-radius: 4px;
  padding: 12px;
  font-family: var(--font-mono), monospace;
  font-size: 13px;
  color: var(--explore-text);
  position: relative;
  margin-bottom: 8px;
  overflow-x: auto;
  white-space: pre;
}

.clone-help-code-copy {
  position: absolute;
  top: 8px;
  right: 8px;
  background: var(--explore-surface-hover);
  border: 1px solid var(--explore-border);
  border-radius: 3px;
  padding: 4px 6px;
  color: var(--explore-text-secondary);
  cursor: pointer;
  font-size: 11px;
  transition: all 0.15s ease;
}

.clone-help-code-copy:hover {
  background: var(--explore-border);
  color: var(--explore-text);
}

.clone-help-info {
  background: rgba(79, 195, 247, 0.1);
  border: 1px solid rgba(79, 195, 247, 0.3);
  border-radius: 6px;
  padding: 12px;
  color: var(--explore-text);
  font-size: 13px;
  line-height: 1.5;
}

.clone-help-info::before {
  content: "ℹ ";
  color: var(--explore-accent);
  font-weight: 600;
}
```

#### 5.2.4 Mobile Responsive

```css
@media (max-width: 767px) {
  .clone-url-widget {
    padding: 16px;
    margin-bottom: 20px;
  }

  .clone-url-widget-title {
    font-size: 15px;
    margin-bottom: 14px;
  }

  .clone-url-input-container {
    flex-direction: column;
  }

  .clone-url-input {
    border-radius: 6px 6px 0 0;
    border-bottom: none;
  }

  .clone-url-copy-btn {
    border-left: none;
    border-top: 1px solid var(--explore-border);
    border-radius: 0 0 6px 6px;
    padding: 12px;
    justify-content: center;
  }

  .clone-help-code {
    padding: 10px;
    font-size: 12px;
  }

  .clone-help-code-copy {
    position: static;
    margin-top: 8px;
    display: inline-block;
  }
}
```

### 5.3 Integration Points

#### 5.3.1 Repository Detail Page Update

```tsx
// In /app/explore/[address]/[name]/page.tsx

import CloneUrlWidget from '@/components/CloneUrlWidget';

// Add after repository header, before RepoStatsCards
<div className="explore-main-content">
  {/* Repository Header */}
  <div className="explore-repo-detail-header">
    {/* Existing header content */}
  </div>

  {/* NEW: Clone URL Widget */}
  <CloneUrlWidget 
    ownerAddress={repo.owner_address}
    repoName={repo.name}
  />

  {/* Repository Stats */}
  <RepoStatsCards 
    address={address} 
    name={name} 
    branch={selectedBranch}
  />

  {/* Rest of existing content */}
</div>
```

### 5.4 State Management

The component manages these states:
- `expandedHelp: boolean` - Whether instructions section is expanded
- `copiedItem: string | null` - Which item was recently copied (for visual feedback)

State is local to the component with no persistence needed.

## 6. API Requirements

### 6.1 No New API Endpoints Required

This feature is entirely client-side:
- Clone URLs are generated from existing `repo.owner_address` and `repo.name`
- Install snippet is static content
- Credential helper setup instructions are static

### 6.2 URL Generation Logic

```typescript
interface CloneUrls {
  https: string;
  ssh: string;
}

function generateCloneUrls(ownerAddress: string, repoName: string): CloneUrls {
  return {
    https: `https://git.repo.box/${ownerAddress}/${repoName}.git`,
    ssh: `git@git.repo.box:${ownerAddress}/${repoName}.git`
  };
}
```

The SSH URL format follows the standard `git@hostname:path` convention used by GitHub and other git hosting services.

## 7. Content Specifications

### 7.1 Install Command

The install command uses the existing install script:

```bash
curl -sSf https://repo.box/install.sh | sh
```

This script:
- Detects platform (Linux/macOS, x86_64/ARM64)
- Downloads appropriate binary from `/releases/{version}/`
- Verifies SHA256 checksum
- Installs to `~/.repobox/bin/repobox`
- Sets up PATH modification

### 7.2 Credential Helper Setup Commands

```bash
# Global credential helper setup
git config --global credential.helper "!repobox credential-helper"

# Alternative: repo-specific setup
git config credential.helper "!repobox credential-helper"
```

### 7.3 Workflow Steps

1. **Install CLI**: `curl -sSf repo.box/install.sh | sh`
2. **Generate identity**: `repobox keys generate`  
3. **Setup credential helper**: `git config --global credential.helper "!repobox credential-helper"`
4. **Clone**: `git clone [URL]`

### 7.4 Educational Copy

**Help section description**:
> "Your EVM identity will be used for push authentication. No passwords required - everything is cryptographic signatures."

**Key benefits**:
- No password management
- Cryptographic authentication  
- Works with existing git workflow
- One-time setup per machine

## 8. Acceptance Criteria

### 8.1 Functional Requirements

- [ ] **AC-001**: Clone widget displays prominently after repo header and before stats
- [ ] **AC-002**: Both HTTPS and SSH clone URLs are shown with copy buttons  
- [ ] **AC-003**: Copy buttons provide visual feedback (icon change + color) when clicked
- [ ] **AC-004**: Expandable help section is collapsed by default
- [ ] **AC-005**: Help section includes 4 numbered steps with code snippets
- [ ] **AC-006**: Each code snippet has a copy button
- [ ] **AC-007**: Install command uses the correct `curl -sSf repo.box/install.sh | sh` format
- [ ] **AC-008**: Credential helper setup includes the exact git config command
- [ ] **AC-009**: Educational info box explains EVM authentication benefits

### 8.2 UI/UX Requirements  

- [ ] **AC-010**: Widget has clear visual hierarchy with proper spacing
- [ ] **AC-011**: Copy buttons meet 44px minimum touch target on mobile
- [ ] **AC-012**: Code snippets are horizontally scrollable on mobile  
- [ ] **AC-013**: Expandable section uses smooth accordion animation
- [ ] **AC-014**: Visual feedback for copy operations lasts 2 seconds
- [ ] **AC-015**: Widget styling matches existing explore page design system

### 8.3 Technical Requirements

- [ ] **AC-016**: Component is reusable and accepts `ownerAddress` and `repoName` props
- [ ] **AC-017**: No new API endpoints required - uses client-side URL generation
- [ ] **AC-018**: All copy operations use clipboard API with fallback for older browsers
- [ ] **AC-019**: Component handles errors gracefully (clipboard access denied, etc.)
- [ ] **AC-020**: Mobile responsive design works correctly at 375px width

### 8.4 Content Requirements

- [ ] **AC-021**: Install command matches exact format from existing install script
- [ ] **AC-022**: Credential helper git config command uses correct syntax  
- [ ] **AC-023**: SSH URL format follows `git@hostname:path.git` convention
- [ ] **AC-024**: Educational copy explains EVM authentication accurately
- [ ] **AC-025**: Step numbering and descriptions are clear and actionable

## 9. Implementation Plan

### 9.1 Phase 1: Core Widget (4 hours)

1. **Create CloneUrlWidget component** (2 hours)
   - Component structure and props
   - URL generation logic
   - Copy functionality with visual feedback
   - Basic styling

2. **Integrate into repository page** (1 hour)
   - Add import and component placement
   - Test with different repository URLs

3. **Mobile responsive styling** (1 hour)
   - Test on various screen sizes
   - Ensure touch targets meet accessibility guidelines

### 9.2 Phase 2: Help Section (3 hours)

1. **Expandable section implementation** (2 hours)
   - Accordion animation
   - Content structure with steps
   - Code blocks with copy buttons

2. **Content and styling refinement** (1 hour)
   - Educational copy review
   - Visual polish and spacing
   - Cross-browser testing

### 9.3 Phase 3: Polish & Testing (2 hours)

1. **Integration testing** (1 hour)
   - Test with various repository types
   - Error handling scenarios
   - Accessibility review

2. **Performance and UX review** (1 hour)
   - Animation smoothness
   - Copy feedback timing
   - Content clarity review

**Total estimated effort: 9 hours**

## 10. Future Considerations

### 10.1 Potential Enhancements

- **SSH key integration**: Help users set up SSH keys for git.repo.box
- **Clone command variants**: Support for shallow clones, specific branches
- **Credential helper status**: Show whether credential helper is already configured
- **Identity verification**: Show current EVM identity if repobox is installed
- **Video tutorials**: Embed demo videos showing the setup process

### 10.2 Analytics & Metrics

Potential metrics to track:
- Clone URL copy rates (HTTPS vs SSH)
- Help section expansion rates  
- Install command copy rates
- Time spent in help section

### 10.3 A11y Considerations

- Ensure screen readers can navigate expanded/collapsed states
- Proper focus management when expanding sections
- Alt text for copy icons and visual feedback
- High contrast testing for copy state indicators

---

## Appendix A: Wire frames

### A.1 Desktop Layout
```
┌────────────────────────────────────────────────────────────┐
│ Repository Header                                          │
├────────────────────────────────────────────────────────────┤
│ 📥 Clone Repository                                        │
│                                                           │
│ HTTPS  ┌─ https://git.repo.box/0xAddr.../repo.git ─┐ [📋] │
│        └───────────────────────────────────────────┘     │
│                                                           │
│ SSH    ┌─ git@git.repo.box:0xAddr.../repo.git ────┐ [📋] │
│        └───────────────────────────────────────────┘     │
│                                                           │
│ ▼ Need authenticated access? Set up repobox               │
├────────────────────────────────────────────────────────────┤
│ Repository Stats Cards                                     │
└────────────────────────────────────────────────────────────┘
```

### A.2 Mobile Layout  
```
┌─────────────────────────────┐
│ Repository Header           │
├─────────────────────────────┤
│ 📥 Clone Repository         │
│                             │
│ HTTPS                       │
│ ┌─ https://git.repo.box...─┐ │
│ └─────────────────────────┘ │
│ ┌─ Copy ─────────────────────┤
│ └─────────────────────────┘ │
│                             │
│ SSH                         │
│ ┌─ git@git.repo.box:...─────┐ │
│ └─────────────────────────┘ │
│ ┌─ Copy ─────────────────────┤
│ └─────────────────────────┘ │
│                             │
│ ▼ Need authenticated        │
│   access? Set up repobox    │
├─────────────────────────────┤
│ Repository Stats Cards      │
└─────────────────────────────┘
```

### A.3 Expanded Help Section
```
┌────────────────────────────────────────────────────────────┐
│ ▲ Need authenticated access? Set up repobox               │
│                                                           │
│ 1. Install repobox CLI                                    │
│    ┌─ curl -sSf repo.box/install.sh | sh ───────┐ [📋]    │
│    └───────────────────────────────────────────┘        │
│                                                           │
│ 2. Generate EVM identity                                  │
│    $ repobox keys generate                                │
│                                                           │
│ 3. Configure git credential helper                       │
│    $ git config --global credential.helper \             │
│        "!repobox credential-helper"                       │
│                                                           │
│ 4. Clone with automatic authentication                   │
│    $ git clone [URL from above]                          │
│                                                           │
│ ℹ  Your EVM identity will be used for push               │
│    authentication. No passwords required - everything     │
│    is cryptographic signatures.                          │
└────────────────────────────────────────────────────────────┘
```