# Mobile-Responsive Explorer Technical Specification

**Priority:** P1  
**Tags:** explorer, ui  
**Date:** 2026-03-21  
**Author:** pm-agent  

## Overview

The repo.box explorer currently provides a desktop-optimized experience that does not adapt well to mobile viewports. This specification details the required changes to make all explorer pages fully responsive across mobile (375px), tablet (768px), and desktop (>768px) viewports.

## Current Architecture Analysis

### Existing Components
- **Explorer Landing** (`/explore/page.tsx`): Stats grid, repository list, activity feed, search
- **Repository View** (`/explore/[address]/[name]/page.tsx`): Repository header, tabs, file browser, commits list  
- **BranchSelector** (`/components/BranchSelector.tsx`): Branch dropdown component
- **Styling**: Centralized in `globals.css` with minimal responsive patterns

### Existing Responsive Patterns
The codebase already includes responsive breakpoints for:
- Footer grid: 640px breakpoint
- Documentation sidebar: 768px breakpoint  
- Case studies layout: 640px breakpoint
- Markdown rendering: 768px breakpoint

## Technical Requirements

### 1. Breakpoint Strategy

**Primary Breakpoints:**
- **Mobile**: 375px - 640px
- **Tablet**: 640px - 768px  
- **Desktop**: 768px+

**Testing Viewports:**
- Mobile: 375px × 667px (iPhone SE)
- Tablet: 768px × 1024px (iPad)
- Desktop: 1024px+ (Standard desktop)

### 2. Explorer Landing Page Responsive Changes

#### 2.1 Header Section
**Current**: Horizontal layout with title and search input
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-header-content {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }
  
  .explore-search-input {
    width: 100%;
    font-size: 16px; /* Prevent zoom on iOS */
  }
}
```

#### 2.2 Stats Grid
**Current**: 3-column horizontal layout
**Mobile Changes**:
```css
.explore-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

@media (max-width: 640px) {
  .explore-stats {
    grid-template-columns: 2fr 1fr; /* commits | owners */
    grid-template-rows: auto auto;
  }
  
  .explore-stat-card:first-child {
    grid-column: 1 / -1; /* repositories spans full width */
  }
}
```

#### 2.3 Repository Cards
**Current**: Wide horizontal cards
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-repo-list {
    gap: 12px;
  }
  
  .explore-repo-card {
    padding: 16px;
  }
  
  .explore-repo-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .explore-repo-name {
    font-size: 14px;
  }
  
  .explore-repo-owner {
    font-size: 11px;
  }
}
```

#### 2.4 Activity Feed
**Current**: Standard vertical layout
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-activity-item {
    padding: 12px 16px;
  }
  
  .explore-activity-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}
```

### 3. Repository View Page Responsive Changes

#### 3.1 Repository Header
**Current**: Horizontal layout with stats on right
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-repo-header {
    flex-direction: column;
    gap: 16px;
  }
  
  .explore-repo-stats {
    align-self: stretch;
    justify-content: space-between;
  }
  
  .explore-clone-url {
    font-size: 10px;
    word-break: break-all;
  }
}
```

#### 3.2 Navigation Tabs
**Current**: Horizontal tab bar
**Mobile Changes**:
```css
@media (max-width: 640px) {
  .explore-tabs {
    overflow-x: auto;
    scrollbar-width: none;
    -webkit-overflow-scrolling: touch;
  }
  
  .explore-tabs::-webkit-scrollbar {
    display: none;
  }
  
  .explore-tab {
    flex-shrink: 0;
    min-width: 80px;
  }
}
```

#### 3.3 File Tree → Breadcrumb Navigation
**Current**: Hierarchical file tree navigation  
**Mobile Changes**: Collapse to breadcrumb-only navigation

**Implementation**:
```css
@media (max-width: 768px) {
  .explore-breadcrumb {
    flex-wrap: wrap;
    gap: 4px;
  }
  
  .explore-breadcrumb-item {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  /* Hide intermediate breadcrumb items on very small screens */
  @media (max-width: 480px) {
    .explore-breadcrumb-separator:not(:last-of-type) {
      display: none;
    }
    
    .explore-breadcrumb-item:not(:first-child):not(:last-child) {
      display: none;
    }
  }
}
```

**Component Updates** (`page.tsx`):
```typescript
// Add mobile-responsive breadcrumb truncation
const truncatedPathParts = useMemo(() => {
  if (typeof window !== 'undefined' && window.innerWidth <= 480) {
    // Show only first and last on very small screens
    if (pathParts.length > 2) {
      return [pathParts[0], '...', pathParts[pathParts.length - 1]];
    }
  }
  return pathParts;
}, [pathParts, /* window resize listener */]);
```

#### 3.4 File List
**Current**: Horizontal layout with file info and size
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-file-item {
    padding: 12px 16px;
  }
  
  .explore-file-info {
    flex: 1;
    min-width: 0;
  }
  
  .explore-file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .explore-file-size {
    font-size: 10px;
    flex-shrink: 0;
  }
}
```

#### 3.5 Commits List - Vertical Stacking
**Current**: Horizontal layout with metadata inline
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-commit-item {
    padding: 16px;
  }
  
  .explore-commit-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    margin-top: 8px;
  }
  
  .explore-commit-hash {
    order: -1; /* Move hash to top */
    font-size: 11px;
  }
  
  .explore-commit-author {
    font-size: 10px;
  }
}
```

### 4. Branch Selector Responsive Behavior

#### 4.1 Dropdown Positioning
**Current**: Fixed positioning below trigger
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .branch-dropdown {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 90vw;
    max-width: 320px;
    max-height: 60vh;
  }
  
  /* Add backdrop */
  .branch-dropdown::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: -1;
  }
}
```

#### 4.2 Filter Input
**Mobile Optimization**:
```css
@media (max-width: 768px) {
  .branch-filter-input {
    font-size: 16px; /* Prevent iOS zoom */
  }
}
```

### 5. Search Interface Responsive Behavior

#### 5.1 Search Results Layout
**Mobile Changes**:
```css
@media (max-width: 768px) {
  .explore-search-section {
    margin-bottom: 24px;
  }
  
  .explore-search-section h3 {
    font-size: 14px;
    margin-bottom: 12px;
  }
}
```

### 6. Touch and Interaction Improvements

#### 6.1 Touch Targets
All interactive elements must meet 44px minimum touch target:
```css
@media (max-width: 768px) {
  .explore-tab,
  .explore-file-item,
  .explore-repo-card,
  .branch-selector-trigger {
    min-height: 44px;
  }
}
```

#### 6.2 Tap Feedback
```css
@media (max-width: 768px) {
  .explore-file-item:active,
  .explore-repo-card:active,
  .explore-tab:active {
    background: rgba(79, 195, 247, 0.1);
    transform: scale(0.98);
  }
}
```

### 7. Performance Considerations

#### 7.1 Virtual Scrolling for Large Lists
For repositories and commits lists exceeding 50 items:
- Implement `react-window` for virtualized scrolling
- Maintain 44px minimum row height on mobile
- Add pull-to-refresh for mobile

#### 7.2 Image Optimization
- Lazy load file icons
- Use CSS sprites for common file type icons
- Optimize branch selector dropdown rendering

## Implementation Plan

### Phase 1: Core Layout (Sprint 1)
1. ✅ Stats grid responsive layout (2×2 on mobile)
2. ✅ Repository header stacking
3. ✅ File tree → breadcrumb collapse
4. ✅ Commits list vertical stacking

### Phase 2: Navigation and Interaction (Sprint 1)
1. ✅ Branch selector modal behavior on mobile
2. ✅ Tab scrolling
3. ✅ Touch target optimization
4. ✅ Breadcrumb truncation logic

### Phase 3: Polish and Performance (Sprint 2)  
1. ⏳ Virtual scrolling implementation
2. ⏳ Pull-to-refresh
3. ⏳ Animation transitions
4. ⏳ Accessibility improvements

## Testing Strategy

### Automated Testing
- Add responsive breakpoint tests using `@testing-library/react`
- Visual regression tests at 375px, 768px, 1024px
- Touch interaction tests

### Manual Testing Checklist

#### Mobile (375px)
- [ ] Stats display in 2×2 grid format
- [ ] Repository cards stack properly  
- [ ] File tree collapses to breadcrumbs
- [ ] Commit metadata stacks vertically
- [ ] Branch selector opens as modal
- [ ] All touch targets ≥44px
- [ ] Horizontal scrolling eliminated
- [ ] Search results readable

#### Tablet (768px)
- [ ] Intermediate layout between mobile/desktop
- [ ] File tree remains functional
- [ ] Stats in 3-column layout
- [ ] Branch dropdown positions correctly

#### Cross-browser Mobile
- [ ] iOS Safari: No zoom on input focus
- [ ] Android Chrome: Smooth scrolling
- [ ] Firefox Mobile: Layout integrity

### Performance Benchmarks
- First Contentful Paint: <2.5s on 3G
- Largest Contentful Paint: <4s on 3G
- Touch response time: <100ms

## Dependencies

### New Dependencies (Optional)
- `react-window`: Virtual scrolling for large lists
- `react-intersection-observer`: Lazy loading optimization

### Browser Support
- iOS Safari 14+
- Android Chrome 90+
- Firefox Mobile 90+
- Samsung Internet 14+

## Rollout Strategy

### Feature Flags
```typescript
const RESPONSIVE_EXPLORER_ENABLED = process.env.NEXT_PUBLIC_RESPONSIVE_EXPLORER === 'true';
```

### A/B Testing
- 50/50 split between legacy and responsive layouts
- Track bounce rate, session duration, and user interactions
- Mobile-first rollout (mobile users → tablet → desktop)

### Monitoring Metrics
- Mobile user engagement (session duration, page views)
- Error rates on mobile devices
- Performance metrics across breakpoints

## Risk Mitigation

### Backwards Compatibility
- All existing desktop functionality preserved
- Progressive enhancement approach
- Graceful degradation for older browsers

### Content Overflow Handling
- Text truncation with expand/collapse
- Horizontal scrolling fallbacks where needed
- Maximum content width constraints

### Edge Cases
- Very long repository names → truncation
- Deep folder hierarchies → breadcrumb compression
- Large file lists → virtualization
- Network timeouts → skeleton loading states

---

**Acceptance Criteria:**
1. All explorer pages render correctly at 375px and 768px viewports
2. File tree navigation transforms to breadcrumb-based navigation on mobile
3. Stats grid uses 2×2 layout on mobile screens
4. Commit lists stack metadata vertically on mobile
5. All interactive elements meet 44px minimum touch target size
6. No horizontal scrolling occurs on any mobile viewport
7. Branch selector functions properly across all screen sizes
8. Performance benchmarks met for mobile connections