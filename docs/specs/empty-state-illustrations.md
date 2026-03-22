# Empty State Illustrations Specification

**Priority:** P2  
**Tags:** explorer, ui  
**Created:** 2026-03-22  
**Status:** Specification Complete

## Overview

This specification defines the replacement of generic "No X found" messages throughout the repo.box explorer with custom SVG illustrations and helpful contextual guidance. The goal is to create a cohesive, branded experience that guides users toward productive actions when encountering empty states.

## Current State Analysis

After thorough code review, the following empty states currently exist in the codebase:

### 1. Main Explorer Page (`/explore`)
- **Location:** `/web/src/app/explore/page.tsx`
- **Current State:** "📦 No repositories yet" / "No matching repositories"
- **Context:** When no repositories exist or search returns no results

### 2. Recent Activity Sidebar
- **Location:** `/web/src/components/explore/ExploreSidebar.tsx`
- **Current State:** "No recent activity"
- **Context:** When no recent commit activity exists

### 3. Address Repository List (`/explore/[address]`)
- **Location:** `/web/src/app/explore/[address]/page.tsx`
- **Current State:** Generic repository icon with "No repositories found"
- **Context:** When a developer has no published repositories

### 4. Repository Commits Tab
- **Location:** `/web/src/app/explore/[address]/[name]/page.tsx`
- **Current State:** "No commits found"
- **Context:** When a repository branch has no commits

### 5. Repository Contributors Tab
- **Location:** `/web/src/app/explore/[address]/[name]/page.tsx`
- **Current State:** "No contributors found"
- **Context:** When repository has no tracked contributors

### 6. Commit List Page
- **Location:** `/web/src/app/explore/[address]/[name]/commits/[branch]/page.tsx`
- **Current State:** "No commits found"
- **Context:** When specific branch has no commits

### 7. Contribution Chart
- **Location:** `/web/src/components/explore/ContributionChart.tsx`
- **Current State:** "No contribution activity found for the selected time range"
- **Context:** When contributor has no activity in selected timeframe

## Brand Identity & Design System

### Color Palette (from globals.css)
```css
/* Primary brand colors */
--bp-accent: #4fc3f7     /* Primary blue */
--bp-accent2: #81d4fa    /* Secondary blue */
--bp-text: #b8d4e3       /* Primary text */
--bp-heading: #e8f4fd    /* Heading text */
--bp-dim: #7a9ab4        /* Muted text */

/* Surface colors */
--bp-bg: #0a1628         /* Background */
--bp-surface: #0d1f35    /* Surface/card background */
--bp-border: rgba(50, 100, 160, 0.25)  /* Border color */
```

### Typography
- **Font Family:** JetBrains Mono (monospace)
- **Style:** Clean, developer-focused, minimal
- **Tone:** Professional yet approachable

### Visual Identity
- **Primary Icon:** Jellyfish/sea life theme (🪼 favicon)
- **Style:** Ocean/water metaphors, fluid animations
- **Approach:** Simple, clean SVG illustrations

## Illustration Specifications

### Design Principles
1. **Consistent Style:** All illustrations use the same visual language
2. **Brand Coherent:** Incorporate ocean/sea life theme where appropriate
3. **Scalable:** Work at multiple sizes (mobile/desktop)
4. **Accessible:** High contrast, meaningful without color
5. **Performance:** Inline SVG, optimized paths

### Technical Requirements
- **Format:** Inline SVG (not external files)
- **Viewbox:** 120x120 standard size
- **Colors:** Use CSS custom properties for theming
- **Animation:** Subtle CSS animations where appropriate
- **Responsive:** Scale appropriately on mobile

### Illustration Set

#### 1. Empty Repository Explorer
**Context:** No repositories found on main explore page  
**Illustration:** Floating jellyfish with empty treasure chest  
**Message:** "No repositories in the ocean yet"  
**Guidance:** "Push your first signed commit to get started"

```svg
<svg viewBox="0 0 120 120" className="empty-state-illustration">
  <defs>
    <linearGradient id="jellyfish-gradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stopColor="var(--bp-accent2)" opacity="0.8"/>
      <stop offset="100%" stopColor="var(--bp-accent)" opacity="0.4"/>
    </linearGradient>
  </defs>
  
  <!-- Jellyfish body -->
  <ellipse cx="60" cy="35" rx="25" ry="15" fill="url(#jellyfish-gradient)" className="float-animation"/>
  
  <!-- Jellyfish tentacles -->
  <path d="M40 45 Q35 65 40 75 Q45 65 50 75 Q55 65 60 75 Q65 65 70 75 Q75 65 80 75" 
        stroke="var(--bp-accent)" strokeWidth="2" fill="none" opacity="0.6"/>
  
  <!-- Empty treasure chest -->
  <rect x="45" y="70" width="30" height="20" rx="3" fill="var(--bp-surface)" 
        stroke="var(--bp-border)" strokeWidth="2"/>
  <path d="M45 75 L75 75" stroke="var(--bp-border)" strokeWidth="1"/>
  
  <!-- Floating bubbles -->
  <circle cx="25" cy="40" r="2" fill="var(--bp-accent2)" opacity="0.5" className="bubble-float"/>
  <circle cx="90" cy="30" r="1.5" fill="var(--bp-accent2)" opacity="0.4" className="bubble-float" style="animation-delay: 0.5s"/>
  <circle cx="85" cy="55" r="1" fill="var(--bp-accent2)" opacity="0.3" className="bubble-float" style="animation-delay: 1s"/>
</svg>
```

#### 2. No Recent Activity
**Context:** Sidebar shows no recent activity  
**Illustration:** Sleeping jellyfish with "Zzz" bubbles  
**Message:** "The ocean is calm"  
**Guidance:** "Recent commits will appear here"

```svg
<svg viewBox="0 0 80 60" className="empty-state-illustration-small">
  <!-- Sleeping jellyfish -->
  <ellipse cx="40" cy="25" rx="18" ry="12" fill="url(#jellyfish-gradient)" opacity="0.6"/>
  <path d="M25 32 Q22 45 25 50 Q30 45 35 50 Q40 45 45 50 Q50 45 55 50" 
        stroke="var(--bp-dim)" strokeWidth="1.5" fill="none" opacity="0.4"/>
  
  <!-- Closed eyes -->
  <ellipse cx="35" cy="22" rx="2" ry="1" fill="var(--bp-dim)"/>
  <ellipse cx="45" cy="22" rx="2" ry="1" fill="var(--bp-dim)"/>
  
  <!-- Sleep bubbles (Zzz) -->
  <text x="60" y="20" fill="var(--bp-dim)" fontSize="8" opacity="0.5">z</text>
  <text x="65" y="15" fill="var(--bp-dim)" fontSize="10" opacity="0.4">z</text>
  <text x="70" y="10" fill="var(--bp-dim)" fontSize="12" opacity="0.3">z</text>
</svg>
```

#### 3. No Developer Repositories
**Context:** User profile with no repositories  
**Illustration:** Fish swimming around empty coral  
**Message:** "No repositories yet"  
**Guidance:** "This developer's repos will appear here once published"

```svg
<svg viewBox="0 0 120 120" className="empty-state-illustration">
  <!-- Empty coral/seaweed -->
  <path d="M30 90 Q25 70 30 50 Q35 30 30 20" stroke="var(--bp-border)" strokeWidth="3" fill="none" opacity="0.6"/>
  <path d="M90 90 Q95 70 90 50 Q85 30 90 20" stroke="var(--bp-border)" strokeWidth="3" fill="none" opacity="0.6"/>
  <path d="M60 90 Q55 60 60 30" stroke="var(--bp-border)" strokeWidth="4" fill="none" opacity="0.4"/>
  
  <!-- Swimming fish -->
  <ellipse cx="45" cy="40" rx="8" ry="4" fill="var(--bp-accent)" opacity="0.7" className="swim-animation"/>
  <path d="M37 40 L32 37 L32 43 Z" fill="var(--bp-accent)" opacity="0.7"/>
  
  <ellipse cx="75" cy="60" rx="6" ry="3" fill="var(--bp-accent2)" opacity="0.6" className="swim-animation-reverse"/>
  <path d="M81 60 L86 57 L86 63 Z" fill="var(--bp-accent2)" opacity="0.6"/>
</svg>
```

#### 4. No Commits
**Context:** Repository or branch with no commits  
**Illustration:** Empty scroll floating in water  
**Message:** "No commits yet"  
**Guidance:** "Push your first commit to see it here"

```svg
<svg viewBox="0 0 120 120" className="empty-state-illustration">
  <!-- Empty scroll -->
  <rect x="35" y="30" width="50" height="60" rx="5" fill="var(--bp-surface)" 
        stroke="var(--bp-border)" strokeWidth="2"/>
  
  <!-- Scroll caps -->
  <circle cx="35" cy="35" r="4" fill="var(--bp-border)"/>
  <circle cx="85" cy="35" r="4" fill="var(--bp-border)"/>
  <circle cx="35" cy="85" r="4" fill="var(--bp-border)"/>
  <circle cx="85" cy="85" r="4" fill="var(--bp-border)"/>
  
  <!-- Empty lines (placeholder) -->
  <line x1="45" y1="45" x2="75" y2="45" stroke="var(--bp-border)" strokeWidth="1" opacity="0.3"/>
  <line x1="45" y1="55" x2="65" y2="55" stroke="var(--bp-border)" strokeWidth="1" opacity="0.3"/>
  <line x1="45" y1="65" x2="70" y2="65" stroke="var(--bp-border)" strokeWidth="1" opacity="0.3"/>
  
  <!-- Floating in water bubbles -->
  <circle cx="20" cy="50" r="1.5" fill="var(--bp-accent2)" opacity="0.4" className="bubble-float"/>
  <circle cx="100" cy="40" r="1" fill="var(--bp-accent2)" opacity="0.3" className="bubble-float"/>
</svg>
```

#### 5. No Contributors
**Context:** Repository with no tracked contributors  
**Illustration:** Single jellyfish in empty space  
**Message:** "Swimming solo"  
**Guidance:** "Contributors will appear as they push commits"

```svg
<svg viewBox="0 0 120 120" className="empty-state-illustration">
  <!-- Solo jellyfish -->
  <ellipse cx="60" cy="45" rx="22" ry="14" fill="url(#jellyfish-gradient)"/>
  <path d="M40 55 Q35 75 40 85 Q45 75 50 85 Q55 75 60 85 Q65 75 70 85 Q75 75 80 85" 
        stroke="var(--bp-accent)" strokeWidth="2" fill="none" opacity="0.7"/>
  
  <!-- Gentle current lines -->
  <path d="M15 30 Q30 35 45 30" stroke="var(--bp-dim)" strokeWidth="1" opacity="0.2"/>
  <path d="M75 25 Q90 30 105 25" stroke="var(--bp-dim)" strokeWidth="1" opacity="0.2"/>
  <path d="M10 70 Q25 75 40 70" stroke="var(--bp-dim)" strokeWidth="1" opacity="0.2"/>
</svg>
```

#### 6. No Search Results
**Context:** Search returns no matching repositories  
**Illustration:** Jellyfish with magnifying glass finding nothing  
**Message:** "No matches in these waters"  
**Guidance:** "Try different search terms or explore all repositories"

```svg
<svg viewBox="0 0 120 120" className="empty-state-illustration">
  <!-- Searching jellyfish -->
  <ellipse cx="45" cy="40" rx="20" ry="12" fill="url(#jellyfish-gradient)"/>
  <path d="M28 48 Q25 65 30 75 Q35 65 40 75 Q45 65 50 75 Q55 65 60 75" 
        stroke="var(--bp-accent)" strokeWidth="2" fill="none" opacity="0.6"/>
  
  <!-- Magnifying glass -->
  <circle cx="75" cy="35" r="12" fill="none" stroke="var(--bp-accent)" strokeWidth="2"/>
  <line x1="84" y1="44" x2="95" y2="55" stroke="var(--bp-accent)" strokeWidth="3"/>
  
  <!-- Question marks floating -->
  <text x="85" y="70" fill="var(--bp-dim)" fontSize="16" opacity="0.4">?</text>
  <text x="25" y="25" fill="var(--bp-dim)" fontSize="12" opacity="0.3">?</text>
</svg>
```

#### 7. No Activity in Time Range
**Context:** Contribution chart shows no activity  
**Illustration:** Clock with jellyfish  
**Message:** "Quiet during this period"  
**Guidance:** "Try a different time range or check back later"

```svg
<svg viewBox="0 0 120 120" className="empty-state-illustration">
  <!-- Clock face -->
  <circle cx="60" cy="60" r="30" fill="none" stroke="var(--bp-border)" strokeWidth="2"/>
  <circle cx="60" cy="60" r="3" fill="var(--bp-dim)"/>
  
  <!-- Clock hands (pointing to "empty time") -->
  <line x1="60" y1="60" x2="60" y2="40" stroke="var(--bp-dim)" strokeWidth="2"/>
  <line x1="60" y1="60" x2="75" y2="60" stroke="var(--bp-dim)" strokeWidth="2"/>
  
  <!-- Small jellyfish floating nearby -->
  <ellipse cx="95" cy="30" rx="10" ry="6" fill="url(#jellyfish-gradient)" opacity="0.5"/>
  <path d="M87 34 Q85 40 87 44" stroke="var(--bp-accent)" strokeWidth="1" fill="none" opacity="0.4"/>
  <path d="M95 34 Q93 40 95 44" stroke="var(--bp-accent)" strokeWidth="1" fill="none" opacity="0.4"/>
  <path d="M103 34 Q101 40 103 44" stroke="var(--bp-accent)" strokeWidth="1" fill="none" opacity="0.4"/>
</svg>
```

## CSS Animation Classes

```css
/* Floating animation for jellyfish */
.float-animation {
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-8px); }
}

/* Bubble floating animation */
.bubble-float {
  animation: bubbleFloat 4s ease-in-out infinite;
}

@keyframes bubbleFloat {
  0%, 100% { 
    transform: translateY(0px);
    opacity: 0.3;
  }
  50% { 
    transform: translateY(-12px);
    opacity: 0.8;
  }
}

/* Swimming animation */
.swim-animation {
  animation: swim 6s ease-in-out infinite;
}

.swim-animation-reverse {
  animation: swimReverse 8s ease-in-out infinite;
}

@keyframes swim {
  0%, 100% { transform: translateX(0px); }
  50% { transform: translateX(15px); }
}

@keyframes swimReverse {
  0%, 100% { transform: translateX(0px); }
  50% { transform: translateX(-15px); }
}

/* Empty state container */
.empty-state-illustration {
  width: 120px;
  height: 120px;
  margin: 0 auto 24px;
  display: block;
}

.empty-state-illustration-small {
  width: 80px;
  height: 60px;
  margin: 0 auto 16px;
  display: block;
}
```

## Component Implementation Structure

### EmptyState Component
Create a reusable component for consistent implementation:

```typescript
interface EmptyStateProps {
  type: 'repositories' | 'activity' | 'commits' | 'contributors' | 'search' | 'timeRange';
  title: string;
  description: string;
  actionLabel?: string;
  actionHref?: string;
  size?: 'small' | 'medium' | 'large';
}
```

### File Structure
```
/web/src/components/empty-states/
├── EmptyState.tsx           # Main component
├── illustrations/
│   ├── JellyfishTreasure.tsx
│   ├── SleepingJellyfish.tsx
│   ├── EmptyOcean.tsx
│   ├── EmptyScroll.tsx
│   ├── SoloJellyfish.tsx
│   ├── SearchingJellyfish.tsx
│   └── TimeJellyfish.tsx
├── animations.css
└── index.ts
```

## Implementation Plan

### Phase 1: Core Infrastructure (2-4 hours)
1. Create EmptyState component structure
2. Implement base SVG illustrations
3. Add CSS animations
4. Set up TypeScript types

### Phase 2: Component Integration (4-6 hours)
1. Replace empty states in main explore page
2. Update sidebar activity component
3. Replace address page empty states
4. Update repository page empty states

### Phase 3: Polish & Testing (2-3 hours)
1. Mobile responsiveness testing
2. Animation performance optimization
3. Accessibility testing (screen readers)
4. Cross-browser compatibility

### Phase 4: Documentation (1-2 hours)
1. Component usage documentation
2. Design system updates
3. Animation performance notes

## Acceptance Criteria

### Functional Requirements
- [ ] All identified empty states show custom illustrations
- [ ] Illustrations are consistent in style and branding
- [ ] Messages are helpful and guide user action
- [ ] Animations are smooth and non-intrusive
- [ ] Component is reusable across different contexts

### Performance Requirements
- [ ] SVG illustrations are inline (no external requests)
- [ ] Animations use CSS transforms (GPU accelerated)
- [ ] Total illustration size < 5KB per illustration
- [ ] No impact on page load performance

### Accessibility Requirements
- [ ] Illustrations have appropriate alt text
- [ ] Color contrast meets WCAG AA standards
- [ ] Animations respect prefers-reduced-motion
- [ ] Screen reader friendly descriptions

### Design Requirements
- [ ] Consistent with repo.box brand colors
- [ ] Ocean/jellyfish theme maintained
- [ ] Professional appearance appropriate for developer tools
- [ ] Responsive across mobile and desktop

## Future Enhancements

### Phase 2 Potential Additions
1. **Micro-interactions:** Hover effects on illustrations
2. **Progressive Enhancement:** More complex animations for modern browsers
3. **Contextual Actions:** Smart action buttons based on empty state context
4. **User Onboarding:** Guide new users through first actions
5. **Seasonal Variations:** Subtle theme variations for special events

### Long-term Considerations
1. **Illustration Library:** Expand for other parts of the application
2. **Animation System:** Develop reusable animation component library
3. **User Testing:** A/B test different illustration styles
4. **Internationalization:** Consider text content for multiple languages

---

## Development Notes

This specification provides a comprehensive blueprint for implementing cohesive empty state illustrations throughout the repo.box explorer. The focus on ocean/jellyfish theming maintains brand consistency while providing a delightful user experience that guides users toward productive actions.

The SVG illustrations are designed to be lightweight, scalable, and accessible while maintaining the technical aesthetic appropriate for a developer tool. The implementation plan allows for incremental development and testing to ensure quality and performance.

**Estimated Total Implementation Time:** 9-15 hours across all phases.