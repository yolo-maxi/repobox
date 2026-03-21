# Repo Stats Cards Technical Specification

**Feature**: Repository Statistics Cards  
**Version**: 1.0  
**Author**: PM Agent  
**Date**: 2026-03-21  

## Overview

Add repository statistics visualization to repo detail pages, displaying:
- **Language breakdown bar** (similar to GitHub's language stats)
- **Total lines of code**
- **Number of unique signers** (contributors with EVM signatures)
- **Repository age** (days since first commit)

Statistics are computed server-side using `git log` analysis and file extension detection, then displayed as colored progress bars and statistical cards on the repo detail page.

## User Experience

### Visual Layout

The stats cards will be positioned on the repo detail page between the repository header and the tabs section. The layout consists of:

1. **Language Bar**: Horizontal colored bar showing relative percentages of each language
2. **Stats Grid**: 2x2 grid of cards showing key metrics

```
┌─ Repository Header ──────────────────────────────────────────┐
│ repo-name                                    123 COMMITS     │
│ 0x1234...abcd                               main BRANCH      │
│ git clone https://...                                        │
└──────────────────────────────────────────────────────────────┘
┌─ Language Breakdown ─────────────────────────────────────────┐
│ TypeScript ████████ JavaScript ██ Python █ Other █          │
│ 62.3%      31.7%              4.2%      1.8%                │
└──────────────────────────────────────────────────────────────┘
┌─ Statistics Cards ───────────────────────────────────────────┐
│ ┌─ Total Lines ─┐  ┌─ Contributors ─┐                       │
│ │    12,847     │  │       4        │                       │
│ │     SLOC      │  │   SIGNERS      │                       │
│ └───────────────┘  └────────────────┘                       │
│ ┌─ Repository ──┐  ┌─ Latest Lang ──┐                       │
│ │    127 days   │  │  TypeScript    │                       │
│ │      AGE      │  │  RECENT LANG   │                       │
│ └───────────────┘  └────────────────┘                       │
└──────────────────────────────────────────────────────────────┘
┌─ Tabs (README | Files | Commits | Config) ──────────────────┐
```

### Responsive Behavior

- **Desktop** (>768px): 2x2 grid layout
- **Mobile** (≤768px): Single column stack
- Language bar remains horizontal on all screen sizes
- Cards stack vertically on mobile

### Loading States

- Show skeleton placeholders while computing statistics
- Computing stats should be under 2 seconds for repos with <10k files
- Display "Computing..." indicator with progress for large repositories

## API Design

### New Endpoints

#### 1. Repository Statistics Endpoint

```
GET /api/explorer/repos/[address]/[name]/stats?branch=[branch]
```

**Query Parameters:**
- `branch` (optional): Branch to analyze. Defaults to HEAD/default branch

**Response Format:**
```typescript
interface RepoStats {
  language_breakdown: LanguageStats[];
  total_lines: number;
  total_files: number;
  unique_signers: number;
  repository_age_days: number;
  last_computed: string; // ISO timestamp
  computation_time_ms: number;
  branch: string;
}

interface LanguageStats {
  name: string;
  lines: number;
  files: number;
  percentage: number;
  color: string; // Hex color for UI
  extensions: string[]; // File extensions included
}
```

**Example Response:**
```json
{
  "language_breakdown": [
    {
      "name": "TypeScript",
      "lines": 8547,
      "files": 34,
      "percentage": 62.3,
      "color": "#3178c6",
      "extensions": [".ts", ".tsx"]
    },
    {
      "name": "JavaScript", 
      "lines": 4352,
      "files": 12,
      "percentage": 31.7,
      "color": "#f1e05a",
      "extensions": [".js", ".jsx"]
    },
    {
      "name": "Python",
      "lines": 578,
      "files": 3,
      "percentage": 4.2,
      "color": "#3572A5",
      "extensions": [".py"]
    },
    {
      "name": "Other",
      "lines": 245,
      "files": 8,
      "percentage": 1.8,
      "color": "#cccccc",
      "extensions": [".md", ".json", ".yml"]
    }
  ],
  "total_lines": 13722,
  "total_files": 57,
  "unique_signers": 4,
  "repository_age_days": 127,
  "last_computed": "2026-03-21T14:30:45Z",
  "computation_time_ms": 1247,
  "branch": "main"
}
```

**Error Responses:**
- `404`: Repository not found
- `400`: Invalid branch name
- `500`: Computation failed or timeout

#### 2. Language Colors Configuration

Use GitHub's language color scheme for consistency. Colors will be hardcoded in the backend with a mapping like:

```rust
// In repobox-server/src/languages.rs
static LANGUAGE_COLORS: &[(&str, &str)] = &[
    ("TypeScript", "#3178c6"),
    ("JavaScript", "#f1e05a"), 
    ("Python", "#3572A5"),
    ("Rust", "#dea584"),
    ("Go", "#00ADD8"),
    ("Java", "#b07219"),
    ("C++", "#f34b7d"),
    ("C", "#555555"),
    ("HTML", "#e34c26"),
    ("CSS", "#563d7c"),
    ("Shell", "#89e051"),
    ("Markdown", "#083fa1"),
    ("JSON", "#292929"),
    ("YAML", "#cb171e"),
    // ... more languages
];
```

## Database Schema Changes

**No database changes required.** All statistics are computed on-demand from the git repository. 

For future optimization, a caching table could be added:

```sql
-- Optional future enhancement
CREATE TABLE repo_stats_cache (
    address TEXT NOT NULL,
    name TEXT NOT NULL,
    branch TEXT NOT NULL,
    stats_json TEXT NOT NULL, -- JSON of RepoStats
    computed_at INTEGER NOT NULL, -- Unix timestamp
    commit_hash TEXT NOT NULL, -- HEAD commit when computed
    PRIMARY KEY(address, name, branch),
    FOREIGN KEY(address, name) REFERENCES repos(address, name)
);

CREATE INDEX idx_repo_stats_cache_computed ON repo_stats_cache(computed_at);
```

This caching is **out of scope** for v1 but provides a clear optimization path.

## Backend Implementation

### File Structure

```
repobox-server/src/
├── routes.rs              # Add stats route handler
├── git.rs                 # Existing git utilities
├── stats/                 # New module
│   ├── mod.rs            # Public interface
│   ├── analyzer.rs       # Core analysis logic
│   ├── languages.rs      # Language detection & colors
│   └── signers.rs        # EVM signer analysis
└── main.rs               # Wire up stats module
```

### Core Analysis Logic

#### 1. Language Detection (`stats/languages.rs`)

```rust
pub struct LanguageDetector {
    extension_map: HashMap<String, String>,
    language_colors: HashMap<String, String>,
}

impl LanguageDetector {
    pub fn detect_file_language(&self, path: &str) -> Option<String> {
        // Extract extension and map to language
        // Special cases: Dockerfile, Makefile, etc.
    }
    
    pub fn get_language_color(&self, language: &str) -> &str {
        // Return hex color for language
    }
}
```

**Language Detection Logic:**
1. Extract file extension from path
2. Map extension to language using static mapping
3. Handle special files (Dockerfile, Makefile, .gitignore, etc.)
4. Return "Other" for unknown extensions

**Extension Mapping (subset):**
```rust
static EXTENSIONS: &[(&str, &str)] = &[
    // Web
    ("js", "JavaScript"),
    ("jsx", "JavaScript"),  
    ("ts", "TypeScript"),
    ("tsx", "TypeScript"),
    ("html", "HTML"),
    ("css", "CSS"),
    ("scss", "CSS"),
    
    // Systems
    ("rs", "Rust"),
    ("go", "Go"),
    ("c", "C"),
    ("cpp", "C++"),
    ("cc", "C++"),
    ("py", "Python"),
    ("java", "Java"),
    
    // Config/Data
    ("json", "JSON"),
    ("yml", "YAML"),
    ("yaml", "YAML"),
    ("toml", "TOML"),
    ("md", "Markdown"),
    ("sh", "Shell"),
    
    // Add more as needed...
];
```

#### 2. Line Counting (`stats/analyzer.rs`)

```rust
pub struct RepoAnalyzer {
    repo_path: PathBuf,
    branch: String,
    language_detector: LanguageDetector,
}

impl RepoAnalyzer {
    pub async fn analyze_repository(&self) -> Result<RepoStats, AnalysisError> {
        let files = self.get_tracked_files()?;
        let mut language_stats = HashMap::new();
        let mut total_lines = 0;
        
        for file_path in files {
            let language = self.language_detector.detect_file_language(&file_path);
            let line_count = self.count_lines_in_file(&file_path)?;
            
            total_lines += line_count;
            
            let lang_name = language.unwrap_or_else(|| "Other".to_string());
            let entry = language_stats.entry(lang_name).or_insert(LanguageStats::new());
            entry.lines += line_count;
            entry.files += 1;
        }
        
        let unique_signers = self.count_unique_signers().await?;
        let repo_age = self.calculate_repository_age()?;
        
        Ok(RepoStats {
            language_breakdown: self.finalize_language_stats(language_stats, total_lines),
            total_lines,
            total_files: files.len(),
            unique_signers,
            repository_age_days: repo_age,
            // ...
        })
    }
    
    fn get_tracked_files(&self) -> Result<Vec<String>, AnalysisError> {
        // Use `git ls-tree -r --name-only HEAD` to get all tracked files
        // Filter out binary files using `git check-attr`
        // Return list of text file paths
    }
    
    fn count_lines_in_file(&self, file_path: &str) -> Result<usize, AnalysisError> {
        // Use `git show HEAD:{file_path} | wc -l`
        // Handle binary files (return 0)
        // Skip empty files
    }
}
```

#### 3. Signer Analysis (`stats/signers.rs`)

```rust
pub struct SignerAnalyzer {
    repo_path: PathBuf,
}

impl SignerAnalyzer {
    pub async fn count_unique_signers(&self) -> Result<u32, AnalysisError> {
        // Use existing git.rs utilities to extract EVM signatures
        // Get all commits: `git rev-list --all`
        // For each commit, extract signer using existing EVM signature logic
        // Count unique signers
        
        let commits = self.get_all_commit_hashes()?;
        let mut unique_signers = HashSet::new();
        
        for commit_hash in commits {
            if let Some(signer) = self.extract_signer(&commit_hash).await? {
                unique_signers.insert(signer);
            }
        }
        
        Ok(unique_signers.len() as u32)
    }
    
    async fn extract_signer(&self, commit_hash: &str) -> Result<Option<String>, AnalysisError> {
        // Reuse existing EVM signature extraction logic from git.rs
        // Return None for unsigned commits
    }
}
```

#### 4. Age Calculation

```rust
impl RepoAnalyzer {
    fn calculate_repository_age(&self) -> Result<u32, AnalysisError> {
        // Get first commit timestamp: `git rev-list --reverse HEAD | head -1`
        // Then: `git show -s --format=%ct <first_commit>`
        // Calculate days between first commit and now
        
        let first_commit = self.get_first_commit_hash()?;
        let first_timestamp = self.get_commit_timestamp(&first_commit)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let age_seconds = now.saturating_sub(first_timestamp);
        let age_days = age_seconds / (24 * 60 * 60);
        
        Ok(age_days as u32)
    }
}
```

### Route Handler

Add to `repobox-server/src/routes.rs`:

```rust
// Add to router()
.route("/{address}/{repo}/stats", get(repo_stats))

async fn repo_stats(
    State(state): State<Arc<AppState>>,
    Path((address, repo)): Path<(String, String)>,
    Query(query): Query<StatsQuery>,
) -> Response {
    let repo_path = match repo_path(address.clone(), repo.clone()) {
        Ok(path) => path,
        Err(status) => return status.into_response(),
    };

    // Check read access (reuse existing logic)
    if let Err(denied) = check_read_access(&state, &repo_path, &headers) {
        return denied;
    }

    let branch = query.branch.unwrap_or_else(|| "HEAD".to_string());
    
    match stats::analyze_repository(&state.data_dir, &repo_path, &branch).await {
        Ok(stats) => Json(stats).into_response(),
        Err(error) => {
            tracing::error!("Stats analysis failed: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, "Analysis failed").into_response()
        }
    }
}

#[derive(serde::Deserialize)]
struct StatsQuery {
    branch: Option<String>,
}
```

## Frontend Implementation

### Component Structure

```
web/src/components/
├── RepoStatsCards.tsx     # Main stats display component
├── LanguageBar.tsx        # Language breakdown bar
├── StatCard.tsx           # Individual stat card
└── LanguageTooltip.tsx    # Hover tooltip for languages
```

### Main Stats Component

```typescript
// web/src/components/RepoStatsCards.tsx
'use client';

import { useState, useEffect } from 'react';
import LanguageBar from './LanguageBar';
import StatCard from './StatCard';

interface RepoStatsCardsProps {
  address: string;
  name: string;
  branch?: string;
}

interface RepoStats {
  language_breakdown: LanguageStats[];
  total_lines: number;
  total_files: number;
  unique_signers: number;
  repository_age_days: number;
  last_computed: string;
  computation_time_ms: number;
  branch: string;
}

export default function RepoStatsCards({ address, name, branch = 'HEAD' }: RepoStatsCardsProps) {
  const [stats, setStats] = useState<RepoStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchStats() {
      try {
        setLoading(true);
        setError(null);
        
        const branchParam = branch !== 'HEAD' ? `?branch=${branch}` : '';
        const response = await fetch(`/api/explorer/repos/${address}/${name}/stats${branchParam}`);
        
        if (!response.ok) {
          throw new Error(`Failed to fetch stats: ${response.status}`);
        }
        
        const data = await response.json();
        setStats(data);
      } catch (err) {
        console.error('Stats fetch error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load statistics');
      } finally {
        setLoading(false);
      }
    }

    fetchStats();
  }, [address, name, branch]);

  if (loading) {
    return <RepoStatsCardsSkeleton />;
  }

  if (error || !stats) {
    return (
      <div className="repo-stats-error">
        <p>Unable to load repository statistics</p>
        {error && <p className="error-detail">{error}</p>}
      </div>
    );
  }

  const topLanguages = stats.language_breakdown.slice(0, 4); // Show top 4
  const hasOtherLanguages = stats.language_breakdown.length > 4;

  return (
    <div className="repo-stats-cards">
      {/* Language Breakdown Bar */}
      {topLanguages.length > 0 && (
        <div className="repo-stats-section">
          <LanguageBar 
            languages={topLanguages} 
            showOthers={hasOtherLanguages}
          />
        </div>
      )}

      {/* Statistics Cards Grid */}
      <div className="repo-stats-grid">
        <StatCard
          title="Total Lines"
          value={stats.total_lines.toLocaleString()}
          label="SLOC"
          icon="📊"
        />
        
        <StatCard
          title="Contributors"
          value={stats.unique_signers.toString()}
          label="SIGNERS"
          icon="👥"
        />
        
        <StatCard
          title="Repository Age"
          value={`${stats.repository_age_days} days`}
          label="AGE"
          icon="📅"
        />
        
        <StatCard
          title="Latest Language"
          value={topLanguages[0]?.name || "Unknown"}
          label="RECENT LANG"
          icon="🔤"
        />
      </div>

      {/* Computation Time Footer */}
      <div className="repo-stats-footer">
        <small>
          Computed in {stats.computation_time_ms}ms • 
          Last updated {new Date(stats.last_computed).toLocaleTimeString()}
        </small>
      </div>
    </div>
  );
}
```

### Language Bar Component

```typescript
// web/src/components/LanguageBar.tsx
interface LanguageBarProps {
  languages: LanguageStats[];
  showOthers?: boolean;
}

export default function LanguageBar({ languages, showOthers = false }: LanguageBarProps) {
  return (
    <div className="language-bar-container">
      <div className="language-bar">
        {languages.map((lang, index) => (
          <LanguageSegment 
            key={lang.name}
            language={lang}
            isFirst={index === 0}
            isLast={index === languages.length - 1 && !showOthers}
          />
        ))}
        {showOthers && (
          <LanguageSegment 
            language={{
              name: "Other",
              percentage: 100 - languages.reduce((sum, l) => sum + l.percentage, 0),
              color: "#cccccc",
              lines: 0,
              files: 0,
              extensions: []
            }}
            isLast={true}
          />
        )}
      </div>

      {/* Language Labels */}
      <div className="language-labels">
        {languages.map((lang) => (
          <div key={lang.name} className="language-label">
            <span 
              className="language-color-dot" 
              style={{ backgroundColor: lang.color }}
            />
            <span className="language-name">{lang.name}</span>
            <span className="language-percentage">{lang.percentage.toFixed(1)}%</span>
          </div>
        ))}
      </div>
    </div>
  );
}
```

### StatCard Component

```typescript
// web/src/components/StatCard.tsx
interface StatCardProps {
  title: string;
  value: string;
  label: string;
  icon?: string;
  className?: string;
}

export default function StatCard({ title, value, label, icon, className = '' }: StatCardProps) {
  return (
    <div className={`stat-card ${className}`}>
      {icon && <div className="stat-card-icon">{icon}</div>}
      <div className="stat-card-content">
        <div className="stat-card-value">{value}</div>
        <div className="stat-card-label">{label}</div>
      </div>
    </div>
  );
}
```

### Integration with Repo Detail Page

Modify `/web/src/app/explore/[address]/[name]/page.tsx` to include the stats cards:

```typescript
// Add after repository header, before tabs
<RepoStatsCards 
  address={address} 
  name={name} 
  branch={selectedBranch}
/>
```

### CSS Styles

```css
/* web/src/styles/repo-stats.css */

.repo-stats-cards {
  margin: 1.5rem 0;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1.5rem;
  background: var(--bg-secondary);
}

.repo-stats-section + .repo-stats-section {
  margin-top: 1.5rem;
}

/* Language Bar */
.language-bar {
  height: 8px;
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  margin-bottom: 1rem;
}

.language-segment {
  height: 100%;
  transition: opacity 0.2s;
}

.language-segment:hover {
  opacity: 0.8;
}

.language-labels {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.language-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.language-color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

/* Stats Grid */
.repo-stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

@media (max-width: 768px) {
  .repo-stats-grid {
    grid-template-columns: 1fr;
  }
}

.stat-card {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 1rem;
  text-align: center;
  transition: box-shadow 0.2s;
}

.stat-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.stat-card-icon {
  font-size: 1.5rem;
  margin-bottom: 0.5rem;
}

.stat-card-value {
  font-size: 1.5rem;
  font-weight: bold;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.stat-card-label {
  font-size: 0.75rem;
  text-transform: uppercase;
  color: var(--text-secondary);
  font-weight: 500;
  letter-spacing: 0.5px;
}

.repo-stats-footer {
  text-align: center;
  margin-top: 1rem;
  color: var(--text-secondary);
}

/* Loading skeleton */
.repo-stats-skeleton {
  /* Skeleton loader styles */
}

.repo-stats-error {
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary);
}
```

## Performance Considerations

### Computation Complexity

**Expected Performance:**
- **Small repos** (<1k files): 200-500ms
- **Medium repos** (1k-10k files): 500ms-2s  
- **Large repos** (>10k files): 2-10s

**Optimization Strategies:**

1. **Parallel Processing**: Analyze files in parallel using Tokio tasks
2. **Binary File Detection**: Skip binary files early using `git check-attr`
3. **Line Count Optimization**: Use `wc -l` for faster line counting
4. **Early Termination**: Support timeout for very large repositories
5. **Streaming**: Process files in batches to avoid memory issues

### Caching Strategy (Future Enhancement)

For v1, statistics are computed on-demand. Future versions can implement:

1. **Branch-based caching**: Cache stats per branch/commit hash
2. **Incremental updates**: Only recompute when files change
3. **Background processing**: Pre-compute stats for popular repositories
4. **CDN caching**: Cache API responses with HTTP headers

### Memory Management

```rust
// Use streaming for large repositories
const MAX_CONCURRENT_FILES: usize = 100;
const MAX_LINE_COUNT_PER_FILE: usize = 50_000;

async fn analyze_files_batched(&self, file_paths: Vec<String>) -> Result<LanguageStats, Error> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_FILES));
    
    let results = stream::iter(file_paths)
        .map(|path| {
            let semaphore = semaphore.clone();
            async move {
                let _permit = semaphore.acquire().await?;
                self.analyze_single_file(&path).await
            }
        })
        .buffered(MAX_CONCURRENT_FILES)
        .collect::<Vec<_>>()
        .await;
    
    // Aggregate results...
}
```

## Security Considerations

### Input Validation

1. **Branch Name Sanitization**: Reuse existing `sanitizeBranchName()` logic
2. **Path Traversal Protection**: Only process tracked git files
3. **Resource Limits**: Implement timeouts and memory limits
4. **Access Control**: Reuse existing repository access control

### Denial of Service Protection

```rust
const MAX_ANALYSIS_TIME_SECONDS: u64 = 30;
const MAX_FILES_TO_ANALYZE: usize = 50_000;
const MAX_MEMORY_MB: usize = 256;

pub async fn analyze_repository_with_limits(&self) -> Result<RepoStats, AnalysisError> {
    let timeout_duration = Duration::from_secs(MAX_ANALYSIS_TIME_SECONDS);
    
    tokio::time::timeout(timeout_duration, async {
        let file_count = self.get_tracked_file_count()?;
        if file_count > MAX_FILES_TO_ANALYZE {
            return Err(AnalysisError::RepositoryTooLarge(file_count));
        }
        
        self.analyze_repository_impl().await
    }).await?
}
```

### Error Handling

- **Graceful degradation**: Show partial stats if some analysis fails
- **User feedback**: Clear error messages for analysis failures
- **Logging**: Log analysis errors for debugging
- **Rate limiting**: Prevent abuse of expensive computation

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let detector = LanguageDetector::new();
        assert_eq!(detector.detect_file_language("main.rs"), Some("Rust".to_string()));
        assert_eq!(detector.detect_file_language("app.tsx"), Some("TypeScript".to_string()));
        assert_eq!(detector.detect_file_language("unknown.xyz"), None);
    }

    #[test] 
    fn test_line_counting() {
        // Test line counting logic
    }

    #[tokio::test]
    async fn test_signer_analysis() {
        // Test unique signer counting
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_repo_stats_api() {
    let app = create_test_app().await;
    
    // Create test repository with known characteristics
    let test_repo = create_test_repo_with_stats().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/repos/0x123/test-repo/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
        
    assert_eq!(response.status(), StatusCode::OK);
    
    let stats: RepoStats = serde_json::from_slice(
        &hyper::body::to_bytes(response.into_body()).await.unwrap()
    ).unwrap();
    
    assert_eq!(stats.total_lines, 150); // Known test data
    assert_eq!(stats.language_breakdown.len(), 2);
    assert_eq!(stats.unique_signers, 1);
}
```

### Frontend Tests

```typescript
// web/src/components/__tests__/RepoStatsCards.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import RepoStatsCards from '../RepoStatsCards';

// Mock fetch
global.fetch = jest.fn();

describe('RepoStatsCards', () => {
  beforeEach(() => {
    (fetch as jest.Mock).mockClear();
  });

  it('displays loading skeleton initially', () => {
    render(<RepoStatsCards address="0x123" name="test-repo" />);
    expect(screen.getByTestId('stats-skeleton')).toBeInTheDocument();
  });

  it('displays stats after successful fetch', async () => {
    const mockStats = {
      language_breakdown: [
        { name: 'TypeScript', percentage: 80, color: '#3178c6', lines: 800, files: 10, extensions: ['.ts'] }
      ],
      total_lines: 1000,
      unique_signers: 3,
      repository_age_days: 30
    };

    (fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockStats,
    });

    render(<RepoStatsCards address="0x123" name="test-repo" />);

    await waitFor(() => {
      expect(screen.getByText('1,000')).toBeInTheDocument(); // Total lines
      expect(screen.getByText('3')).toBeInTheDocument(); // Signers
      expect(screen.getByText('TypeScript')).toBeInTheDocument();
    });
  });

  it('handles fetch errors gracefully', async () => {
    (fetch as jest.Mock).mockRejectedValueOnce(new Error('Network error'));

    render(<RepoStatsCards address="0x123" name="test-repo" />);

    await waitFor(() => {
      expect(screen.getByText(/Unable to load repository statistics/)).toBeInTheDocument();
    });
  });
});
```

### Performance Tests

```rust
#[tokio::test]
async fn test_analysis_performance() {
    let large_repo = create_large_test_repo().await; // 5k files
    
    let start = Instant::now();
    let stats = RepoAnalyzer::new(&large_repo)
        .analyze_repository()
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_secs() < 10, "Analysis took too long: {:?}", duration);
    assert!(stats.total_files > 1000);
}
```

## Implementation Timeline

### Phase 1: Backend Foundation (Week 1)
- [ ] Create `stats` module structure
- [ ] Implement language detection
- [ ] Implement line counting
- [ ] Add stats API route
- [ ] Basic error handling and validation

### Phase 2: Complete Analysis (Week 1-2)
- [ ] Implement signer analysis (reuse existing EVM logic)
- [ ] Add repository age calculation
- [ ] Performance optimizations (parallelization)
- [ ] Comprehensive testing

### Phase 3: Frontend Components (Week 2)
- [ ] Create `RepoStatsCards` component
- [ ] Implement `LanguageBar` visualization
- [ ] Add loading states and error handling
- [ ] CSS styling and responsive design

### Phase 4: Integration & Polish (Week 2-3)
- [ ] Integrate into repo detail page
- [ ] Cross-browser testing
- [ ] Performance testing with large repositories
- [ ] Documentation and examples

### Phase 5: Testing & Deployment (Week 3)
- [ ] End-to-end testing
- [ ] Load testing
- [ ] Security review
- [ ] Production deployment

## Future Enhancements

### V2 Features
1. **Historical Trends**: Track language distribution over time
2. **Advanced Metrics**: Complexity scores, test coverage estimation
3. **Comparison**: Compare stats across branches or repositories  
4. **Caching**: Implement smart caching for faster load times
5. **Filters**: Filter by date range, contributor, file types

### V3 Features
1. **Real-time Updates**: WebSocket-based live statistics
2. **Collaboration Insights**: Activity patterns, contribution graphs
3. **Code Quality**: Integration with linting and analysis tools
4. **Export**: Export stats as JSON/CSV for external analysis

## Dependencies

### New Rust Dependencies
```toml
# Add to repobox-server/Cargo.toml
[dependencies]
# Existing dependencies...
tokio-stream = "0.1"        # For streaming file processing
serde_json = "1.0"          # JSON serialization (likely already present)
```

### Frontend Dependencies  
```json
// No new dependencies required - using existing React infrastructure
```

### System Dependencies
- Git (already required)
- Standard Unix utilities (`wc`, `grep`) for file processing

## Conclusion

This specification provides a comprehensive plan for implementing repository statistics cards in repo.box. The design emphasizes:

- **Performance**: Optimized analysis algorithms and parallel processing
- **Accuracy**: Precise language detection and line counting using git operations
- **User Experience**: Clean, responsive UI with loading states and error handling
- **Maintainability**: Well-structured code with comprehensive testing
- **Security**: Proper input validation and resource limits

The feature enhances the repository explorer experience by providing immediate insights into repository composition and contributor activity, helping users quickly understand the nature and scale of any codebase.

---

**Next Steps**: Begin implementation with Phase 1 (Backend Foundation), focusing on core analysis logic and API endpoint creation.