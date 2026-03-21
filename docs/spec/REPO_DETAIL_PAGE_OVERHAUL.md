# Technical Specification: Repo Detail Page Comprehensive Overhaul

**Task**: Fix repo detail page (explorer) — comprehensive overhaul  
**Priority**: P0  
**File**: `/web/src/app/explore/[address]/[name]/page.tsx`  
**Date**: 2026-03-21  
**Author**: PM Agent  

## Overview

The current repo detail page at `/explore/[address]/[name]` has several critical issues affecting UX and functionality. This spec provides a complete technical approach to fix all identified problems while maintaining existing functionality and improving the overall user experience.

## Current Issues Analysis

### 1. Layout Problems
- **Issue**: Full-width layout instead of sidebar + constrained column
- **Current**: Uses `.explore-page` with full-width content
- **Expected**: Should match `/explore` page with left sidebar + constrained main column

### 2. Broken Tabs
- **Issue**: Files, Commits, Config, Contributors tabs have UI problems
- **Root Cause**: Poor visual hierarchy, spacing, and interaction states
- **Current**: Functional but poor UX

### 3. SSH Clone URL
- **Issue**: SSH authentication not implemented
- **Current**: Shows both HTTPS and SSH options in `CloneUrlWidget`
- **Expected**: Remove SSH option entirely

### 4. Contributor Count Inconsistency  
- **Issue**: Explorer listing shows different contributor count than detail page
- **Current**: Two different counting methods/APIs
- **Expected**: Consistent count across all views

### 5. Missing Contribution Chart
- **Issue**: Contributors tab lacks visual representation
- **Current**: Just cards/list
- **Expected**: Chart showing contribution timeline or distribution

### 6. Language Bar Duplication
- **Issue**: "Other" appears twice in language breakdown
- **Current**: Binary/data files counted separately as "Other"
- **Expected**: Single "Other" category, exclude binary files

### 7. URL Schema
- **Issue**: Non-GitHub-style URLs for navigation
- **Current**: Custom navigation pattern
- **Expected**: `/tree/{branch}/{path}`, `/blob/{branch}/{path}`, `/commits/{branch}`

## Technical Approach

### 1. Layout System Redesign

#### 1.1 Create Consistent Layout Structure
```typescript
// New layout component structure
export default function RepoPage() {
  return (
    <div className="explore-layout">
      <ExploreHeader />
      <div className="explore-container">
        <ExploreSidebar />
        <main className="explore-main">
          <RepoHeader />
          <RepoTabs />
          <RepoContent />
        </main>
      </div>
    </div>
  );
}
```

#### 1.2 CSS Layout Updates
```css
.explore-layout {
  min-height: 100vh;
  background: var(--bp-bg);
}

.explore-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: 24px;
}

.explore-main {
  min-width: 0; /* Prevent overflow */
}
```

#### 1.3 Shared Components Extraction
- Extract header navigation to `<ExploreHeader />`
- Create reusable `<ExploreSidebar />` for both explore home and detail pages
- Ensure consistent styling and behavior

### 2. Tab System Improvements

#### 2.1 Enhanced Tab Component
```typescript
interface TabConfig {
  id: string;
  label: string;
  icon?: React.ReactNode;
  count?: number;
  disabled?: boolean;
}

const tabs: TabConfig[] = [
  { id: 'readme', label: 'README', icon: <FileIcon /> },
  { id: 'files', label: 'Files', icon: <FolderIcon />, count: fileCount },
  { id: 'commits', label: 'Commits', icon: <GitCommitIcon />, count: commitCount },
  { id: 'contributors', label: 'Contributors', icon: <UsersIcon />, count: contributorCount },
  { id: 'config', label: 'Config', icon: <SettingsIcon /> },
];
```

#### 2.2 Improved Visual Design
- Better spacing and typography
- Clear active/inactive states
- Consistent icons and count badges
- Improved mobile responsive behavior

### 3. Clone URL Widget Fix

#### 3.1 Remove SSH Option
```typescript
// Update CloneUrlWidget.tsx
export default function CloneUrlWidget({ ownerAddress, repoName }: Props) {
  const httpsUrl = `https://git.repo.box/${ownerAddress}/${repoName}.git`;
  
  return (
    <div className="clone-url-widget">
      <div className="clone-url-group">
        <div className="clone-url-label">HTTPS (EVM-authenticated)</div>
        <CloneUrlInput url={httpsUrl} />
      </div>
      <AuthSetupInstructions />
    </div>
  );
}
```

#### 3.2 Enhanced Help Section
- Clearer instructions for credential helper setup
- Emphasis on EVM-based authentication
- Remove all SSH references

### 4. Contributor Count Consistency

#### 4.1 API Standardization
```typescript
// Ensure both endpoints use same counting logic
// /api/explorer/repos - should use same contributor counting as detail page
// /api/explorer/repos/[address]/[name]/contributors - standardize response

interface ContributorCount {
  unique_signers: number; // EVM addresses who have pushed commits
  calculation_method: 'unique_signer_addresses';
}
```

#### 4.2 Caching Strategy
- Cache contributor counts to avoid recalculation
- Invalidate cache on new pushes
- Consistent counting logic across all endpoints

### 5. Contribution Chart Implementation

#### 5.1 Chart Component
```typescript
interface ContributionChartProps {
  contributors: Contributor[];
  timeRange: 'week' | 'month' | 'year';
}

export function ContributionChart({ contributors, timeRange }: ContributionChartProps) {
  // Implementation using lightweight charting library
  // Show commit frequency over time
  // Color-coded by contributor
  return <div className="contribution-chart">{/* Chart implementation */}</div>;
}
```

#### 5.2 Data Structure
```typescript
interface ContributorActivity {
  address: string;
  dailyCommits: Record<string, number>; // YYYY-MM-DD -> count
  totalCommits: number;
  firstCommit: string;
  lastCommit: string;
}
```

### 6. Language Bar Fix

#### 6.1 File Type Filtering
```typescript
// Update language calculation to exclude binary files
const EXCLUDED_EXTENSIONS = [
  '.png', '.jpg', '.jpeg', '.gif', '.webp', '.ico',
  '.mp3', '.mp4', '.avi', '.mov', '.wav',
  '.zip', '.tar', '.gz', '.7z',
  '.exe', '.dll', '.so', '.dylib',
  '.pdf', '.doc', '.docx', '.xls', '.xlsx'
];

function shouldIncludeInLanguageStats(filename: string): boolean {
  const ext = path.extname(filename).toLowerCase();
  return !EXCLUDED_EXTENSIONS.includes(ext);
}
```

#### 6.2 Improved Aggregation
```typescript
function aggregateLanguageStats(files: FileAnalysis[]): LanguageStats[] {
  const filteredFiles = files.filter(file => 
    shouldIncludeInLanguageStats(file.path) && 
    file.type === 'text' // Only text files
  );
  
  // Group and aggregate
  const grouped = groupBy(filteredFiles, 'language');
  const withOthers = aggregateSmallLanguages(grouped, 0.05); // 5% threshold
  
  return withOthers;
}
```

### 7. GitHub-Style URL Implementation

#### 7.1 New Routing Structure
```
Current:
/explore/[address]/[name] - repo home
/explore/[address]/[name]?tab=files&path=src - file navigation

New:
/explore/[address]/[name] - repo home (README)
/explore/[address]/[name]/tree/[branch]/[...path] - directory view  
/explore/[address]/[name]/blob/[branch]/[...path] - file view
/explore/[address]/[name]/commits/[branch] - commits for branch
/explore/[address]/[name]/commit/[hash] - single commit view
```

#### 7.2 Next.js Route Structure
```
app/explore/[address]/[name]/
├── page.tsx                           # Repo home
├── tree/[branch]/[...path]/page.tsx   # Directory view
├── blob/[branch]/[...path]/page.tsx   # File view  
├── commits/[branch]/page.tsx          # Commits list
└── commit/[hash]/page.tsx             # Single commit
```

#### 7.3 URL Generation Utilities
```typescript
export const repoUrls = {
  home: (addr: string, name: string) => 
    `/explore/${addr}/${name}`,
  tree: (addr: string, name: string, branch: string, path: string = '') =>
    `/explore/${addr}/${name}/tree/${branch}/${path}`,
  blob: (addr: string, name: string, branch: string, path: string) =>
    `/explore/${addr}/${name}/blob/${branch}/${path}`,
  commits: (addr: string, name: string, branch: string) =>
    `/explore/${addr}/${name}/commits/${branch}`,
  commit: (addr: string, name: string, hash: string) =>
    `/explore/${addr}/${name}/commit/${hash}`,
};
```

## File Changes Required

### New Files
```
web/src/app/explore/[address]/[name]/tree/[branch]/[...path]/page.tsx
web/src/app/explore/[address]/[name]/blob/[branch]/[...path]/page.tsx
web/src/app/explore/[address]/[name]/commits/[branch]/page.tsx
web/src/components/explore/ExploreHeader.tsx
web/src/components/explore/ExploreSidebar.tsx  
web/src/components/explore/ContributionChart.tsx
web/src/lib/repoUrls.ts
```

### Modified Files
```
web/src/app/explore/[address]/[name]/page.tsx - Major refactor
web/src/components/CloneUrlWidget.tsx - Remove SSH option
web/src/components/RepoStatsCards.tsx - Fix language bar duplication
web/src/app/api/explorer/repos/route.ts - Consistent contributor counting
web/src/app/globals.css - Layout system updates
```

### API Changes

#### New Endpoints
```
GET /api/explorer/repos/[address]/[name]/activity
POST /api/explorer/repos/[address]/[name]/invalidate-cache
```

#### Modified Endpoints
```
GET /api/explorer/repos/[address]/[name]/stats
- Add file type filtering for language stats
- Exclude binary files from calculations

GET /api/explorer/repos/[address]/[name]/contributors  
- Add daily activity data for charts
- Consistent counting with listing endpoint
```

## Component Architecture

### 1. Layout Components
```
<ExploreLayout>
  <ExploreHeader />
  <div className="explore-container">
    <ExploreSidebar>
      <StatsOverview />
      <SortOptions />
      <RecentActivity />
    </ExploreSidebar>
    <main>
      <RepoHeader />
      <CloneUrlWidget />
      <RepoStatsCards />
      <RepoTabs />
      <RepoTabContent />
    </main>
  </div>
</ExploreLayout>
```

### 2. Tab Content Components
```
<RepoTabContent activeTab={activeTab}>
  <ReadmeTab content={readme} />
  <FilesTab files={files} path={currentPath} />
  <CommitsTab commits={commits} branch={selectedBranch} />
  <ContributorsTab contributors={contributors} />
  <ConfigTab config={repoConfig} owner={ownerAddress} />
</RepoTabContent>
```

## Testing Strategy

### 1. Visual Regression Tests
- Screenshot comparison for layout changes
- Mobile/desktop responsive breakpoints
- Tab interaction states

### 2. API Integration Tests  
- Contributor count consistency across endpoints
- Language bar duplication prevention
- Branch navigation with new URL structure

### 3. User Journey Tests
- Clone URL copy functionality (HTTPS only)
- Tab navigation and state management
- File/directory navigation with new URLs
- Branch switching maintains context

### 4. Performance Tests
- Page load times with new layout
- Chart rendering performance
- API response caching effectiveness

## Implementation Plan

### Phase 1: Layout Foundation (Day 1)
1. Extract shared layout components
2. Update CSS grid system
3. Implement consistent header/sidebar

### Phase 2: Core Functionality (Day 2)  
1. Fix tab system with improved UI
2. Remove SSH from clone widget
3. Fix contributor count consistency
4. Fix language bar duplication

### Phase 3: Advanced Features (Day 3)
1. Implement contribution charts
2. GitHub-style URL routing
3. Enhanced navigation and breadcrumbs

### Phase 4: Polish & Testing (Day 4)
1. Responsive design refinements
2. Performance optimizations
3. Comprehensive testing
4. Documentation updates

## Backward Compatibility

### URL Redirects
```typescript
// Handle old URL structure in middleware
export function middleware(request: NextRequest) {
  const url = request.nextUrl.clone();
  
  // Redirect old tab-based URLs to new structure
  if (url.searchParams.has('tab') && url.searchParams.get('tab') === 'files') {
    const path = url.searchParams.get('path') || '';
    url.pathname = `${url.pathname}/tree/HEAD/${path}`;
    url.search = '';
    return NextResponse.redirect(url);
  }
}
```

### API Versioning
- Maintain existing API responses
- Add new fields without breaking changes
- Deprecation warnings for old usage patterns

## Risk Mitigation

### 1. Scope Creep Prevention
- Clearly defined requirements
- Incremental implementation phases
- Regular review checkpoints

### 2. Performance Concerns
- Lazy loading for chart components
- Efficient caching strategies  
- Progressive enhancement approach

### 3. Browser Compatibility
- Graceful degradation for older browsers
- CSS feature detection
- Polyfills where necessary

## Success Criteria

### User Experience
- ✅ Consistent layout with explore homepage
- ✅ Intuitive tab navigation  
- ✅ Clear, actionable clone instructions
- ✅ Visual contribution insights
- ✅ Accurate language statistics

### Technical Quality
- ✅ No console errors or warnings
- ✅ Responsive across all devices
- ✅ < 2s page load time
- ✅ Accessible to screen readers
- ✅ SEO-friendly URL structure

### Data Consistency
- ✅ Contributor counts match across all views
- ✅ Language statistics exclude binary files
- ✅ Branch navigation maintains context
- ✅ File paths resolve correctly

This specification provides a comprehensive roadmap for fixing all identified issues while improving the overall architecture and user experience of the repo detail page.