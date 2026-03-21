# Commit Detail Page with Diff Viewer

**Specification ID**: `SPEC-COMMIT-001`  
**Priority**: P1  
**Tags**: explorer, ui, git, diff  
**Author**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b)  
**Date**: 2026-03-21  

## Summary

Implement a dedicated commit detail page accessible via `/explore/[address]/[name]/commit/[hash]` that displays comprehensive commit information including full diff with colored additions/deletions, metadata, and syntax-highlighted code blocks for all changed files.

## Context and Motivation

**Current State:**
- Commit list in repo explorer shows basic commit info (hash, message, author, timestamp)
- Commit hashes are displayed but not clickable
- No way to view commit details, changes, or diffs
- Users cannot inspect what changed in individual commits

**Problem:**
- Missing crucial git workflow functionality for code review
- No visibility into commit content and file changes
- Difficult to understand project evolution and development history
- Poor developer experience compared to GitHub/GitLab commit pages

**Solution:**
- Create dedicated commit detail page with full diff display
- Add syntax highlighting for code changes
- Show comprehensive commit metadata
- Enable easy navigation between commits and back to repository

## Technical Specification

### Current Architecture Analysis

**Repository Explorer Structure:**
```
/explore/[address]/[name]/           # Main repo page with tabs
├── tabs: readme, files, commits, config
├── API: /api/explorer/repos/[address]/[name]/commits
├── Data: GitCommit[] from getCommitHistory()
└── UI: explore-commit-list with non-clickable hashes
```

**Existing Git Utilities (`/lib/git.ts`):**
```typescript
interface GitCommit {
  hash: string;
  author: string; 
  email: string;
  timestamp: number;
  message: string;
}

// Available functions:
getCommitHistory(address, name, limit) // Used by current commits tab
getFileContent(address, name, filePath) // Used for file viewer
```

**Missing Functionality:**
- Individual commit data fetching
- Git diff parsing and display
- File change statistics
- Commit parent/child navigation
- Syntax highlighting for diffs

### Proposed Implementation

#### 1. New Route Structure

**File**: `web/src/app/explore/[address]/[name]/commit/[hash]/page.tsx`

**Route Pattern**: `/explore/[address]/[name]/commit/[hash]`

**Examples:**
```
/explore/0x123.../myrepo/commit/abc1234567890def  # Full hash
/explore/0x123.../myrepo/commit/abc1234          # Short hash (auto-resolve)
```

#### 2. API Endpoint Enhancement

**File**: `web/src/app/api/explorer/repos/[address]/[name]/commits/[hash]/route.ts`

**New Endpoint**: `GET /api/explorer/repos/[address]/[name]/commits/[hash]`

**Response Schema:**
```typescript
interface CommitDetail {
  hash: string;           // Full 40-char hash
  shortHash: string;      // First 7 characters
  author: string;         // Author name
  email: string;          // Author email
  signerAddress: string;  // EVM signer address
  timestamp: number;      // Unix timestamp
  message: string;        // Commit message
  parentHash: string | null;  // Parent commit hash
  childHash: string | null;   // Next commit hash (for navigation)
  fileChanges: FileChange[];  // Changed files with diffs
  stats: {
    additions: number;    // Total lines added
    deletions: number;    // Total lines deleted
    filesChanged: number; // Number of files modified
  };
}

interface FileChange {
  path: string;          // File path
  status: 'added' | 'modified' | 'deleted' | 'renamed';
  oldPath?: string;      // For renamed files
  additions: number;     // Lines added in this file
  deletions: number;     // Lines deleted in this file
  hunks: DiffHunk[];     // Diff content
}

interface DiffHunk {
  oldStart: number;      // Starting line in old file
  oldCount: number;      // Number of lines in old file
  newStart: number;      // Starting line in new file  
  newCount: number;      // Number of lines in new file
  lines: DiffLine[];     // Individual diff lines
}

interface DiffLine {
  type: 'context' | 'addition' | 'deletion';
  content: string;       // Line content (without +/- prefix)
  oldLineNumber?: number; // Line number in old file
  newLineNumber?: number; // Line number in new file
}
```

#### 3. Git Operations Enhancement

**File**: `web/src/lib/git.ts`

**New Functions:**

```typescript
// Get detailed commit information
export function getCommitDetail(address: string, name: string, hash: string): CommitDetail | null {
  const repoPath = getRepoPath(address, name);
  
  try {
    // Normalize hash (handle short hashes)
    const fullHash = gitCommand(repoPath, `rev-parse ${hash}^{commit}`);
    
    // Get commit metadata
    const [author, email, timestamp, message] = gitCommand(
      repoPath, 
      `show -s --format='%an|%ae|%at|%B' ${fullHash}`
    ).split('|');
    
    // Get parent hash
    const parentHash = gitCommand(repoPath, `rev-parse ${fullHash}^`).trim() || null;
    
    // Get next commit (child)
    const childHash = getChildCommit(repoPath, fullHash);
    
    // Get file changes with stats
    const fileChanges = getCommitFileChanges(repoPath, fullHash);
    
    // Calculate total stats
    const stats = calculateDiffStats(fileChanges);
    
    return {
      hash: fullHash,
      shortHash: fullHash.substring(0, 7),
      author,
      email,
      signerAddress: repo.owner_address, // From repo record
      timestamp: parseInt(timestamp),
      message: message.trim(),
      parentHash,
      childHash,
      fileChanges,
      stats
    };
  } catch (error) {
    return null;
  }
}

// Get file changes for a commit
function getCommitFileChanges(repoPath: string, hash: string): FileChange[] {
  try {
    // Get list of changed files with status
    const filesOutput = gitCommand(repoPath, `diff-tree --name-status ${hash}^..${hash}`);
    const fileEntries = filesOutput.split('\n').filter(Boolean);
    
    return fileEntries.map(entry => {
      const [status, ...pathParts] = entry.split('\t');
      const path = pathParts.join('\t');
      
      // Get diff for this specific file
      const diffOutput = gitCommand(
        repoPath, 
        `diff --unified=3 ${hash}^..${hash} -- "${path}"`
      );
      
      const hunks = parseDiffOutput(diffOutput);
      const stats = calculateFileStats(hunks);
      
      return {
        path,
        status: mapGitStatus(status),
        additions: stats.additions,
        deletions: stats.deletions,
        hunks
      };
    });
  } catch (error) {
    return [];
  }
}

// Parse git diff output into structured hunks
function parseDiffOutput(diffOutput: string): DiffHunk[] {
  const lines = diffOutput.split('\n');
  const hunks: DiffHunk[] = [];
  let currentHunk: Partial<DiffHunk> | null = null;
  
  for (const line of lines) {
    // Parse hunk header: @@ -oldStart,oldCount +newStart,newCount @@
    const hunkMatch = line.match(/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/);
    if (hunkMatch) {
      if (currentHunk) {
        hunks.push(currentHunk as DiffHunk);
      }
      
      currentHunk = {
        oldStart: parseInt(hunkMatch[1]),
        oldCount: parseInt(hunkMatch[2] || '1'),
        newStart: parseInt(hunkMatch[3]),
        newCount: parseInt(hunkMatch[4] || '1'),
        lines: []
      };
      continue;
    }
    
    // Parse diff lines
    if (currentHunk && (line.startsWith(' ') || line.startsWith('+') || line.startsWith('-'))) {
      const type = line[0] === '+' ? 'addition' : 
                   line[0] === '-' ? 'deletion' : 'context';
      
      currentHunk.lines!.push({
        type,
        content: line.substring(1),
        oldLineNumber: type !== 'addition' ? calculateOldLineNumber(currentHunk.lines!, currentHunk.oldStart!) : undefined,
        newLineNumber: type !== 'deletion' ? calculateNewLineNumber(currentHunk.lines!, currentHunk.newStart!) : undefined
      });
    }
  }
  
  if (currentHunk) {
    hunks.push(currentHunk as DiffHunk);
  }
  
  return hunks;
}

// Helper functions for line number calculation and status mapping
function mapGitStatus(status: string): FileChange['status'] {
  switch (status[0]) {
    case 'A': return 'added';
    case 'M': return 'modified'; 
    case 'D': return 'deleted';
    case 'R': return 'renamed';
    default: return 'modified';
  }
}

function calculateOldLineNumber(lines: DiffLine[], startLine: number): number {
  const relevantLines = lines.filter(l => l.type !== 'addition').length;
  return startLine + relevantLines - 1;
}

function calculateNewLineNumber(lines: DiffLine[], startLine: number): number {
  const relevantLines = lines.filter(l => l.type !== 'deletion').length;
  return startLine + relevantLines - 1;
}

function getChildCommit(repoPath: string, hash: string): string | null {
  try {
    // Find commits that have this hash as parent
    const output = gitCommand(repoPath, `rev-list --children --all | grep "^${hash}"`);
    const parts = output.trim().split(' ');
    return parts.length > 1 ? parts[1] : null;
  } catch {
    return null;
  }
}
```

#### 4. UI Component Structure

**File**: `web/src/app/explore/[address]/[name]/commit/[hash]/page.tsx`

**Component Architecture:**
```
CommitDetailPage
├── CommitHeader (metadata, navigation)
├── CommitStats (additions/deletions summary)
├── FileChangeList
│   └── FileChangeItem[]
│       ├── FileHeader (name, status, stats)
│       └── DiffViewer
│           └── DiffHunk[]
│               └── DiffLine[]
└── Navigation (parent/child commit links)
```

**Core Components:**

```typescript
interface CommitDetailPageProps {
  params: { address: string; name: string; hash: string };
}

export default function CommitDetailPage({ params }: CommitDetailPageProps) {
  const [commit, setCommit] = useState<CommitDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());
  
  // Fetch commit data
  useEffect(() => {
    fetchCommitDetail();
  }, [params.hash]);
  
  return (
    <div className="commit-detail-page">
      <CommitHeader commit={commit} />
      <CommitStats stats={commit?.stats} />
      <FileChangeList 
        changes={commit?.fileChanges} 
        expanded={expandedFiles}
        onToggleExpand={toggleFileExpansion}
      />
      <CommitNavigation 
        parentHash={commit?.parentHash}
        childHash={commit?.childHash}
        repoPath={`/explore/${params.address}/${params.name}`}
      />
    </div>
  );
}
```

#### 5. Diff Viewer Implementation

**Component**: `DiffViewer`

**Key Features:**
- Syntax highlighting using `react-syntax-highlighter`
- Line-by-line diff display with proper alignment
- Color coding for additions (+), deletions (-), and context
- Line number display for both old and new versions
- Expandable/collapsible hunks for large diffs
- Copy-to-clipboard functionality

**Styling Approach:**
```css
/* Diff viewer base styles */
.diff-viewer {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  border: 1px solid var(--bp-border);
  border-radius: 8px;
  overflow: hidden;
}

/* File header */
.diff-file-header {
  background: var(--bp-surface);
  padding: 12px 16px;
  border-bottom: 1px solid var(--bp-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.diff-file-path {
  font-weight: 500;
  color: var(--bp-heading);
}

.diff-file-stats {
  display: flex;
  gap: 8px;
  font-size: 11px;
}

.diff-additions {
  color: #4ade80;
}

.diff-deletions {
  color: #f87171;
}

/* Diff content */
.diff-content {
  max-height: 600px;
  overflow-y: auto;
}

.diff-hunk {
  border-bottom: 1px solid var(--bp-border);
}

.diff-hunk-header {
  background: rgba(79, 195, 247, 0.1);
  padding: 4px 16px;
  font-size: 11px;
  color: var(--bp-accent);
  font-weight: 500;
}

.diff-line {
  display: flex;
  align-items: center;
  min-height: 20px;
  position: relative;
}

.diff-line-numbers {
  display: flex;
  min-width: 80px;
  background: var(--bp-surface);
  border-right: 1px solid var(--bp-border);
  font-size: 10px;
  color: var(--bp-dim);
  user-select: none;
}

.diff-line-old,
.diff-line-new {
  width: 40px;
  text-align: right;
  padding: 0 8px;
}

.diff-line-content {
  flex: 1;
  padding: 0 12px;
  white-space: pre-wrap;
  word-break: break-all;
}

/* Line type styling */
.diff-line.addition {
  background: rgba(74, 222, 128, 0.1);
}

.diff-line.addition .diff-line-content {
  background: rgba(74, 222, 128, 0.15);
}

.diff-line.deletion {
  background: rgba(248, 113, 113, 0.1);
}

.diff-line.deletion .diff-line-content {
  background: rgba(248, 113, 113, 0.15);
}

.diff-line.context {
  background: transparent;
}

/* Syntax highlighting integration */
.diff-line-content code {
  background: transparent !important;
  padding: 0 !important;
}
```

#### 6. Syntax Highlighting Integration

**Dependencies**: Add to `package.json`
```json
{
  "dependencies": {
    "react-syntax-highlighter": "^15.5.0",
    "@types/react-syntax-highlighter": "^15.5.11"
  }
}
```

**Implementation:**
```typescript
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

function SyntaxHighlightedDiffLine({ 
  content, 
  filePath, 
  type 
}: { 
  content: string; 
  filePath: string; 
  type: 'addition' | 'deletion' | 'context' 
}) {
  const language = detectLanguageFromPath(filePath);
  
  if (language === 'text' || content.trim() === '') {
    return <span>{content}</span>;
  }
  
  return (
    <SyntaxHighlighter
      language={language}
      style={vscDarkPlus}
      customStyle={{
        margin: 0,
        padding: 0,
        background: 'transparent',
        fontSize: '12px',
        lineHeight: '20px'
      }}
      codeTagProps={{
        style: {
          background: 'transparent',
          fontFamily: 'inherit'
        }
      }}
    >
      {content}
    </SyntaxHighlighter>
  );
}

function detectLanguageFromPath(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase();
  const languageMap: Record<string, string> = {
    'js': 'javascript',
    'jsx': 'jsx',
    'ts': 'typescript',
    'tsx': 'tsx',
    'py': 'python',
    'rs': 'rust',
    'go': 'go',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c',
    'cs': 'csharp',
    'php': 'php',
    'rb': 'ruby',
    'sh': 'bash',
    'yml': 'yaml',
    'yaml': 'yaml',
    'json': 'json',
    'md': 'markdown',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sql': 'sql'
  };
  
  return languageMap[ext || ''] || 'text';
}
```

### Database and Storage

**No database changes required** - all commit data is read directly from git repository using existing git operations.

**Performance Considerations:**
- Cache commit details in memory for frequently accessed commits
- Implement pagination for commits with many file changes
- Lazy-load diff content for large files
- Add timeout protection for git operations

### URL Structure and Navigation

**New Routes:**
```
/explore/[address]/[name]/commit/[hash]     # Commit detail page
/explore/[address]/[name]/commits           # All commits page (future)
/explore/[address]/[name]/compare/[base]...[head]  # Commit comparison (future)
```

**Navigation Flow:**
```
Repository Page → Commits Tab → Click commit hash → Commit Detail Page
                                     ↕
                              Navigate parent/child commits
                                     ↕
                               Back to repository
```

**Breadcrumb Structure:**
```
repo.box > Explorer > [address] > [name] > commit [short-hash]
```

## Implementation Plan

### Phase 1: Core Infrastructure (4-6 hours)

**Backend API Development:**
1. **Create API endpoint** (`/api/explorer/repos/[address]/[name]/commits/[hash]/route.ts`)
   - Implement request validation and error handling
   - Add repository existence verification
   - Return 404 for invalid/non-existent commits

2. **Enhance git utilities** (`/lib/git.ts`) 
   - Add `getCommitDetail()` function
   - Implement diff parsing with `parseDiffOutput()`
   - Add helper functions for line numbers and stats
   - Handle edge cases (merge commits, empty commits, binary files)

3. **Unit tests for git operations**
   - Test commit detail extraction
   - Test diff parsing accuracy  
   - Test error handling for invalid hashes
   - Performance testing for large commits

### Phase 2: UI Components (6-8 hours)

**Component Development:**
1. **Create base page** (`/explore/[address]/[name]/commit/[hash]/page.tsx`)
   - Set up routing and parameter extraction
   - Implement loading states and error handling
   - Add basic layout structure

2. **Build commit header component**
   - Display commit metadata (hash, author, signer, timestamp)
   - Add copy-to-clipboard for hashes
   - Include navigation breadcrumbs

3. **Implement diff viewer components**
   - Create `DiffViewer`, `FileChangeList`, `DiffHunk` components
   - Add line number display and proper alignment
   - Implement expand/collapse functionality for files

4. **Add navigation components**
   - Parent/child commit navigation buttons
   - Back to repository link
   - Quick jump to specific file changes

### Phase 3: Styling and Polish (3-4 hours)

**Visual Enhancement:**
1. **Create diff-specific CSS**
   - Color-coded additions/deletions
   - Consistent spacing and typography
   - Responsive design for mobile viewing
   - Dark theme integration

2. **Add syntax highlighting**
   - Install and configure `react-syntax-highlighter`
   - Implement language detection
   - Optimize highlighting performance
   - Handle edge cases (binary files, very long lines)

3. **Implement interactive features**
   - File expansion/collapse
   - Copy diff content to clipboard
   - Keyboard shortcuts for navigation
   - Scroll-to-file functionality

### Phase 4: Integration and Testing (3-4 hours)

**Integration Work:**
1. **Update existing commit list**
   - Make commit hashes clickable links
   - Add hover states and visual feedback
   - Update existing commit tab styling

2. **Cross-page navigation**
   - Ensure proper URL handling and browser history
   - Add OpenGraph meta tags for sharing
   - Test deep linking to specific commits

3. **Performance optimization**
   - Implement lazy loading for large diffs
   - Add request caching where appropriate
   - Optimize rendering for commits with many files
   - Add loading spinners and progressive enhancement

## Affected Files

### New Files
```
web/src/app/explore/[address]/[name]/commit/[hash]/page.tsx          # Main commit detail page
web/src/app/api/explorer/repos/[address]/[name]/commits/[hash]/route.ts  # API endpoint
web/src/components/DiffViewer.tsx                                    # Diff display component
web/src/components/FileChangeList.tsx                               # File changes list
web/src/components/CommitHeader.tsx                                  # Commit metadata header
web/src/components/CommitNavigation.tsx                             # Parent/child navigation
```

### Modified Files
```
web/src/lib/git.ts                                 # Add commit detail functions
web/src/app/explore/[address]/[name]/page.tsx     # Make commit hashes clickable
web/src/app/globals.css                           # Add diff viewer styles
web/src/lib/utils.ts                              # Add diff-related utilities
package.json                                      # Add syntax highlighter dependencies
```

### Testing Files
```
web/src/lib/__tests__/git.test.ts                # Test commit detail functions
web/src/components/__tests__/DiffViewer.test.tsx # Test diff display
cypress/e2e/commit-detail.cy.ts                  # E2E tests for commit page
```

## Testing Strategy

### Unit Tests

**Git Operations (`web/src/lib/__tests__/git.test.ts`):**
```typescript
describe('getCommitDetail', () => {
  it('should fetch commit metadata correctly', () => {
    // Test commit hash, author, message extraction
  });
  
  it('should handle short hash input', () => {
    // Test hash resolution from 7-char to full hash
  });
  
  it('should return null for invalid hash', () => {
    // Test error handling
  });
  
  it('should parse diff output correctly', () => {
    // Test diff parsing with additions, deletions, context
  });
  
  it('should calculate line numbers accurately', () => {
    // Test line number calculation in hunks
  });
});

describe('parseDiffOutput', () => {
  it('should parse simple file diff', () => {
    const diffOutput = `@@ -1,3 +1,3 @@
 line 1
-old line 2
+new line 2
 line 3`;
    
    const hunks = parseDiffOutput(diffOutput);
    expect(hunks).toHaveLength(1);
    expect(hunks[0].lines).toHaveLength(4);
  });
  
  it('should handle multiple hunks', () => {
    // Test parsing diffs with multiple hunks
  });
  
  it('should handle binary files gracefully', () => {
    // Test binary file diff handling
  });
});
```

**Component Tests (`web/src/components/__tests__/DiffViewer.test.tsx`):**
```typescript
describe('DiffViewer', () => {
  it('should render file changes correctly', () => {
    // Test file list rendering
  });
  
  it('should show/hide diffs on expand', () => {
    // Test expand/collapse functionality
  });
  
  it('should highlight syntax correctly', () => {
    // Test syntax highlighting integration
  });
  
  it('should display line numbers accurately', () => {
    // Test line number display
  });
});
```

### Integration Tests

**API Tests:**
```typescript
describe('/api/explorer/repos/[address]/[name]/commits/[hash]', () => {
  it('should return commit details for valid hash', async () => {
    const response = await fetch('/api/explorer/repos/test-addr/test-repo/commits/abc123');
    expect(response.status).toBe(200);
    const data = await response.json();
    expect(data).toHaveProperty('hash');
    expect(data).toHaveProperty('fileChanges');
  });
  
  it('should return 404 for invalid hash', async () => {
    const response = await fetch('/api/explorer/repos/test-addr/test-repo/commits/invalid');
    expect(response.status).toBe(404);
  });
  
  it('should handle repository not found', async () => {
    const response = await fetch('/api/explorer/repos/missing/repo/commits/abc123');
    expect(response.status).toBe(404);
  });
});
```

### End-to-End Tests

**Cypress Tests (`cypress/e2e/commit-detail.cy.ts`):**
```typescript
describe('Commit Detail Page', () => {
  beforeEach(() => {
    cy.visit('/explore/0x123.../test-repo');
  });
  
  it('should navigate to commit detail from commits tab', () => {
    cy.get('[data-testid=commits-tab]').click();
    cy.get('.explore-commit-hash').first().click();
    cy.url().should('include', '/commit/');
    cy.get('.commit-header').should('exist');
  });
  
  it('should display commit metadata correctly', () => {
    cy.visit('/explore/0x123.../test-repo/commit/abc123');
    cy.get('.commit-hash').should('contain', 'abc123');
    cy.get('.commit-author').should('exist');
    cy.get('.commit-timestamp').should('exist');
  });
  
  it('should show file changes with diffs', () => {
    cy.visit('/explore/0x123.../test-repo/commit/abc123');
    cy.get('.file-change-item').should('exist');
    cy.get('.diff-line.addition').should('exist');
    cy.get('.diff-line.deletion').should('exist');
  });
  
  it('should allow expanding/collapsing file diffs', () => {
    cy.visit('/explore/0x123.../test-repo/commit/abc123');
    cy.get('.file-header').first().click();
    cy.get('.diff-content').should('be.visible');
    cy.get('.file-header').first().click();
    cy.get('.diff-content').should('not.be.visible');
  });
  
  it('should navigate between parent/child commits', () => {
    cy.visit('/explore/0x123.../test-repo/commit/abc123');
    cy.get('[data-testid=parent-commit]').click();
    cy.url().should('include', '/commit/def456');
  });
});
```

### Performance Tests

**Load Testing:**
```bash
# Test large commit handling
curl -w "%{time_total}" /api/explorer/repos/test/repo/commits/large-commit-hash

# Test concurrent requests
ab -n 100 -c 10 /api/explorer/repos/test/repo/commits/abc123

# Memory usage with large diffs
node --inspect test-large-commit-parsing.js
```

## Acceptance Criteria

### ✅ Functional Requirements

**Core Functionality:**
1. **Commit Detail Display**:
   - ✅ Show full commit metadata (hash, author, signer, timestamp, message)
   - ✅ Display parent commit hash and navigation link
   - ✅ Show file change statistics (additions, deletions, files changed)
   - ✅ Handle both full and short commit hashes in URL

2. **Diff Viewer**:
   - ✅ Display unified diff for each changed file
   - ✅ Color-code additions (green), deletions (red), and context
   - ✅ Show line numbers for both old and new versions
   - ✅ Support expand/collapse for individual files
   - ✅ Handle all file change types (added, modified, deleted, renamed)

3. **Syntax Highlighting**:
   - ✅ Auto-detect programming language from file extension
   - ✅ Apply syntax highlighting to diff content
   - ✅ Fall back gracefully for unsupported file types
   - ✅ Maintain readability with diff colors

4. **Navigation**:
   - ✅ Clickable commit hashes in existing commits tab
   - ✅ Parent/child commit navigation buttons
   - ✅ Breadcrumb navigation back to repository
   - ✅ Proper browser history handling

### ✅ User Experience Requirements

**Interface Design:**
5. **Visual Design**:
   - ✅ Consistent with existing repo.box design system
   - ✅ Responsive layout for desktop and mobile
   - ✅ Clear visual hierarchy and spacing
   - ✅ Accessible color contrast for diff highlighting

6. **Performance**:
   - ✅ Page loads within 2 seconds for typical commits
   - ✅ Graceful handling of commits with 50+ file changes
   - ✅ Lazy loading for very large diffs (>1000 lines)
   - ✅ No browser freezing during diff parsing

7. **Usability**:
   - ✅ Copy commit hash to clipboard functionality
   - ✅ Keyboard shortcuts for navigation (n/p for next/prev commit)
   - ✅ Jump to specific file changes via anchor links
   - ✅ Clear error messages for invalid commit hashes

### ✅ Technical Requirements

**Implementation Quality:**
8. **Code Quality**:
   - ✅ TypeScript interfaces for all data structures
   - ✅ Comprehensive error handling and logging
   - ✅ Unit test coverage >90% for new functions
   - ✅ Integration tests for API endpoints

9. **Security**:
   - ✅ Proper input validation for commit hashes
   - ✅ No XSS vulnerabilities in diff content display
   - ✅ Rate limiting for API endpoints
   - ✅ No exposure of sensitive git repository data

10. **Performance**:
    - ✅ Git operations complete within 5 seconds
    - ✅ Memory usage stays under 100MB for large commits
    - ✅ No memory leaks in long-running sessions
    - ✅ Proper cleanup of temporary git resources

## Security Considerations

### Input Validation

**Commit Hash Validation:**
```typescript
function validateCommitHash(hash: string): boolean {
  // Allow full hashes (40 hex chars) and short hashes (7+ hex chars)
  return /^[a-f0-9]{7,40}$/i.test(hash);
}
```

**Path Traversal Prevention:**
- Validate all file paths in diff output
- Reject paths containing `..` or absolute paths
- Sanitize file names for display

### XSS Prevention

**Content Sanitization:**
```typescript
// Sanitize diff content before display
function sanitizeDiffLine(content: string): string {
  return content
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;');
}
```

**Component Security:**
- Use React's built-in XSS protection
- Avoid `dangerouslySetInnerHTML`
- Sanitize any user-generated content in commit messages

### Git Operation Security

**Repository Access Control:**
- Verify repository permissions before diff access
- Respect existing repository visibility settings
- No bypass of authentication/authorization

**Resource Limits:**
- Timeout git operations after 10 seconds
- Limit diff output to 10MB per commit
- Prevent infinite loops in diff parsing

## Migration and Deployment

### Deployment Strategy

**Phase 1: Backend Deployment**
1. Deploy API endpoint with feature flag
2. Test git operations in production environment
3. Monitor performance and error rates
4. Gradually enable for test repositories

**Phase 2: Frontend Deployment**
1. Deploy UI components without navigation links
2. Test page rendering and diff display
3. Enable direct URL access for testing
4. Add navigation links in existing UI

**Phase 3: Full Rollout**
1. Enable clickable commit hashes in commits tab
2. Add commit detail links throughout the application
3. Update documentation and help content
4. Monitor usage analytics and performance

### Rollback Plan

**If Critical Issues Arise:**
1. **Disable navigation links** - remove clickable hashes from commits tab
2. **Return 503 for commit detail pages** - show maintenance message
3. **Revert API endpoints** - return 404 for commit detail requests
4. **Database rollback not required** - no schema changes

**Monitoring and Alerts:**
- Error rate >5% for commit detail API
- Response time >5 seconds for git operations
- Memory usage >500MB for git processes
- User report of missing or incorrect diff data

### Documentation Updates

**User Documentation:**
1. Add commit detail page to user guide
2. Update explorer documentation with new navigation
3. Create diff reading tutorial for new users
4. Add keyboard shortcuts reference

**Developer Documentation:**
1. Document new API endpoints in OpenAPI spec
2. Add git utility functions to code documentation
3. Update testing guidelines for diff components
4. Create troubleshooting guide for git operations

## Metrics and Monitoring

### Performance Metrics

**API Performance:**
- `commit_detail_request_duration_seconds` (histogram)
- `commit_detail_request_total` (counter with status labels)
- `git_operation_duration_seconds` (histogram by operation type)
- `diff_parsing_duration_seconds` (histogram)

**User Engagement:**
- `commit_detail_page_views_total` (counter)
- `commit_navigation_clicks_total` (counter by direction)
- `file_expansion_interactions_total` (counter)
- `commit_hash_copy_events_total` (counter)

**Error Tracking:**
- `commit_detail_errors_total` (counter by error type)
- `git_operation_failures_total` (counter by operation)
- `invalid_commit_hash_requests_total` (counter)

### Alerting Rules

**Critical Alerts:**
```yaml
# High error rate
- alert: CommitDetailHighErrorRate
  expr: rate(commit_detail_errors_total[5m]) > 0.1
  for: 2m
  
# Slow git operations  
- alert: GitOperationsSlow
  expr: histogram_quantile(0.95, git_operation_duration_seconds) > 5
  for: 5m
  
# Memory usage spike
- alert: GitProcessMemoryHigh
  expr: process_resident_memory_bytes > 500 * 1024 * 1024
  for: 1m
```

**Warning Alerts:**
```yaml
# Increased latency
- alert: CommitDetailSlowResponse
  expr: histogram_quantile(0.95, commit_detail_request_duration_seconds) > 2
  for: 5m
  
# High request volume
- alert: CommitDetailHighVolume  
  expr: rate(commit_detail_request_total[5m]) > 10
  for: 10m
```

### Analytics Dashboard

**Key Metrics to Track:**
1. **Usage Patterns**:
   - Most viewed commits by repository
   - Peak usage hours and days
   - Average time spent on commit detail pages
   - Common navigation paths (parent/child/back to repo)

2. **Performance Insights**:
   - Git operation latency by repository size
   - Diff parsing time by number of files changed
   - Page load time by commit complexity
   - Error rates by commit hash type (full vs short)

3. **User Behavior**:
   - File expansion rates by file type
   - Copy-to-clipboard usage frequency
   - Navigation pattern analysis
   - Mobile vs desktop usage patterns

## Future Enhancements

### Phase 2 Features (Post-MVP)

**Enhanced Diff Features:**
1. **Side-by-side diff view** - alternative to unified diff
2. **Word-level highlighting** - highlight changed words within lines  
3. **Diff statistics visualization** - graphical representation of changes
4. **File tree navigation** - jump between files in large commits

**Commit Comparison:**
1. **Compare any two commits** - `/explore/.../compare/hash1...hash2`
2. **Branch comparison** - compare commits across branches
3. **Merge commit visualization** - show multiple parent diffs
4. **Cherry-pick detection** - identify similar changes across commits

**Interactive Features:**
1. **Inline comments** - allow commenting on specific lines
2. **Commit annotation** - user-generated notes on commits
3. **Share commit links** - social sharing with OpenGraph previews
4. **Export diffs** - download as patch files

### Phase 3 Features (Advanced)

**Advanced Visualization:**
1. **Commit graph** - visual representation of commit history
2. **File history timeline** - track file changes across commits
3. **Code evolution animation** - animated diff playback
4. **Blame integration** - show who changed each line

**AI-Powered Features:**
1. **Commit summary generation** - AI-generated commit summaries
2. **Change impact analysis** - predict affected functionality
3. **Code quality assessment** - automated code review feedback
4. **Similar commit detection** - find related changes

**Developer Tools:**
1. **API access** - programmatic access to commit data
2. **Webhook notifications** - notify on specific commit patterns
3. **IDE integration** - browser extension for developer tools
4. **CLI companion** - command-line tool for commit analysis

### Technical Debt and Optimization

**Performance Optimization:**
1. **Diff caching** - cache parsed diffs for frequently accessed commits
2. **Incremental loading** - load visible diff hunks first
3. **Background processing** - pre-process diffs for popular repos
4. **CDN integration** - cache diff assets globally

**Code Quality Improvements:**
1. **Type safety enhancement** - stricter TypeScript types
2. **Component modularity** - break down large components
3. **Testing expansion** - visual regression tests
4. **Accessibility audit** - WCAG compliance review

## Implementation Checklist

### ✅ Phase 1: Backend Infrastructure

**Git Utilities Enhancement:**
- [ ] Implement `getCommitDetail()` function in `/lib/git.ts`
- [ ] Add `parseDiffOutput()` with proper hunk parsing
- [ ] Create `getCommitFileChanges()` for file-level diffs
- [ ] Add helper functions for line number calculation
- [ ] Implement commit parent/child resolution
- [ ] Add error handling for invalid hashes and empty repos
- [ ] Write unit tests for all new git functions

**API Endpoint Creation:**
- [ ] Create `/api/explorer/repos/[address]/[name]/commits/[hash]/route.ts`
- [ ] Implement request validation and parameter extraction
- [ ] Add error handling for missing repos and invalid hashes
- [ ] Include proper TypeScript interfaces for responses
- [ ] Add request logging and performance monitoring
- [ ] Write integration tests for API endpoint

### ✅ Phase 2: Frontend Components

**Page Structure:**
- [ ] Create `/explore/[address]/[name]/commit/[hash]/page.tsx`
- [ ] Implement React component with proper routing
- [ ] Add loading states and error boundaries
- [ ] Create responsive layout structure

**Component Development:**
- [ ] Build `CommitHeader` component for metadata display
- [ ] Implement `FileChangeList` for file change overview
- [ ] Create `DiffViewer` component with proper styling
- [ ] Add `DiffHunk` component for individual diff sections
- [ ] Build `CommitNavigation` for parent/child links
- [ ] Implement expand/collapse functionality

**Styling and Interaction:**
- [ ] Add CSS classes for diff visualization
- [ ] Implement syntax highlighting integration
- [ ] Create responsive design for mobile devices
- [ ] Add interactive features (copy, expand, navigate)
- [ ] Write component unit tests

### ✅ Phase 3: Integration and Polish

**Navigation Enhancement:**
- [ ] Update existing commits tab to make hashes clickable
- [ ] Add proper Link components with Next.js routing
- [ ] Implement breadcrumb navigation
- [ ] Add browser history management

**Performance and UX:**
- [ ] Optimize git operations for large commits
- [ ] Implement lazy loading for large diffs
- [ ] Add keyboard shortcuts for navigation
- [ ] Create loading spinners and progress indicators
- [ ] Add error recovery and retry mechanisms

**Testing and Quality Assurance:**
- [ ] Write end-to-end tests with Cypress
- [ ] Perform accessibility audit and fixes
- [ ] Test with various commit types and sizes
- [ ] Validate mobile responsiveness
- [ ] Performance testing and optimization

### ✅ Phase 4: Deployment and Monitoring

**Deployment Preparation:**
- [ ] Add feature flags for gradual rollout
- [ ] Set up monitoring and alerting
- [ ] Create deployment scripts and CI/CD integration
- [ ] Document rollback procedures

**Production Deployment:**
- [ ] Deploy backend APIs to production
- [ ] Deploy frontend components with feature flags
- [ ] Enable navigation links gradually
- [ ] Monitor performance and error rates
- [ ] Collect user feedback and usage analytics

**Documentation and Communication:**
- [ ] Update user documentation
- [ ] Create developer API documentation  
- [ ] Write deployment and maintenance guides
- [ ] Communicate feature launch to users

---

**Specification approved for implementation**  
**Next steps**: Add to KANBAN.md as "In Progress" and begin Phase 1 development.

**Estimated Timeline**: 16-20 hours across 4 phases
**Dependencies**: None (uses existing infrastructure)
**Risk Level**: Low (read-only feature with comprehensive testing plan)