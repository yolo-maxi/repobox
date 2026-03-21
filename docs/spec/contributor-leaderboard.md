# Contributor Leaderboard + Charts Feature Specification

## Overview

This specification defines the implementation of a comprehensive contributor leaderboard and analytics system for repo.box repositories. The feature will provide detailed per-signer statistics, visual charts, and a heatmap showing contribution patterns over time.

## Current State Analysis

### Existing Implementation
- Basic contributor API endpoint exists at `/api/explorer/repos/[address]/[name]/contributors`
- Returns push-based statistics: `pushCount`, `lastPush`, `isOwner`
- Contributors tab exists in repo detail page with basic grid layout
- Links to individual contributor profiles

### Limitations of Current Implementation
- Only tracks push events, not actual code contribution metrics
- No visualization of contribution patterns
- Missing detailed statistics like lines added/removed, files touched
- No time-based analysis or heatmaps
- No comparison or ranking beyond push count

## Technical Requirements

### 1. Enhanced API Endpoint

#### Endpoint: `GET /api/explorer/repos/[address]/[name]/contributors`

**Query Parameters:**
- `sort` (optional): `lines`, `commits`, `recency`, `files` (default: `lines`)
- `period` (optional): `week`, `month`, `quarter`, `year`, `all` (default: `all`)
- `branch` (optional): specific branch filter (default: all branches)

**Enhanced Response Format:**
```typescript
interface ContributorStats {
  address: string;
  isOwner: boolean;
  metrics: {
    commits: number;
    linesAdded: number;
    linesRemoved: number;
    linesNet: number;  // added - removed
    filesModified: number;
    filesCreated: number;
    filesDeleted: number;
    firstCommit: string;  // ISO timestamp
    lastCommit: string;   // ISO timestamp
    activeDays: number;   // unique days with commits
  };
  timeline: Array<{
    date: string;        // YYYY-MM-DD
    commits: number;
    linesAdded: number;
    linesRemoved: number;
    filesModified: number;
  }>;
  languageBreakdown: Array<{
    language: string;
    linesAdded: number;
    linesRemoved: number;
    filesModified: number;
  }>;
  // Legacy fields for backward compatibility
  pushCount: number;
  lastPush: string;
}

interface ContributorsResponse {
  contributors: ContributorStats[];
  repositoryTotals: {
    totalCommits: number;
    totalLines: number;
    totalFiles: number;
    activePeriod: string;  // "2023-01-15 to 2024-03-21"
  };
  periodSummary?: {
    period: string;
    contributorCount: number;
    totalActivity: number;
  };
}
```

### 2. Database Schema Extensions

#### New Table: `contributor_stats`
```sql
CREATE TABLE contributor_stats (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_address TEXT NOT NULL,
  repo_name TEXT NOT NULL,
  contributor_address TEXT NOT NULL,
  commit_hash TEXT NOT NULL,
  commit_date DATE NOT NULL,
  lines_added INTEGER NOT NULL DEFAULT 0,
  lines_removed INTEGER NOT NULL DEFAULT 0,
  files_modified INTEGER NOT NULL DEFAULT 0,
  files_created INTEGER NOT NULL DEFAULT 0,
  files_deleted INTEGER NOT NULL DEFAULT 0,
  file_extensions TEXT,  -- JSON array of extensions touched
  created_at TEXT NOT NULL,
  
  UNIQUE(repo_address, repo_name, commit_hash, contributor_address)
);

CREATE INDEX idx_contributor_stats_repo ON contributor_stats(repo_address, repo_name);
CREATE INDEX idx_contributor_stats_contributor ON contributor_stats(contributor_address);
CREATE INDEX idx_contributor_stats_date ON contributor_stats(commit_date);
```

#### New Table: `contributor_file_changes`
```sql
CREATE TABLE contributor_file_changes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_address TEXT NOT NULL,
  repo_name TEXT NOT NULL,
  contributor_address TEXT NOT NULL,
  commit_hash TEXT NOT NULL,
  file_path TEXT NOT NULL,
  change_type TEXT NOT NULL,  -- 'modified', 'created', 'deleted', 'renamed'
  lines_added INTEGER NOT NULL DEFAULT 0,
  lines_removed INTEGER NOT NULL DEFAULT 0,
  language TEXT,  -- detected from file extension
  
  FOREIGN KEY(repo_address, repo_name, commit_hash, contributor_address) 
    REFERENCES contributor_stats(repo_address, repo_name, commit_hash, contributor_address)
);

CREATE INDEX idx_file_changes_repo ON contributor_file_changes(repo_address, repo_name);
```

### 3. Git Log Analysis Implementation

#### Core Analysis Function

```typescript
// New function in /lib/git.ts
export interface CommitAnalysis {
  hash: string;
  author_address: string;  // extracted from REPOBOX SIGNATURE
  timestamp: number;
  date: string;  // YYYY-MM-DD
  fileChanges: Array<{
    path: string;
    status: 'added' | 'modified' | 'deleted' | 'renamed';
    oldPath?: string;
    additions: number;
    deletions: number;
    language: string;
  }>;
  totals: {
    linesAdded: number;
    linesRemoved: number;
    filesModified: number;
    filesCreated: number;
    filesDeleted: number;
  };
}

export function analyzeRepositoryContributions(
  address: string, 
  name: string, 
  branch: string = 'HEAD'
): CommitAnalysis[] {
  const repoPath = getRepoPath(address, name);
  
  try {
    // Get all commits with numstat
    const output = gitCommand(
      repoPath, 
      `log --numstat --format="COMMIT:%H|%at|%s" ${branch}`
    );
    
    const commits: CommitAnalysis[] = [];
    let currentCommit: Partial<CommitAnalysis> | null = null;
    
    output.split('\n').forEach(line => {
      if (line.startsWith('COMMIT:')) {
        // Save previous commit if exists
        if (currentCommit && currentCommit.hash) {
          commits.push(currentCommit as CommitAnalysis);
        }
        
        // Parse new commit
        const [hash, timestamp, message] = line.substring(7).split('|');
        const commitDate = new Date(parseInt(timestamp) * 1000);
        
        // Extract signer address from commit message or signature
        const signerAddress = extractSignerAddress(address, name, hash);
        
        currentCommit = {
          hash,
          author_address: signerAddress,
          timestamp: parseInt(timestamp),
          date: commitDate.toISOString().split('T')[0],
          fileChanges: [],
          totals: {
            linesAdded: 0,
            linesRemoved: 0,
            filesModified: 0,
            filesCreated: 0,
            filesDeleted: 0
          }
        };
      } else if (line.trim() && currentCommit) {
        // Parse numstat line: "additions deletions filename"
        const match = line.match(/^(\d+|-)\s+(\d+|-)\s+(.+)$/);
        if (match) {
          const [, additionsStr, deletionsStr, filePath] = match;
          const additions = additionsStr === '-' ? 0 : parseInt(additionsStr);
          const deletions = deletionsStr === '-' ? 0 : parseInt(deletionsStr);
          
          // Determine change type and language
          const changeType = determineChangeType(additions, deletions, filePath);
          const language = detectLanguageFromExtension(filePath);
          
          const fileChange = {
            path: filePath,
            status: changeType,
            additions,
            deletions,
            language
          };
          
          currentCommit.fileChanges!.push(fileChange);
          
          // Update totals
          currentCommit.totals!.linesAdded += additions;
          currentCommit.totals!.linesRemoved += deletions;
          
          if (changeType === 'added') currentCommit.totals!.filesCreated++;
          else if (changeType === 'deleted') currentCommit.totals!.filesDeleted++;
          else currentCommit.totals!.filesModified++;
        }
      }
    });
    
    // Don't forget the last commit
    if (currentCommit && currentCommit.hash) {
      commits.push(currentCommit as CommitAnalysis);
    }
    
    return commits;
  } catch (error) {
    console.error('Error analyzing repository contributions:', error);
    return [];
  }
}

function extractSignerAddress(address: string, name: string, hash: string): string {
  // Extract EVM address from REPOBOX SIGNATURE block
  // This logic already exists in the codebase - reuse it
  try {
    const repoPath = getRepoPath(address, name);
    const commitContent = gitCommand(repoPath, `show --format=full ${hash}`);
    
    // Extract signature and recover address
    // Use existing signature verification logic
    return recoverAddressFromCommit(commitContent);
  } catch {
    return 'unknown';
  }
}

function determineChangeType(additions: number, deletions: number, filePath: string): 'added' | 'modified' | 'deleted' {
  if (deletions === 0) return 'added';
  if (additions === 0) return 'deleted';
  return 'modified';
}

function detectLanguageFromExtension(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase();
  const languageMap: Record<string, string> = {
    'js': 'JavaScript',
    'ts': 'TypeScript',
    'jsx': 'React',
    'tsx': 'React',
    'py': 'Python',
    'rs': 'Rust',
    'go': 'Go',
    'java': 'Java',
    'cpp': 'C++',
    'c': 'C',
    'php': 'PHP',
    'rb': 'Ruby',
    'md': 'Markdown',
    'html': 'HTML',
    'css': 'CSS',
    'scss': 'SCSS',
    'sql': 'SQL',
    'sh': 'Shell',
    'yml': 'YAML',
    'yaml': 'YAML',
    'json': 'JSON',
    'toml': 'TOML',
    'xml': 'XML'
  };
  
  return languageMap[ext || ''] || 'Other';
}
```

### 4. Data Population Strategy

#### Background Sync Process

```typescript
// New function for analyzing and storing contribution data
export async function syncContributorStats(address: string, name: string): Promise<void> {
  try {
    // Check if sync is needed (compare last sync timestamp)
    const lastSync = await runQueryOne(
      'SELECT MAX(created_at) as last_sync FROM contributor_stats WHERE repo_address = ? AND repo_name = ?',
      [address, name]
    );
    
    const commitAnalyses = analyzeRepositoryContributions(address, name);
    
    for (const analysis of commitAnalyses) {
      if (!analysis.author_address || analysis.author_address === 'unknown') continue;
      
      // Insert contributor stats
      await runQuery(`
        INSERT OR REPLACE INTO contributor_stats (
          repo_address, repo_name, contributor_address, commit_hash, commit_date,
          lines_added, lines_removed, files_modified, files_created, files_deleted,
          file_extensions, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `, [
        address, name, analysis.author_address, analysis.hash, analysis.date,
        analysis.totals.linesAdded, analysis.totals.linesRemoved,
        analysis.totals.filesModified, analysis.totals.filesCreated, analysis.totals.filesDeleted,
        JSON.stringify(analysis.fileChanges.map(f => f.language)),
        new Date().toISOString()
      ]);
      
      // Insert file changes
      for (const fileChange of analysis.fileChanges) {
        await runQuery(`
          INSERT OR REPLACE INTO contributor_file_changes (
            repo_address, repo_name, contributor_address, commit_hash,
            file_path, change_type, lines_added, lines_removed, language
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        `, [
          address, name, analysis.author_address, analysis.hash,
          fileChange.path, fileChange.status, fileChange.additions, fileChange.deletions,
          fileChange.language
        ]);
      }
    }
  } catch (error) {
    console.error('Error syncing contributor stats:', error);
    throw error;
  }
}
```

### 5. Enhanced UI Components

#### Contributors Tab Layout

The enhanced Contributors tab will replace the current simple grid with a comprehensive dashboard:

```
┌─────────────────────────────────────────────────────────┐
│  🏆 Contributor Leaderboard                             │
├─────────────────────────────────────────────────────────┤
│  Sort: [Lines Written ▼] Period: [All Time ▼] Branch: [main ▼] │
├─────────────────────────────────────────────────────────┤
│  ┌─ Top Contributor ──────────────────────────────────┐  │
│  │ 👑 0xDbbA...2048 (owner)                          │  │
│  │ 2,547 lines • 89 commits • 15 files • 23 days     │  │
│  │ ████████████████████████████████████ 64%          │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌─ Contributor Rankings ─────────────────────────────┐  │
│  │ 2. 0xAAc0...4a00   █████████████░░░ 31%  1,203 ℓ  │  │
│  │ 3. 0x9aBA...34b    ████░░░░░░░░░░░  9%    427 ℓ   │  │
│  │ 4. 0x1234...5678   ██░░░░░░░░░░░░   5%    234 ℓ   │  │
│  └────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  📊 Contribution Timeline                               │
│  ┌─ 90 Day Activity Heatmap ─────────────────────────┐  │
│  │ Mar  Apr  May  Jun  Jul  Aug  Sep  Oct  Nov  Dec  │  │
│  │ ▓▓▒░ ░▒▓▓ ▒░▒▓ ░▓▓▒ ▒▒▓▓ ░░▒▓ ▓▒▒░ ▓▓▒▒ ▒▓▓▒ ░▒▓░ │  │
│  │ ▒▒▓▓ ▓▒░▓ ▒▒▓▒ ▓░▒▒ ▓▒▓▓ ▒▓▒▓ ▒▒▓▓ ▒▓▒░ ▓▓▒▓ ▒▓▒▒ │  │
│  │ ░▒▓▒ ▒▓▒▓ ▓▒▒▓ ▒▓▒▓ ▒▒▓▒ ▓▓▒▒ ░▒▓▓ ▓▒▓▓ ▒▒▓▒ ▓▒▒▓ │  │
│  └─ Legend: ░ 0-2  ▒ 3-7  ▓ 8+ commits per day ──────┘  │
├─────────────────────────────────────────────────────────┤
│  📈 Language Breakdown (by lines contributed)           │
│  ┌────────────────────────────────────────────────────┐  │
│  │ TypeScript  ████████████████████████████████ 67%   │  │
│  │ Rust        █████████████████████░░░░░░░░░░ 28%    │  │
│  │ Markdown    ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 4%     │  │
│  │ CSS         █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 1%     │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

#### Component Implementation

**ContributorLeaderboard.tsx:**
```typescript
interface ContributorLeaderboardProps {
  address: string;
  name: string;
  branch: string;
}

export default function ContributorLeaderboard({ address, name, branch }: ContributorLeaderboardProps) {
  const [contributors, setContributors] = useState<ContributorStats[]>([]);
  const [sortBy, setSortBy] = useState<'lines' | 'commits' | 'recency' | 'files'>('lines');
  const [period, setPeriod] = useState<'week' | 'month' | 'quarter' | 'year' | 'all'>('all');
  const [loading, setLoading] = useState(true);
  
  // Component implementation...
}
```

**ContributionHeatmap.tsx:**
```typescript
interface ContributionHeatmapProps {
  contributors: ContributorStats[];
  selectedContributor?: string;
}

export default function ContributionHeatmap({ contributors, selectedContributor }: ContributionHeatmapProps) {
  // Generate GitHub-style contribution heatmap
  // Show activity intensity by day over last 12 months
  // Support filtering to specific contributor
}
```

**LanguageChart.tsx:**
```typescript
interface LanguageChartProps {
  contributors: ContributorStats[];
}

export default function LanguageChart({ contributors }: LanguageChartProps) {
  // Horizontal bar chart showing language breakdown
  // Aggregate data across all contributors
  // Color-coded bars with percentages
}
```

### 6. Implementation Plan

#### Phase 1: Backend Infrastructure (Week 1)
1. **Database Schema Setup**
   - Create new tables: `contributor_stats`, `contributor_file_changes`
   - Add migration script for existing repositories
   - Update database utility functions

2. **Git Analysis Engine**
   - Implement `analyzeRepositoryContributions()` function
   - Add signer address extraction from commits
   - Create language detection utilities
   - Add file change classification logic

3. **Data Sync Process**
   - Implement `syncContributorStats()` function
   - Create background sync triggered on push events
   - Add incremental sync for performance
   - Add data validation and error handling

#### Phase 2: API Enhancement (Week 1-2)
1. **Enhanced Contributors API**
   - Extend existing `/api/explorer/repos/[address]/[name]/contributors` endpoint
   - Add query parameters for sorting and filtering
   - Implement aggregation queries for statistics
   - Add timeline data generation
   - Maintain backward compatibility with existing response format

2. **Performance Optimization**
   - Add database indexes for fast queries
   - Implement response caching (Redis or memory)
   - Add query pagination for large repositories
   - Optimize database queries with proper joins

#### Phase 3: Frontend Components (Week 2)
1. **Contributor Leaderboard Component**
   - Replace existing basic grid layout
   - Add sorting and filtering controls
   - Implement rank visualization with progress bars
   - Add crown icon for top contributor
   - Style with consistent design system

2. **Contribution Timeline/Heatmap**
   - Create GitHub-style activity heatmap
   - Use canvas or SVG for rendering
   - Add hover states with daily details
   - Support date range selection
   - Color intensity based on activity level

3. **Language Breakdown Chart**
   - Horizontal bar chart with percentage labels
   - Color-coded by language
   - Tooltip with detailed statistics
   - Responsive design for mobile

#### Phase 4: Integration and Polish (Week 2-3)
1. **Tab Integration**
   - Update repo detail page to use new components
   - Ensure smooth transitions between tabs
   - Add loading states and error handling
   - Test responsive behavior

2. **Data Population**
   - Run initial sync for all existing repositories
   - Add sync status indicators in admin interface
   - Handle edge cases (empty repos, corrupted history)
   - Monitor sync performance and add throttling if needed

3. **Testing and Optimization**
   - Test with repositories of various sizes
   - Optimize for repositories with 1000+ commits
   - Add error boundaries and fallback UI
   - Verify mobile responsiveness

### 7. Security Considerations

1. **SQL Injection Prevention**
   - Use parameterized queries for all database operations
   - Validate input parameters (sort, period, branch)
   - Sanitize branch names before git operations

2. **Resource Protection**
   - Rate limit API endpoints to prevent abuse
   - Implement query timeouts for large repositories
   - Cache expensive operations
   - Monitor memory usage during large syncs

3. **Data Integrity**
   - Validate git log output before processing
   - Handle corrupted commit signatures gracefully
   - Implement data consistency checks
   - Add rollback capability for failed syncs

### 8. Performance Considerations

1. **Database Optimization**
   - Index on common query patterns
   - Use aggregate tables for frequently accessed statistics
   - Implement proper query pagination
   - Consider read replicas for heavy read workloads

2. **Sync Performance**
   - Process commits in batches to avoid memory issues
   - Skip already-processed commits (incremental sync)
   - Use background workers for large repositories
   - Implement sync prioritization (recent activity first)

3. **Frontend Performance**
   - Lazy load chart components
   - Virtualize large contributor lists
   - Debounce filter changes
   - Use React.memo for expensive components

### 9. Testing Strategy

1. **Unit Tests**
   - Test git analysis functions with sample repositories
   - Verify database operations with mock data
   - Test component rendering with various data states
   - Validate sorting and filtering logic

2. **Integration Tests**
   - Test full sync process with real git repositories
   - Verify API endpoint responses with different parameters
   - Test UI components with live data
   - Check performance with large datasets

3. **End-to-End Tests**
   - Test complete user flows from repository view to contributor details
   - Verify responsive behavior on different screen sizes
   - Test error handling with malformed repositories
   - Validate accessibility compliance

### 10. Monitoring and Maintenance

1. **Metrics Collection**
   - Track API response times
   - Monitor sync success/failure rates
   - Measure database query performance
   - Log user interaction patterns

2. **Operational Health**
   - Set up alerts for sync failures
   - Monitor database growth and cleanup old data
   - Track memory usage during large operations
   - Implement automated health checks

3. **Future Enhancements**
   - Add contributor profile pages
   - Implement team/organization views
   - Add contribution streaks and achievements
   - Consider machine learning for contribution insights

---

## Conclusion

This comprehensive specification provides a roadmap for implementing a sophisticated contributor leaderboard and analytics system for repo.box. The implementation balances feature richness with performance considerations, ensuring the system can scale to handle repositories of varying sizes while providing meaningful insights into contributor activity.

The phased approach allows for incremental delivery and testing, while the detailed technical specifications ensure consistent implementation across the entire feature set.