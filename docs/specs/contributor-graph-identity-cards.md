# Contributor Graph / Identity Cards - Implementation Specification

## Overview

Implement a visual contributor graph showing all unique EVM signers for a repository as interactive identity cards. Each identity card displays the contributor's address, alias (if available), contribution metrics, and a mini contribution heatmap, providing a comprehensive view of who has contributed to the project over time.

## Feature Requirements

### Core Identity Card Components
- **EVM Address Display**: Full address with copy-to-clipboard functionality
- **Alias Resolution**: Show human-readable aliases when available (from database)
- **Commit Count**: Total number of commits by this contributor
- **Date Range**: First and last commit timestamps
- **Mini Contribution Heatmap**: 12-week activity visualization (GitHub-style)
- **Click-to-Navigate**: Link each card to the contributor's address page

### UI Integration
- Add as a new "Contributors" tab in the repository explorer
- Responsive grid layout (1-3 columns based on screen width)
- Sort options: by commit count, recent activity, or alphabetical
- Search/filter functionality for repositories with many contributors

## Technical Architecture

### 1. Backend API Enhancement

#### New Endpoint: `GET /api/explorer/repos/[address]/[name]/contributors`

**Location**: `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/contributors/route.ts`

**Query Parameters**:
```typescript
interface ContributorQuery {
  branch?: string;      // Target branch (default: HEAD)
  sort?: 'commits' | 'recent' | 'alphabetical';
  limit?: number;       // Max results (default: 50)
  search?: string;      // Filter by address/alias
}
```

**Response Schema**:
```typescript
interface ContributorResponse {
  contributors: ContributorCard[];
  total_count: number;
  analysis_branch: string;
  analysis_time_ms: number;
}

interface ContributorCard {
  address: string;
  alias?: string;
  avatar_seed: string;
  commit_count: number;
  first_commit_date: string;    // ISO 8601
  last_commit_date: string;     // ISO 8601
  contribution_heatmap: HeatmapData[];
  is_owner: boolean;
}

interface HeatmapData {
  week_start: string;           // ISO 8601 week start (Monday)
  daily_commits: number[];      // [Mon, Tue, Wed, Thu, Fri, Sat, Sun]
}
```

### 2. Backend Implementation Strategy

#### Git Analysis Engine

**Extend `repobox-server/src/git.rs`** with contributor analysis:

```rust
pub(crate) fn analyze_repository_contributors(
    repo_path: &Path,
    branch: Option<&str>
) -> std::io::Result<Vec<ContributorAnalysis>> {
    // Use git rev-list to get all commits
    // Parse commit objects to extract EVM signatures  
    // Group by signer address and aggregate statistics
    // Generate heatmap data for last 12 weeks
}

pub(crate) struct ContributorAnalysis {
    pub address: String,
    pub commit_timestamps: Vec<i64>,
    pub commit_hashes: Vec<String>,
    pub total_commits: u32,
}
```

**Algorithm Overview**:
1. Execute `git rev-list --all --format=raw [branch]` to get commit objects
2. Parse each commit using existing `extract_signer_from_commit_text()` from current codebase
3. Build HashMap<EVM_Address, Vec<CommitData>> for grouping
4. Calculate aggregate statistics and heatmap data
5. Integrate with alias lookup from database

#### Database Integration

**Leverage existing tables**:
- `aliases` table: Map EVM addresses to human-readable names
- `push_log` table: Fallback data source for performance optimization
- No new schema changes required

**Alias Resolution**:
```sql
SELECT alias FROM aliases WHERE address = ?
```

### 3. Frontend Component Architecture

#### Core Components

**1. ContributorIdentityCards** (Container Component)
```typescript
// Location: /home/xiko/repobox/web/src/components/ContributorIdentityCards.tsx
interface ContributorIdentityCardsProps {
  address: string;
  name: string; 
  branch?: string;
}

export default function ContributorIdentityCards({ address, name, branch }: ContributorIdentityCardsProps) {
  // Fetch contributors data from API
  // Implement sorting and filtering logic
  // Render grid of IdentityCard components
  // Handle loading and error states
}
```

**2. IdentityCard** (Individual Card Component)
```typescript
// Location: /home/xiko/repobox/web/src/components/IdentityCard.tsx
interface IdentityCardProps {
  contributor: ContributorCard;
  onClick: () => void;
}

export default function IdentityCard({ contributor, onClick }: IdentityCardProps) {
  // Render avatar (generated from address hash)
  // Display address with copy button
  // Show alias if available
  // Render contribution metrics
  // Embed mini heatmap
}
```

**3. MiniContributionHeatmap** (Heatmap Visualization)
```typescript
// Location: /home/xiko/repobox/web/src/components/MiniContributionHeatmap.tsx
interface MiniContributionHeatmapProps {
  heatmapData: HeatmapData[];
  compact?: boolean;
}

export default function MiniContributionHeatmap({ heatmapData, compact }: MiniContributionHeatmapProps) {
  // Render SVG grid of contribution squares
  // Apply color intensity based on commit count
  // Add hover tooltips with exact numbers
}
```

#### Integration Points

**Repository Page Tab Addition**
- Modify `/home/xiko/repobox/web/src/app/explore/[address]/[name]/page.tsx`
- Add "Contributors" to existing tab system
- Integrate ContributorIdentityCards component

**Stats Card Enhancement**  
- Update RepoStatsCards to show unique contributor count
- Link contributor count to Contributors tab

### 4. UI/UX Design Specifications

#### Identity Card Layout
```
┌─────────────────────────────────────────┐
│ [Avatar] 0x1234...5678          [Copy]  │
│          alice-developer                │
│                                         │
│ 🔧 23 commits                          │
│ 📅 Mar 1 - Mar 20, 2026               │
│                                         │
│ ┌─────────────────────────────┐        │
│ │ ▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢  │ (12w) │
│ │ ▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢▢  │       │
│ └─────────────────────────────┘        │
└─────────────────────────────────────────┘
```

#### Visual Hierarchy
- **Primary**: EVM address (larger, bold font)
- **Secondary**: Alias name (medium, colored accent) 
- **Tertiary**: Metrics and dates (smaller, muted color)
- **Interactive**: Copy button, hover states, click affordance

#### Responsive Grid Layout
```css
.contributor-cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 24px;
  padding: 24px;
}

@media (max-width: 768px) {
  .contributor-cards-grid {
    grid-template-columns: 1fr;
    gap: 16px;
    padding: 16px;
  }
}
```

### 5. Data Processing Algorithms

#### Heatmap Generation Algorithm

```typescript
function generateContributionHeatmap(commitTimestamps: number[]): HeatmapData[] {
  const WEEKS_TO_SHOW = 12;
  const MS_PER_DAY = 24 * 60 * 60 * 1000;
  const MS_PER_WEEK = 7 * MS_PER_DAY;
  
  const now = Date.now();
  const startTime = now - (WEEKS_TO_SHOW * MS_PER_WEEK);
  
  // Create week buckets
  const weeks: Map<string, number[]> = new Map();
  
  for (let i = 0; i < WEEKS_TO_SHOW; i++) {
    const weekStart = new Date(startTime + (i * MS_PER_WEEK));
    const mondayStart = getMondayOfWeek(weekStart);
    weeks.set(mondayStart.toISOString(), [0, 0, 0, 0, 0, 0, 0]);
  }
  
  // Fill in commit data
  commitTimestamps.forEach(timestamp => {
    if (timestamp >= startTime / 1000) {
      const commitDate = new Date(timestamp * 1000);
      const weekStart = getMondayOfWeek(commitDate);
      const dayIndex = commitDate.getDay() === 0 ? 6 : commitDate.getDay() - 1; // Mon=0, Sun=6
      
      const weekKey = weekStart.toISOString();
      if (weeks.has(weekKey)) {
        weeks.get(weekKey)![dayIndex]++;
      }
    }
  });
  
  return Array.from(weeks.entries())
    .map(([week_start, daily_commits]) => ({ week_start, daily_commits }))
    .sort((a, b) => new Date(a.week_start).getTime() - new Date(b.week_start).getTime());
}
```

#### Contributor Sorting Logic

```typescript
function sortContributors(
  contributors: ContributorCard[], 
  sortBy: 'commits' | 'recent' | 'alphabetical'
): ContributorCard[] {
  switch (sortBy) {
    case 'commits':
      return contributors.sort((a, b) => b.commit_count - a.commit_count);
    case 'recent':
      return contributors.sort((a, b) => 
        new Date(b.last_commit_date).getTime() - new Date(a.last_commit_date).getTime()
      );
    case 'alphabetical':
      return contributors.sort((a, b) => {
        const nameA = a.alias || a.address;
        const nameB = b.alias || b.address;
        return nameA.localeCompare(nameB);
      });
  }
}
```

### 6. Performance Optimization

#### Caching Strategy
- **In-Memory Cache**: Repository contributor data (TTL: 5 minutes)
- **Cache Key Format**: `contributors:${address}:${name}:${branch}`
- **Invalidation**: On new pushes to repository (webhook-based)

#### Lazy Loading Implementation
```typescript
// Progressive loading for repositories with many contributors
const useContributorsPagination = (address: string, name: string) => {
  const [contributors, setContributors] = useState<ContributorCard[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  
  const loadMore = useCallback(async () => {
    if (loading || !hasMore) return;
    
    setLoading(true);
    const response = await fetch(`/api/explorer/repos/${address}/${name}/contributors?offset=${contributors.length}&limit=20`);
    const data = await response.json();
    
    setContributors(prev => [...prev, ...data.contributors]);
    setHasMore(data.contributors.length === 20);
    setLoading(false);
  }, [address, name, contributors.length, loading, hasMore]);
  
  return { contributors, loadMore, hasMore, loading };
};
```

### 7. Database Queries

#### Existing Table Utilization

**Fetch Contributors with Aliases**:
```sql
SELECT DISTINCT 
  pl.pusher_address as address,
  a.alias,
  COUNT(pl.id) as commit_count,
  MIN(pl.pushed_at) as first_commit,
  MAX(pl.pushed_at) as last_commit
FROM push_log pl
LEFT JOIN aliases a ON pl.pusher_address = a.address  
WHERE pl.address = ? AND pl.name = ?
GROUP BY pl.pusher_address, a.alias
ORDER BY commit_count DESC;
```

**Check Repository Ownership**:
```sql
SELECT owner_address FROM repos WHERE address = ? AND name = ?;
```

### 8. CSS Styling Integration

#### Extend Existing Design System

**Add to `/home/xiko/repobox/web/src/app/globals.css`**:

```css
/* Contributor Identity Cards */
.contributor-cards-container {
  padding: 24px;
  background: var(--bp-surface);
}

.contributor-cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 24px;
  margin-top: 24px;
}

.identity-card {
  background: var(--bp-bg);
  border: 1px solid var(--bp-border);
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.identity-card:hover {
  border-color: var(--bp-accent);
  box-shadow: 0 4px 12px rgba(79, 195, 247, 0.1);
  transform: translateY(-2px);
}

.identity-card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.identity-card-avatar {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  background: linear-gradient(135deg, var(--bp-accent), var(--bp-accent2));
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  color: var(--bp-bg);
}

.identity-card-info {
  flex: 1;
  min-width: 0;
}

.identity-card-address {
  font-family: var(--font-mono), "JetBrains Mono", monospace;
  font-size: 13px;
  color: var(--bp-heading);
  font-weight: 600;
  margin-bottom: 4px;
}

.identity-card-alias {
  font-size: 12px;
  color: var(--bp-accent);
  font-weight: 500;
}

.identity-card-copy {
  background: transparent;
  border: 1px solid var(--bp-border);
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 11px;
  color: var(--bp-dim);
  cursor: pointer;
  transition: all 0.2s ease;
}

.identity-card-copy:hover {
  background: var(--bp-surface);
  border-color: var(--bp-accent);
  color: var(--bp-accent);
}

.identity-card-metrics {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 16px;
  font-size: 12px;
}

.identity-card-metric {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--bp-text);
}

.identity-card-metric-icon {
  color: var(--bp-accent);
}

.identity-card-dates {
  font-size: 11px;
  color: var(--bp-dim);
  margin-bottom: 16px;
}

.identity-card-owner-badge {
  background: var(--bp-accent);
  color: var(--bp-bg);
  font-size: 9px;
  padding: 3px 8px;
  border-radius: 4px;
  font-weight: 600;
  text-transform: uppercase;
  position: absolute;
  top: 12px;
  right: 12px;
}

/* Mini Contribution Heatmap */
.mini-heatmap {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  grid-template-rows: repeat(7, 1fr);
  gap: 2px;
  width: 100%;
  height: 60px;
}

.heatmap-square {
  background: var(--bp-border);
  border-radius: 2px;
  transition: background-color 0.2s ease;
}

.heatmap-square.level-1 { background: rgba(79, 195, 247, 0.3); }
.heatmap-square.level-2 { background: rgba(79, 195, 247, 0.5); }  
.heatmap-square.level-3 { background: rgba(79, 195, 247, 0.7); }
.heatmap-square.level-4 { background: rgba(79, 195, 247, 0.9); }

/* Responsive adjustments */
@media (max-width: 768px) {
  .contributor-cards-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }
  
  .contributor-cards-container {
    padding: 16px;
  }
  
  .identity-card-metrics {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
}

/* Loading states */
.contributor-cards-skeleton {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 24px;
}

.identity-card-skeleton {
  background: var(--bp-surface);
  border-radius: 12px;
  padding: 20px;
  animation: pulse 2s ease-in-out infinite;
}

.skeleton-line {
  background: var(--bp-border);
  border-radius: 4px;
  margin-bottom: 12px;
}

.skeleton-line.short { width: 60%; height: 12px; }
.skeleton-line.medium { width: 80%; height: 14px; }
.skeleton-line.long { width: 100%; height: 16px; }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
```

### 9. Error Handling & Edge Cases

#### API Error States
```typescript
interface ContributorError {
  code: 'EMPTY_REPO' | 'INVALID_BRANCH' | 'GIT_ERROR' | 'TIMEOUT' | 'PERMISSION_DENIED';
  message: string;
  details?: any;
}

// Error responses
{
  "error": "No commits found in repository",
  "code": "EMPTY_REPO"
}

{
  "error": "Branch 'feature/new-ui' does not exist",  
  "code": "INVALID_BRANCH"
}

{
  "error": "Repository analysis timed out after 30 seconds",
  "code": "TIMEOUT"
}
```

#### Frontend Error Handling
```typescript
const ContributorErrorState: React.FC<{ error: ContributorError; onRetry: () => void }> = ({ error, onRetry }) => {
  const errorMessages = {
    EMPTY_REPO: "This repository doesn't have any commits yet.",
    INVALID_BRANCH: "The selected branch doesn't exist.",
    GIT_ERROR: "Unable to access repository data.",
    TIMEOUT: "Repository analysis is taking longer than expected.",
    PERMISSION_DENIED: "You don't have permission to view this repository."
  };

  return (
    <div className="contributor-error-state">
      <h3>Unable to load contributors</h3>
      <p>{errorMessages[error.code] || error.message}</p>
      <button onClick={onRetry} className="retry-button">Try Again</button>
    </div>
  );
};
```

### 10. Testing Strategy

#### Unit Tests
```typescript
// Test heatmap generation algorithm
describe('generateContributionHeatmap', () => {
  it('should generate 12 weeks of data', () => {
    const timestamps = [/* test data */];
    const heatmap = generateContributionHeatmap(timestamps);
    expect(heatmap).toHaveLength(12);
  });

  it('should handle empty commit history', () => {
    const heatmap = generateContributionHeatmap([]);
    expect(heatmap.every(week => week.daily_commits.every(count => count === 0))).toBe(true);
  });
});

// Test contributor sorting
describe('sortContributors', () => {
  it('should sort by commit count descending', () => {
    const contributors = [/* test data */];
    const sorted = sortContributors(contributors, 'commits');
    expect(sorted[0].commit_count).toBeGreaterThan(sorted[1].commit_count);
  });
});
```

#### Integration Tests
```typescript
// Test API endpoint
describe('/api/explorer/repos/[address]/[name]/contributors', () => {
  it('should return contributor data for valid repository', async () => {
    const response = await GET(request, { params: { address: '0x123', name: 'repo' } });
    const data = await response.json();
    
    expect(data.contributors).toBeDefined();
    expect(Array.isArray(data.contributors)).toBe(true);
  });

  it('should handle invalid repository', async () => {
    const response = await GET(request, { params: { address: '0x999', name: 'nonexistent' } });
    expect(response.status).toBe(404);
  });
});
```

#### E2E Tests
```typescript
// Test complete user flow
describe('Contributor Identity Cards E2E', () => {
  it('should display contributors and navigate to address page', async () => {
    await page.goto('/explore/0x123/repo');
    await page.click('[data-tab="contributors"]');
    
    await expect(page.locator('.identity-card')).toBeVisible();
    await page.click('.identity-card:first-child');
    
    await expect(page.url()).toContain('/explore/0x456');
  });
});
```

### 11. Deployment & Monitoring

#### Feature Flag Configuration
```typescript
// Environment variable for gradual rollout
export const FEATURES = {
  CONTRIBUTOR_IDENTITY_CARDS: process.env.ENABLE_CONTRIBUTOR_CARDS === 'true'
};

// Component usage
{FEATURES.CONTRIBUTOR_IDENTITY_CARDS && <ContributorIdentityCards />}
```

#### Performance Monitoring
```typescript
// Add metrics collection
const analyticsEvents = {
  CONTRIBUTOR_CARDS_LOADED: 'contributor_cards_loaded',
  CONTRIBUTOR_CARD_CLICKED: 'contributor_card_clicked',
  CONTRIBUTOR_ANALYSIS_SLOW: 'contributor_analysis_slow'
};

// Track performance
const trackContributorAnalysis = (address: string, name: string, durationMs: number) => {
  analytics.track(analyticsEvents.CONTRIBUTOR_CARDS_LOADED, {
    repository: `${address}/${name}`,
    analysis_time_ms: durationMs,
    contributor_count: contributorCount
  });
  
  if (durationMs > 5000) {
    analytics.track(analyticsEvents.CONTRIBUTOR_ANALYSIS_SLOW, {
      repository: `${address}/${name}`,
      analysis_time_ms: durationMs
    });
  }
};
```

### 12. Implementation Checklist

#### Backend Tasks
- [ ] Extend `repobox-server/src/git.rs` with contributor analysis functions
- [ ] Create contributors API endpoint in Next.js
- [ ] Implement caching layer with Redis/in-memory cache
- [ ] Add database queries for alias resolution
- [ ] Write unit tests for git commit parsing
- [ ] Add error handling for edge cases

#### Frontend Tasks  
- [ ] Create `ContributorIdentityCards` container component
- [ ] Create `IdentityCard` individual card component
- [ ] Create `MiniContributionHeatmap` visualization component
- [ ] Integrate contributors tab into repository page
- [ ] Add loading states and skeleton UI
- [ ] Implement responsive grid layout
- [ ] Add sorting and filtering functionality
- [ ] Create error states and retry mechanisms

#### Integration Tasks
- [ ] Update repository stats cards to show contributor count
- [ ] Add CSS styles following existing design system
- [ ] Implement copy-to-clipboard for addresses
- [ ] Add hover states and click affordances
- [ ] Connect identity cards to existing address pages
- [ ] Test branch-specific contributor analysis

#### Testing & QA
- [ ] Write unit tests for heatmap algorithm
- [ ] Create integration tests for API endpoints
- [ ] Add E2E tests for user interactions
- [ ] Test performance with large repositories
- [ ] Verify responsive design on mobile devices
- [ ] Accessibility audit and WCAG compliance

#### Deployment
- [ ] Add feature flag for gradual rollout
- [ ] Set up performance monitoring
- [ ] Create deployment documentation
- [ ] Plan rollback strategy
- [ ] Monitor error rates and performance metrics

## Success Criteria

### Functional Requirements
✅ **Identity Cards Display**: All unique EVM signers shown as visual identity cards
✅ **Essential Information**: Address, alias, commit count, date range displayed clearly  
✅ **Contribution Heatmap**: 12-week activity visualization for each contributor
✅ **Navigation Integration**: Click on cards navigates to address pages
✅ **Repository Integration**: Accessible via Contributors tab in repository explorer

### Performance Requirements  
✅ **Fast Loading**: <2 seconds analysis for repos with <1,000 commits
✅ **Scalable**: <10 seconds analysis for repos with <10,000 commits  
✅ **Responsive UI**: Cards render within 100ms of receiving data
✅ **Efficient Caching**: >80% cache hit ratio for frequently accessed repos

### User Experience Requirements
✅ **Intuitive Design**: Clear visual hierarchy and interaction patterns
✅ **Responsive Layout**: Works seamlessly on desktop, tablet, and mobile
✅ **Accessible**: Follows WCAG accessibility guidelines
✅ **Error Resilience**: Graceful handling of edge cases and failures

This comprehensive implementation specification provides all necessary technical details for developing the Contributor Graph / Identity Cards feature, including complete code examples, database queries, API schemas, and testing strategies.