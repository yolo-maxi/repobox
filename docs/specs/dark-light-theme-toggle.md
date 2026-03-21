# Dark/Light Theme Toggle - Technical Specification

**Feature:** Dark/Light Theme Toggle for repo.box Dashboard  
**Priority:** P2  
**Timeline:** 2 days  
**Author:** pm-agent  
**Date:** 2026-03-21  

## Overview

Add a theme toggle to the repo.box dashboard header that allows users to switch between dark and light modes. The preference should persist across browser sessions using localStorage and leverage CSS variables for seamless theme switching.

## UI Design

### Toggle Placement
- **Location**: Top-right corner of the dashboard header, next to the project name
- **Position**: Between the "🪸 repo.box" title and the right edge
- **Layout**: Horizontal inline with header content

### Visual Design
- **Component**: Custom toggle switch (not a basic checkbox)
- **Dark mode icon**: 🌙 (moon emoji) 
- **Light mode icon**: ☀️ (sun emoji)
- **Toggle switch**: 24px wide × 14px tall rounded pill
- **Toggle handle**: 12px diameter circle with 2px margin
- **Colors**:
  - Dark mode toggle: `#374151` background, `#F3F4F6` handle
  - Light mode toggle: `#FEF3C7` background, `#F59E0B` handle
- **Animation**: 200ms ease-in-out transition for handle movement and colors

### Header Layout Update
```
┌─────────────────────────────────────────────────────────┐
│ 🪸 repo.box                                    [🌙/☀️] │
│ Build Dashboard — Synthesis Hackathon (deadline: Mar 23) │
└─────────────────────────────────────────────────────────┘
```

## CSS Architecture

### CSS Variables Structure
Create a comprehensive CSS variable system that covers all color tokens used in the dashboard:

```css
:root {
  /* Primary theme colors */
  --bg-primary: #0a0a0a;
  --bg-secondary: #111111;
  --bg-tertiary: #1a1a1a;
  --bg-quaternary: #1a1a2e;
  
  /* Text colors */
  --text-primary: #e0e0e0;
  --text-secondary: #aaa;
  --text-tertiary: #888;
  --text-muted: #666;
  --text-accent: #7eb6ff;
  
  /* Border colors */
  --border-primary: #222;
  --border-secondary: #333;
  --border-tertiary: #555;
  
  /* Status colors */
  --success-bg: #052e16;
  --success-text: #4ade80;
  --success-border: #166534;
  
  --error-bg: #3b1111;
  --error-text: #f87171;
  --error-border: #7f1d1d;
  
  --warning-text: #f59e0b;
  --info-bg: #1a1a2e;
  --info-text: #60a5fa;
  --info-border: #1e3a5f;
  
  /* Interactive elements */
  --hover-bg: #2a2a2a;
  --active-bg: #374151;
  --input-bg: #1a1a1a;
  --button-primary-bg: #052e16;
}

[data-theme="light"] {
  /* Primary theme colors */
  --bg-primary: #ffffff;
  --bg-secondary: #f9fafb;
  --bg-tertiary: #f3f4f6;
  --bg-quaternary: #e5e7eb;
  
  /* Text colors */
  --text-primary: #111827;
  --text-secondary: #374151;
  --text-tertiary: #6b7280;
  --text-muted: #9ca3af;
  --text-accent: #2563eb;
  
  /* Border colors */
  --border-primary: #e5e7eb;
  --border-secondary: #d1d5db;
  --border-tertiary: #9ca3af;
  
  /* Status colors */
  --success-bg: #f0fdf4;
  --success-text: #15803d;
  --success-border: #22c55e;
  
  --error-bg: #fef2f2;
  --error-text: #dc2626;
  --error-border: #ef4444;
  
  --warning-text: #d97706;
  --info-bg: #f0f9ff;
  --info-text: #1d4ed8;
  --info-border: #3b82f6;
  
  /* Interactive elements */
  --hover-bg: #f3f4f6;
  --active-bg: #e5e7eb;
  --input-bg: #ffffff;
  --button-primary-bg: #f0fdf4;
}
```

### Implementation Strategy
1. **CSS Variable Migration**: Replace all hardcoded colors in the styles object with CSS variables
2. **Theme Attribute**: Use `data-theme` attribute on the `<html>` element to control theming
3. **Fallback Support**: Maintain hardcoded colors as fallbacks for older browsers

## State Management

### localStorage Schema
```typescript
interface ThemePreference {
  theme: 'dark' | 'light' | 'auto';
  lastUpdated: number; // timestamp
}

// Storage key: 'repobox:theme'
// Example value: { "theme": "light", "lastUpdated": 1679875200000 }
```

### React State Architecture
```typescript
// hooks/useTheme.ts
interface ThemeState {
  theme: 'dark' | 'light';
  systemPreference: 'dark' | 'light';
  userPreference: 'dark' | 'light' | 'auto';
  setTheme: (theme: 'dark' | 'light' | 'auto') => void;
  toggleTheme: () => void;
}

const useTheme = (): ThemeState => {
  // Implementation details in step-by-step plan
}
```

### Theme Detection Logic
1. **Priority Order**:
   - User's explicit choice (if set) → use that
   - System preference (`prefers-color-scheme`) → use as fallback
   - Default → 'dark' (current behavior)

2. **System Preference Detection**:
   ```typescript
   const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
   ```

3. **Preference Persistence**:
   - Save to localStorage on every theme change
   - Load from localStorage on app initialization
   - Handle localStorage unavailable gracefully

## Implementation Plan

### Phase 1: CSS Variable Foundation (Day 1, 2-3 hours)
1. **Create CSS variables file**:
   - Create `repobox-dashboard/src/styles/themes.css`
   - Define complete variable set for both themes
   - Import in `layout.tsx`

2. **Migrate existing styles**:
   - Update `page.tsx` styles object to use CSS variables
   - Replace hardcoded colors: `#0a0a0a` → `var(--bg-primary)`
   - Test in browser to ensure visual parity

3. **Add theme attribute support**:
   - Update `layout.tsx` to accept `data-theme` attribute
   - Test theme switching manually by changing HTML attribute

### Phase 2: Theme Toggle Component (Day 1, 2-3 hours)
1. **Create ThemeToggle component**:
   - Build in `components/ThemeToggle.tsx`
   - Implement toggle switch design
   - Add click animations and transitions

2. **Create useTheme hook**:
   - Implement in `hooks/useTheme.ts`
   - Handle localStorage read/write
   - Manage system preference detection
   - Provide theme state and toggle function

3. **Integrate into header**:
   - Add ThemeToggle to dashboard header
   - Position according to design specifications
   - Test toggle functionality

### Phase 3: Persistence & Polish (Day 2, 2-3 hours)
1. **localStorage integration**:
   - Implement preference persistence
   - Handle edge cases (localStorage disabled, corrupted data)
   - Add preference migration for existing users

2. **System preference sync**:
   - Listen for system theme changes
   - Update theme when user changes OS preference
   - Handle prefers-color-scheme media query

3. **Testing & refinement**:
   - Test in multiple browsers
   - Verify localStorage persistence
   - Polish animations and transitions
   - Edge case testing (disabled JS, localStorage, etc.)

### Phase 4: Auto-mode (Optional Enhancement)
1. **Three-state toggle**:
   - Extend toggle to support: Dark → Light → Auto → Dark
   - Update UI to show auto mode state
   - Implement auto mode logic (follows system preference)

## File Changes

### New Files
```
repobox-dashboard/src/
├── components/
│   └── ThemeToggle.tsx          # Theme toggle component
├── hooks/
│   └── useTheme.ts             # Theme management hook
└── styles/
    └── themes.css              # CSS variables for both themes
```

### Modified Files

#### `/src/app/layout.tsx`
- Import themes.css
- Add theme context provider
- Update body styles to use CSS variables
- Add data-theme attribute binding

#### `/src/app/page.tsx`
- Import and add ThemeToggle component to header
- Migrate styles object from hardcoded colors to CSS variables
- Update all color references to use theme variables

### File Change Summary
- **New files**: 3 files (ThemeToggle.tsx, useTheme.ts, themes.css)
- **Modified files**: 2 files (layout.tsx, page.tsx)
- **Lines changed**: ~150 lines added, ~50 lines modified

## Technical Considerations

### Performance
- CSS variables have excellent browser support (95%+)
- No runtime CSS-in-JS overhead
- Theme switching is instantaneous (no re-renders)
- localStorage operations are minimal and non-blocking

### Accessibility
- Toggle includes proper ARIA labels
- Respects `prefers-color-scheme` system setting
- High contrast maintained in both themes
- Keyboard navigation support for toggle

### Browser Compatibility
- CSS variables: IE 11+ (graceful degradation)
- localStorage: IE 8+ (feature detection)
- prefers-color-scheme: Chrome 76+, Firefox 67+ (progressive enhancement)

### Future Extensibility
- CSS variable architecture supports additional themes
- Component API allows for theme customization
- Hook architecture can be extended for theme-specific logic

## Success Criteria

### Functional Requirements ✅
- [ ] Toggle switches between dark and light themes
- [ ] User preference persists across browser sessions
- [ ] Respects system preference when no user preference is set
- [ ] All dashboard elements properly themed in both modes

### Visual Requirements ✅
- [ ] Toggle matches design specifications
- [ ] Smooth transitions between themes (200ms)
- [ ] No visual glitches during theme switching
- [ ] Text remains readable in both themes

### Technical Requirements ✅
- [ ] CSS variables used for all themeable properties
- [ ] localStorage integration working correctly
- [ ] No runtime errors in browser console
- [ ] Component is reusable and well-documented

## Risk Mitigation

### Risk: CSS variable browser support
- **Mitigation**: Provide hardcoded fallbacks for each variable
- **Impact**: Low (95%+ browser support)

### Risk: localStorage unavailable
- **Mitigation**: Feature detection and graceful fallback to default theme
- **Impact**: Low (affects <1% of users)

### Risk: Color contrast issues
- **Mitigation**: Use established design system colors with WCAG AA compliance
- **Impact**: Medium (accessibility critical)

### Risk: Performance impact
- **Mitigation**: CSS variables are more performant than runtime theme switching
- **Impact**: Low (performance improved vs. alternatives)

## Deployment Notes

1. **Backward Compatibility**: Existing dark theme users will see no changes
2. **Progressive Enhancement**: Light theme is additive, doesn't break existing functionality
3. **Feature Flag**: Could be deployed behind a feature flag if needed
4. **Analytics**: Consider tracking theme preference for usage insights

---

## Appendix A: Color Mapping

### Current Dark Theme → CSS Variables
```css
/* Backgrounds */
#0a0a0a → var(--bg-primary)
#111111 → var(--bg-secondary)  
#1a1a1a → var(--bg-tertiary)
#1a1a2e → var(--bg-quaternary)

/* Text */
#e0e0e0 → var(--text-primary)
#aaa → var(--text-secondary)
#888 → var(--text-tertiary)
#666 → var(--text-muted)
#7eb6ff → var(--text-accent)

/* Borders */
#222 → var(--border-primary)
#333 → var(--border-secondary)
#555 → var(--border-tertiary)
```

### Light Theme Color Selection Rationale
- **High Contrast**: Ensures WCAG AA compliance
- **Tailwind Inspired**: Uses proven color combinations from Tailwind CSS
- **Brand Consistency**: Maintains repo.box brand identity
- **Accessibility**: Tested with color contrast analyzers

## Appendix B: Implementation Code Samples

### ThemeToggle Component Preview
```typescript
interface ThemeToggleProps {
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export function ThemeToggle({ className, size = 'md' }: ThemeToggleProps) {
  const { theme, toggleTheme } = useTheme();
  
  return (
    <button
      onClick={toggleTheme}
      className={`theme-toggle ${className}`}
      aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
    >
      <div className="toggle-track">
        <div className="toggle-handle">
          {theme === 'dark' ? '🌙' : '☀️'}
        </div>
      </div>
    </button>
  );
}
```

### useTheme Hook Preview
```typescript
export function useTheme(): ThemeState {
  const [theme, setThemeState] = useState<'dark' | 'light'>('dark');
  
  const setTheme = useCallback((newTheme: 'dark' | 'light' | 'auto') => {
    // Implementation
  }, []);
  
  const toggleTheme = useCallback(() => {
    setTheme(theme === 'dark' ? 'light' : 'dark');
  }, [theme, setTheme]);
  
  return { theme, setTheme, toggleTheme };
}
```

---

**Spec Status**: Complete ✅  
**Ready for Implementation**: Yes ✅  
**Estimated Effort**: 2 days (6-8 hours total) ✅