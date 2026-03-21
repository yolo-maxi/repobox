# Contributor Graph / Identity Cards - Technical Specification

## Overview

Add a "Contributors" page to the repo.box explorer that displays all unique EVM signers for a repository as identity cards. Each card shows the signer's address (with alias if known), commit count, first/last commit dates, and a mini contribution heatmap. Cards link to the address page.

## User Experience

### Navigation
- Add a "Contributors" tab to the repository navigation (alongside README, Files, Commits, Config)
- Show contributor count in the repository stats cards
- Link each contributor card to `/explore/{address}` (existing address page)

### Visual Design
- Grid layout of contributor identity cards (responsive: 1-3 columns based on screen width)
- Each card contains:
  - EVM address with copy button
  - Alias/ENS name if known (future enhancement)
  - Avatar (generated from address hash)
  - Commit count with icon
  - Date range: "First: MM/DD/YY • Last: MM/DD/YY"
  - Mini contribution heatmap (7 days × recent weeks, like GitHub)
  - Clickable area linking to address page

## API Design

### New Endpoint: GET `/api/explorer/repos/[address]/[name]/contributors`

**Query Parameters:**
- `branch` (optional): Target branch, defaults to `HEAD`
- `sort` (optional): Sort order - `commits` (default), `recent`, `alphabetical`
- `limit` (optional): Maximum contributors to return, defaults to 50

**Response Schema:**
```typescript
interface ContributorResponse {
  contributors: Contributor[];
  total_contributors: number;
  analysis_time_ms: number;
  last_computed: string;
  branch: string;
}

interface Contributor {
  address: string;
  alias?: string;
  commit_count: number;
  first_commit_date: string; // ISO string
  last_commit_date: string;  // ISO string
  heatmap_data: HeatmapWeek[];
  avatar_seed: string; // For deterministic avatar generation
}

interface HeatmapWeek {
  week_start: string; // ISO date of Monday
  daily_commits: number[]; // Array of 7 numbers (Mon-Sun)
}
```

**Implementation Location:** 
`/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/contributors/route.ts`

## Backend Implementation

### Core Algorithm

The implementation leverages existing git commit parsing infrastructure in `repobox-server/src/git.rs`:

1. **Extract All Signed Commits:**
   - Use `git rev-list --all --format=raw` to get all commit objects
   - Parse each commit using existing `extract_signer_from_commit_text()` function
   - Filter to commits with valid EVM signatures (65-byte gpgsig)

2. **Aggregate by Signer:**
   - Group commits by recovered EVM address
   - Calculate first/last commit timestamps
   - Count total commits per address

3. **Generate Heatmap Data:**
   - For each contributor, generate last 12 weeks of commit activity
   - Group commits by week (Monday-Sunday) and day
   - Format as array of `HeatmapWeek` objects

### Git Integration Functions

Extend `git.rs` with new functions:

```rust
// In repobox-server/src/git.rs
pub(crate) fn get_all_contributors(
    data_dir: &Path, 
    repo: &RepoPath, 
    branch: Option<&str>
) -> std::io::Result<Vec<ContributorData>> {
    // Implementation details below
}

pub(crate) struct ContributorData {
    pub address: String,
    pub commit_count: u32,
    pub first_commit_timestamp: i64,
    pub last_commit_timestamp: i64,
    pub commit_timestamps: Vec<i64>, // For heatmap generation
}
```

**Algorithm Details:**
1. Execute `git rev-list --all --format=raw [branch]` to get all commit objects
2. Parse each commit object using existing `extract_signer_from_commit_text()`
3. Build HashMap<Address, Vec<CommitTimestamp>>
4. Convert to ContributorData structs with aggregated statistics

### Database Integration

**No schema changes required.** The feature uses:
- Existing `push_log` table for fallback data
- Direct git repository analysis for primary data source
- In-memory processing for real-time contributor analysis

## Frontend Implementation

### New Components

#### 1. ContributorCards Component
**Location:** `/home/xiko/repobox/web/src/components/ContributorCards.tsx`

```typescript
interface ContributorCardsProps {
  address: string;
  name: string;
  branch?: string;
}

export default function ContributorCards({ address, name, branch }: ContributorCardsProps) {
  // Fetch contributors data
  // Render grid of ContributorCard components
}
```

#### 2. ContributorCard Component
**Location:** `/home/xiko/repobox/web/src/components/ContributorCard.tsx`

```typescript
interface ContributorCardProps {
  contributor: Contributor;
  onClick?: () => void;
}

export default function ContributorCard({ contributor, onClick }: ContributorCardProps) {
  // Render individual contributor card
  // Generate avatar from address hash
  // Format dates and commit count
  // Render mini heatmap
}
```

#### 3. ContributorHeatmap Component
**Location:** `/home/xiko/repobox/web/src/components/ContributorHeatmap.tsx`

```typescript
interface ContributorHeatmapProps {
  heatmapData: HeatmapWeek[];
  compact?: boolean; // For mini version in cards
}

export default function ContributorHeatmap({ heatmapData, compact }: ContributorHeatmapProps) {
  // Render SVG heatmap grid
  // Color squares based on commit intensity
  // Tooltip on hover showing exact counts
}
```

### Tab Integration

**Modify:** `/home/xiko/repobox/web/src/app/explore/[address]/[name]/page.tsx`

Add "Contributors" to the tabs array and implement the tab content:

```typescript
// Add to existing tabs
const tabs = ['readme', 'files', 'commits', 'contributors', 'config'];

// Add tab content rendering
{activeTab === 'contributors' && (
  <div className="explore-contributors">
    <ContributorCards address={address} name={name} branch={selectedBranch} />
  </div>
)}
```

### Avatar Generation

Use deterministic avatar generation based on address hash:

```typescript
// In /home/xiko/repobox/web/src/lib/utils.ts
export function generateAvatar(address: string): string {
  // Create deterministic avatar using address as seed
  // Options: identicon, geometric patterns, or simple color blocks
  // Return data URL or CSS background
}
```

## Data Structures & Algorithms

### Commit Timeline Processing

**Algorithm for Heatmap Generation:**

1. **Time Bucketing:**
   ```typescript
   function generateHeatmapData(commitTimestamps: number[]): HeatmapWeek[] {
     const weeks: Map<string, number[]> = new Map();
     const now = new Date();
     const twelveWeeksAgo = new Date(now.getTime() - (12 * 7 * 24 * 60 * 60 * 1000));
     
     commitTimestamps.forEach(timestamp => {
       if (timestamp >= twelveWeeksAgo.getTime() / 1000) {
         const weekStart = getWeekStart(new Date(timestamp * 1000));
         const dayOfWeek = getDayOfWeek(new Date(timestamp * 1000)); // 0=Mon, 6=Sun
         
         if (!weeks.has(weekStart)) {
           weeks.set(weekStart, [0, 0, 0, 0, 0, 0, 0]);
         }
         weeks.get(weekStart)![dayOfWeek]++;
       }
     });
     
     return Array.from(weeks.entries())
       .map(([weekStart, dailyCommits]) => ({ week_start: weekStart, daily_commits: dailyCommits }))
       .sort((a, b) => new Date(a.week_start).getTime() - new Date(b.week_start).getTime());
   }
   ```

2. **Performance Optimization:**
   - Cache contributor data for 5 minutes using in-memory cache
   - Implement pagination if contributor count > 100
   - Use worker threads for large repositories (>10,000 commits)

### Sorting Algorithms

**Sort Options:**
- `commits`: Sort by total commit count (descending)
- `recent`: Sort by last commit date (most recent first)
- `alphabetical`: Sort by address (ascending)

```typescript
function sortContributors(contributors: Contributor[], sortBy: SortOption): Contributor[] {
  switch (sortBy) {
    case 'commits':
      return contributors.sort((a, b) => b.commit_count - a.commit_count);
    case 'recent':
      return contributors.sort((a, b) => 
        new Date(b.last_commit_date).getTime() - new Date(a.last_commit_date).getTime()
      );
    case 'alphabetical':
      return contributors.sort((a, b) => a.address.localeCompare(b.address));
    default:
      return contributors;
  }
}
```

## Integration Points

### 1. Repository Stats Cards
**File:** `/home/xiko/repobox/web/src/components/RepoStatsCards.tsx`

Modify the stats cards to include contributor count:

```typescript
<StatCard
  title="Contributors"
  value={stats.unique_signers.toString()}
  label="SIGNERS"
  icon="👥"
/>
```

The `unique_signers` field is already computed in the existing stats API.

### 2. Address Page Enhancement
**File:** `/home/xiko/repobox/web/src/app/explore/[address]/page.tsx`

No changes required - the existing address page will automatically show all repositories for a contributor when users click on contributor cards.

### 3. Navigation Updates
**File:** `/home/xiko/repobox/web/src/app/explore/[address]/[name]/page.tsx`

Add the Contributors tab to the existing navigation system.

## Database Schema

**No database schema changes required.**

The feature uses:
- Real-time git repository analysis for primary data
- Existing `push_log` table for performance hints (optional fallback)
- In-memory caching for frequently accessed repositories

## Performance Considerations

### Caching Strategy

1. **In-Memory Cache:**
   ```typescript
   // Cache key: `contributors:${address}:${name}:${branch}`
   // TTL: 5 minutes
   // Invalidation: On new pushes to repository
   ```

2. **Background Processing:**
   - For repositories with >5,000 commits, queue analysis for background processing
   - Return cached data immediately if available
   - Stream updates via WebSocket (future enhancement)

### Optimization Techniques

1. **Lazy Loading:**
   - Load first 20 contributors immediately
   - Implement "Load More" button for repositories with many contributors
   - Virtualized scrolling for extremely large contributor lists

2. **Commit Parsing Optimization:**
   - Parse commits in batches of 1000
   - Stop parsing if commit is older than 2 years (configurable)
   - Use git's `--since` flag to limit parsing scope

3. **Heatmap Calculation:**
   - Only calculate heatmaps for last 12 weeks
   - Pre-aggregate daily commit counts during parsing
   - Use efficient date arithmetic to avoid Date object creation

## Error Handling

### API Error Responses

```typescript
// Empty repository
{ error: "No commits found in repository", code: "EMPTY_REPO" }

// Invalid branch
{ error: "Branch 'invalid-branch' does not exist", code: "INVALID_BRANCH" }

// Processing timeout
{ error: "Repository analysis timed out", code: "TIMEOUT" }

// Git operation failure
{ error: "Unable to access repository", code: "GIT_ERROR" }
```

### Frontend Error States

1. **Loading State:** Skeleton cards while fetching data
2. **Empty State:** "No contributors found" with helpful message
3. **Error State:** "Unable to load contributors" with retry button
4. **Partial Error:** Show available data with warning banner

## Security Considerations

### Input Validation

1. **Branch Name Sanitization:**
   - Use existing `sanitizeBranchName()` function from `git.ts`
   - Reject branches with path traversal attempts (`../`, etc.)

2. **Address Validation:**
   - Validate EVM address format (0x + 40 hex characters)
   - Use existing `validate_address()` function

3. **Rate Limiting:**
   - Implement per-IP rate limiting on contributor API endpoint
   - Cache results to reduce git operations

### Data Privacy

- **No PII Storage:** Only EVM addresses are processed and stored
- **Public Data Only:** All git commit data is already public
- **Address Anonymization:** Option to display truncated addresses

## Testing Strategy

### Unit Tests

1. **Commit Parsing Tests:**
   ```typescript
   // Test EVM signature extraction from commit objects
   // Test handling of unsigned commits
   // Test malformed git objects
   ```

2. **Heatmap Algorithm Tests:**
   ```typescript
   // Test date bucketing logic
   // Test edge cases (timezone handling, leap years)
   // Test empty date ranges
   ```

3. **API Response Tests:**
   ```typescript
   // Test various repository configurations
   // Test error conditions
   // Test caching behavior
   ```

### Integration Tests

1. **End-to-End Repository Analysis:**
   ```bash
   # Create test repository with multiple signers
   # Verify contributor extraction accuracy
   # Test branch-specific analysis
   ```

2. **Frontend Component Tests:**
   ```typescript
   // Test contributor card rendering
   // Test heatmap visualization
   // Test sorting and filtering
   ```

### Performance Tests

1. **Large Repository Handling:**
   - Test with repositories containing 10,000+ commits
   - Measure memory usage during analysis
   - Verify timeout handling

2. **Concurrent Request Handling:**
   - Simulate multiple users accessing contributor data
   - Test cache effectiveness
   - Measure response time distribution

## Deployment Considerations

### Feature Flags

Implement feature toggle for gradual rollout:

```typescript
// In environment configuration
ENABLE_CONTRIBUTORS_TAB=true

// In component
const showContributorsTab = process.env.ENABLE_CONTRIBUTORS_TAB === 'true';
```

### Performance Monitoring

Add metrics collection for:
- Contributor analysis time per repository size
- Cache hit/miss ratios
- API response times
- Error rates

### Rollback Strategy

- Feature can be disabled via environment variable
- No database migrations to reverse
- Cache can be safely cleared without data loss

## Future Enhancements

### Phase 2 Features (Beyond MVP)

1. **ENS Integration:** Resolve contributor addresses to ENS names
2. **Advanced Heatmaps:** Click to view commit details for specific days
3. **Contribution Trends:** Line charts showing contributor activity over time
4. **Team Analytics:** Group contributors by organization or project role
5. **Real-time Updates:** WebSocket updates when new commits are pushed

### Extension Points

The implementation provides clean extension points for:
- Additional sorting options
- Custom avatar providers
- Alternative heatmap visualizations
- Integration with external identity services

## Implementation Timeline

**Estimated Development Time: 2-3 days**

### Day 1: Backend Implementation
- [ ] Add contributor extraction functions to `git.rs`
- [ ] Implement contributors API endpoint
- [ ] Add caching layer
- [ ] Write unit tests for git parsing logic

### Day 2: Frontend Components
- [ ] Create ContributorCard component
- [ ] Create ContributorHeatmap component
- [ ] Implement ContributorCards container
- [ ] Add Contributors tab to repository page

### Day 3: Integration & Polish
- [ ] Integrate with existing repository stats
- [ ] Add loading states and error handling
- [ ] Implement responsive design
- [ ] Write integration tests
- [ ] Performance optimization and testing

## Success Metrics

### Functional Requirements Verification
- ✅ Display all unique EVM signers as identity cards
- ✅ Show commit count, first/last commit dates
- ✅ Mini contribution heatmap for each contributor
- ✅ Link each card to the address page
- ✅ Branch-specific contributor analysis

### Performance Targets
- Analysis completion in <2 seconds for repositories with <1,000 commits
- Analysis completion in <10 seconds for repositories with <10,000 commits
- UI renders within 100ms of receiving API response
- Cache hit ratio >80% for frequently accessed repositories

### User Experience Goals
- Intuitive navigation from repository to contributors to individual profiles
- Clear visual hierarchy in contributor cards
- Responsive design working on mobile devices
- Accessible design following WCAG guidelines

This comprehensive specification provides everything needed for a development sub-agent to implement the Contributors feature without requiring clarification on architecture, patterns, or technical details.