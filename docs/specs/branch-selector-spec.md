# Branch Selector in Repo Detail - Technical Specification

## Overview

This specification details the implementation of a branch selector dropdown in the repository detail page that allows users to switch between different branches and view branch-specific content.

## Current State Analysis

### Frontend Implementation
**File:** `/home/xiko/repobox/web/src/app/explore/[address]/[name]/page.tsx`

**Current Behavior:**
- Shows only the default branch (main)
- Displays static branch name in repository stats: `{repo.default_branch}`
- All git operations target HEAD/main branch implicitly
- File tree, commits, and README are all fetched from default branch

### Backend API Analysis
**File:** `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/route.ts`

**Current Implementation:**
- `getDefaultBranch()` returns symbolic-ref HEAD
- `getFileTree()` uses `HEAD` for git ls-tree operations
- `getCommitHistory()` shows commits from HEAD
- `getReadmeContent()` reads from `HEAD:README.md`

**Missing Capabilities:**
- No API endpoint for listing available branches
- No branch parameter in existing endpoints
- Git operations hardcoded to use HEAD

### Backend Git Library Analysis
**File:** `/home/xiko/repobox/web/src/lib/git.ts`

**Current Git Operations:**
- `getDefaultBranch()`: `git symbolic-ref HEAD`
- `getFileTree()`: `git ls-tree HEAD` (or `HEAD:path`)
- `getCommitHistory()`: `git log HEAD`
- `getFileContent()`: `git show HEAD:filepath`

**Required Enhancements:**
- Function to list all branches
- Branch parameter support in existing functions
- Validation for branch existence

## Requirements

### Functional Requirements

1. **Branch Listing**
   - Display dropdown with all available branches
   - Show current/default branch as selected initially
   - Handle repositories with no branches (empty repos)
   - Handle repositories with only one branch

2. **Branch Switching**
   - Update file tree when branch changes
   - Update commits list to show branch-specific commits
   - Update README content from selected branch
   - Preserve current path when switching branches (if path exists)
   - Update URL to reflect selected branch (optional)

3. **Content Updates**
   - File tree reflects selected branch state
   - Commit history shows branch-specific commits
   - README content loads from selected branch
   - File viewer shows files from selected branch

4. **User Experience**
   - Smooth transitions between branches
   - Loading indicators during branch switches
   - Clear visual feedback on current branch
   - Graceful handling of missing content in branches

### Non-Functional Requirements

1. **Performance**
   - Branch list cached after first load
   - Debounced branch switching to prevent rapid API calls
   - Efficient git operations using branch refs

2. **Compatibility**
   - Works with existing repository permissions
   - Maintains current URL structure compatibility
   - Preserves existing keyboard navigation

3. **Error Handling**
   - Graceful fallback to default branch on errors
   - Clear error messages for inaccessible branches
   - Handle edge cases (corrupted branches, etc.)

## Design Specification

### UI/UX Design

#### Branch Selector Placement
- **Location**: Replace the current static branch display in repository stats
- **Current**: `<span className="explore-stat-value">{repo.default_branch}</span>`
- **New**: Interactive dropdown in the same location

#### Visual Design
```tsx
// New branch selector component structure
<div className="explore-repo-stats">
  <div className="explore-stat-item">
    <span className="explore-stat-value">{repo.commit_count}</span>
    <span className="explore-stat-label">COMMITS</span>
  </div>
  <div className="explore-stat-item explore-stat-branch">
    <BranchSelector
      branches={branches}
      currentBranch={selectedBranch}
      defaultBranch={repo.default_branch}
      onChange={handleBranchChange}
    />
    <span className="explore-stat-label">BRANCH</span>
  </div>
</div>
```

#### Component Specifications
- **Trigger**: Button showing current branch name with dropdown arrow
- **Dropdown**: List of all available branches with checkmark on current
- **Styling**: Matches existing stat item styling
- **Icon**: Git branch icon (🌿 or similar) next to branch names
- **Highlight**: Default branch marked with "default" badge

### API Changes Required

#### 1. New Endpoint: List Branches
**Route**: `/api/explorer/repos/[address]/[name]/branches`

```typescript
// GET /api/explorer/repos/[address]/[name]/branches
interface BranchesResponse {
  default_branch: string;
  branches: Array<{
    name: string;
    is_default: boolean;
    last_commit: {
      hash: string;
      timestamp: number;
      message: string;
    };
  }>;
}
```

#### 2. Enhanced Existing Endpoints
Add optional `branch` query parameter to existing endpoints:

**Modified Routes:**
- `/api/explorer/repos/[address]/[name]?branch=feature-branch`
- `/api/explorer/repos/[address]/[name]/tree/[...path]?branch=feature-branch`
- `/api/explorer/repos/[address]/[name]/commits?branch=feature-branch`
- `/api/explorer/repos/[address]/[name]/blob/[...path]?branch=feature-branch`

## Implementation Plan

### Phase 1: Backend Git Library Extensions

**File**: `/home/xiko/repobox/web/src/lib/git.ts`

#### New Functions
```typescript
// List all branches with metadata
export function getBranches(address: string, name: string): GitBranch[] {
  const repoPath = getRepoPath(address, name);
  try {
    // git for-each-ref --format='%(refname:short)|%(objectname)|%(committerdate:unix)|%(contents:subject)' refs/heads/
    const output = gitCommand(
      repoPath, 
      "for-each-ref --format='%(refname:short)|%(objectname)|%(committerdate:unix)|%(contents:subject)' refs/heads/"
    );
    if (!output) return [];
    
    return output.split('\n').map(line => {
      const [name, hash, timestamp, message] = line.split('|');
      return {
        name,
        is_default: false, // Set by getDefaultBranch comparison
        last_commit: {
          hash,
          timestamp: parseInt(timestamp),
          message
        }
      };
    });
  } catch {
    return [];
  }
}

// Check if branch exists
export function branchExists(address: string, name: string, branchName: string): boolean {
  const repoPath = getRepoPath(address, name);
  try {
    gitCommand(repoPath, `rev-parse --verify refs/heads/${branchName}`);
    return true;
  } catch {
    return false;
  }
}
```

#### Modified Functions
```typescript
// Add branch parameter to existing functions
export function getFileTree(
  address: string, 
  name: string, 
  path: string = '', 
  branch: string = 'HEAD'
): GitFileEntry[] {
  const repoPath = getRepoPath(address, name);
  try {
    // Validate branch first
    if (branch !== 'HEAD' && !branchExists(address, name, branch)) {
      throw new Error(`Branch '${branch}' does not exist`);
    }
    
    const ref = branch === 'HEAD' ? 'HEAD' : `refs/heads/${branch}`;
    const treePath = path ? `${ref}:${path}` : ref;
    const output = gitCommand(repoPath, `ls-tree ${treePath}`);
    // ... rest of implementation
  }
  // ... error handling
}

export function getCommitHistory(
  address: string, 
  name: string, 
  limit: number = 50,
  branch: string = 'HEAD'
): GitCommit[] {
  const repoPath = getRepoPath(address, name);
  try {
    const ref = branch === 'HEAD' ? 'HEAD' : `refs/heads/${branch}`;
    const output = gitCommand(repoPath, `log --format='%H|%an|%ae|%at|%s' -n ${limit} ${ref}`);
    // ... rest of implementation
  }
  // ... error handling
}

export function getFileContent(
  address: string, 
  name: string, 
  filePath: string,
  branch: string = 'HEAD'
): string | null {
  const repoPath = getRepoPath(address, name);
  try {
    const ref = branch === 'HEAD' ? 'HEAD' : `refs/heads/${branch}`;
    return gitCommand(repoPath, `show ${ref}:${filePath}`);
  } catch {
    return null;
  }
}

export function getReadmeContent(
  address: string, 
  name: string,
  branch: string = 'HEAD'
): string | null {
  const readmeFiles = ['README.md', 'readme.md', 'README', 'readme', 'README.txt'];
  
  for (const readmeFile of readmeFiles) {
    const content = getFileContent(address, name, readmeFile, branch);
    if (content) {
      return content;
    }
  }
  
  return null;
}
```

### Phase 2: API Endpoint Implementation

#### New Branches Endpoint
**File**: `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/branches/route.ts`

```typescript
import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getBranches, getDefaultBranch } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name } = await context.params;
    
    if (!address || !name) {
      return NextResponse.json(
        { error: 'Address and name are required' },
        { status: 400 }
      );
    }
    
    // Verify repo exists
    const repo = await runQueryOne('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name]);
    
    if (!repo) {
      return NextResponse.json(
        { error: 'Repository not found' },
        { status: 404 }
      );
    }
    
    try {
      const branches = getBranches(address, name);
      const defaultBranch = getDefaultBranch(address, name);
      
      // Mark default branch
      const branchesWithDefault = branches.map(branch => ({
        ...branch,
        is_default: branch.name === defaultBranch
      }));
      
      return NextResponse.json({
        default_branch: defaultBranch,
        branches: branchesWithDefault
      });
    } catch (gitError) {
      // Empty repo or no commits
      return NextResponse.json({
        default_branch: 'main',
        branches: []
      });
    }
  } catch (error) {
    console.error('Error fetching branches:', error);
    return NextResponse.json(
      { error: 'Failed to fetch branches' },
      { status: 500 }
    );
  }
}
```

#### Modify Existing Endpoints
Update existing API routes to accept `branch` query parameter:

**Files to modify:**
- `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/route.ts`
- `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/tree/[...path]/route.ts`
- `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/commits/route.ts`
- `/home/xiko/repobox/web/src/app/api/explorer/repos/[address]/[name]/blob/[...path]/route.ts`

**Example modification** for main repo route:
```typescript
export async function GET(request: NextRequest, context: RouteContext) {
  // ... existing validation code
  
  // Extract branch parameter
  const { searchParams } = new URL(request.url);
  const branch = searchParams.get('branch') || 'HEAD';
  
  // Validate branch if specified
  if (branch !== 'HEAD' && !branchExists(address, name, branch)) {
    return NextResponse.json(
      { error: `Branch '${branch}' does not exist` },
      { status: 404 }
    );
  }
  
  try {
    // Pass branch parameter to git functions
    const commitCount = getCommitCount(address, name, branch);
    const defaultBranch = getDefaultBranch(address, name);
    const fileTree = getFileTree(address, name, '', branch);
    const readmeContent = getReadmeContent(address, name, branch);
    
    return NextResponse.json({
      ...repo,
      commit_count: commitCount,
      default_branch: defaultBranch,
      current_branch: branch === 'HEAD' ? defaultBranch : branch,
      file_tree: fileTree,
      readme_content: readmeContent
    });
  } catch (gitError) {
    // ... error handling
  }
}
```

### Phase 3: Frontend Component Implementation

#### BranchSelector Component
**File**: `/home/xiko/repobox/web/src/components/BranchSelector.tsx`

```typescript
'use client';

import { useState, useEffect, useRef } from 'react';

interface Branch {
  name: string;
  is_default: boolean;
  last_commit: {
    hash: string;
    timestamp: number;
    message: string;
  };
}

interface BranchSelectorProps {
  branches: Branch[];
  currentBranch: string;
  defaultBranch: string;
  onChange: (branch: string) => void;
  disabled?: boolean;
}

export default function BranchSelector({ 
  branches, 
  currentBranch, 
  defaultBranch, 
  onChange,
  disabled = false 
}: BranchSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [filter, setFilter] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Filter branches based on search
  const filteredBranches = branches.filter(branch =>
    branch.name.toLowerCase().includes(filter.toLowerCase())
  );

  const handleBranchSelect = (branchName: string) => {
    onChange(branchName);
    setIsOpen(false);
    setFilter('');
  };

  const currentBranchData = branches.find(b => b.name === currentBranch);

  return (
    <div className="branch-selector" ref={dropdownRef}>
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        className={`branch-selector-trigger ${disabled ? 'disabled' : ''}`}
        disabled={disabled}
      >
        <span className="branch-icon">🌿</span>
        <span className="branch-name">{currentBranch}</span>
        {currentBranchData?.is_default && (
          <span className="branch-default-badge">default</span>
        )}
        <span className={`branch-arrow ${isOpen ? 'open' : ''}`}>▼</span>
      </button>

      {isOpen && (
        <div className="branch-dropdown">
          {branches.length > 10 && (
            <div className="branch-filter">
              <input
                type="text"
                placeholder="Filter branches..."
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                className="branch-filter-input"
              />
            </div>
          )}
          
          <div className="branch-list">
            {filteredBranches.length === 0 ? (
              <div className="branch-empty">No branches found</div>
            ) : (
              filteredBranches.map((branch) => (
                <button
                  key={branch.name}
                  onClick={() => handleBranchSelect(branch.name)}
                  className={`branch-item ${branch.name === currentBranch ? 'current' : ''}`}
                >
                  <div className="branch-item-info">
                    <span className="branch-item-name">
                      {branch.name === currentBranch && <span className="check">✓ </span>}
                      {branch.name}
                    </span>
                    {branch.is_default && (
                      <span className="branch-item-badge">default</span>
                    )}
                  </div>
                  <div className="branch-item-meta">
                    <span className="branch-last-commit">
                      {branch.last_commit.message.substring(0, 50)}
                      {branch.last_commit.message.length > 50 && '...'}
                    </span>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
```

#### CSS Styles
**File**: `/home/xiko/repobox/web/src/app/globals.css` (additions)

```css
/* Branch Selector Styles */
.branch-selector {
  position: relative;
  display: inline-block;
}

.branch-selector-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: transparent;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: monospace;
  font-size: 14px;
}

.branch-selector-trigger:hover:not(.disabled) {
  border-color: #d1d5db;
  background: #f9fafb;
}

.branch-selector-trigger.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.branch-icon {
  font-size: 12px;
}

.branch-name {
  font-weight: 600;
}

.branch-default-badge {
  background: #10b981;
  color: white;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
  font-weight: 600;
}

.branch-arrow {
  font-size: 10px;
  transition: transform 0.2s ease;
}

.branch-arrow.open {
  transform: rotate(180deg);
}

.branch-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  margin-top: 4px;
  max-height: 300px;
  overflow: hidden;
}

.branch-filter {
  padding: 12px;
  border-bottom: 1px solid #e5e7eb;
}

.branch-filter-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 4px;
  font-size: 14px;
}

.branch-list {
  max-height: 250px;
  overflow-y: auto;
}

.branch-item {
  display: block;
  width: 100%;
  padding: 12px;
  border: none;
  background: none;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.2s ease;
  border-bottom: 1px solid #f3f4f6;
}

.branch-item:hover {
  background: #f9fafb;
}

.branch-item.current {
  background: #eff6ff;
  border-color: #dbeafe;
}

.branch-item:last-child {
  border-bottom: none;
}

.branch-item-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.branch-item-name {
  font-weight: 600;
  font-family: monospace;
}

.check {
  color: #10b981;
  font-weight: bold;
}

.branch-item-badge {
  background: #f3f4f6;
  color: #6b7280;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
  font-weight: 600;
}

.branch-item-meta {
  font-size: 12px;
  color: #6b7280;
}

.branch-last-commit {
  font-family: system-ui;
}

.branch-empty {
  padding: 12px;
  text-align: center;
  color: #6b7280;
  font-style: italic;
}

/* Stat item modifications */
.explore-stat-branch .explore-stat-value {
  display: flex;
  align-items: center;
}
```

### Phase 4: Integration with Repo Page

#### Modified RepoPage Component
**File**: `/home/xiko/repobox/web/src/app/explore/[address]/[name]/page.tsx`

**Key changes:**

1. **State Management**
```typescript
// Add new state variables
const [selectedBranch, setSelectedBranch] = useState<string>('HEAD');
const [branches, setBranches] = useState<Branch[]>([]);
const [branchLoading, setBranchLoading] = useState(false);
```

2. **Fetch Branches**
```typescript
// Add to useEffect
useEffect(() => {
  if (!address || !name) return;
  
  const fetchData = async () => {
    try {
      const [repoRes, commitsRes, branchesRes] = await Promise.all([
        fetch(`/api/explorer/repos/${address}/${name}${selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : ''}`),
        fetch(`/api/explorer/repos/${address}/${name}/commits?limit=30${selectedBranch !== 'HEAD' ? `&branch=${selectedBranch}` : ''}`),
        fetch(`/api/explorer/repos/${address}/${name}/branches`)
      ]);
      
      // ... handle responses
      
      if (branchesRes.ok) {
        const branchData = await branchesRes.json();
        setBranches(branchData.branches || []);
      }
    } catch (error) {
      // ... error handling
    }
  };
  
  fetchData();
}, [address, name, selectedBranch]);
```

3. **Branch Change Handler**
```typescript
const handleBranchChange = async (newBranch: string) => {
  setBranchLoading(true);
  setSelectedBranch(newBranch);
  
  try {
    // Update URL without page reload (optional)
    const url = new URL(window.location.href);
    if (newBranch !== repo?.default_branch) {
      url.searchParams.set('branch', newBranch);
    } else {
      url.searchParams.delete('branch');
    }
    window.history.replaceState({}, '', url.toString());
    
    // Fetch updated data will happen via useEffect dependency
  } catch (error) {
    console.error('Error switching branch:', error);
    // Revert to previous branch on error
    setSelectedBranch(repo?.default_branch || 'main');
  } finally {
    setBranchLoading(false);
  }
};
```

4. **Update Repository Stats Section**
```tsx
<div className="explore-repo-stats">
  <div className="explore-stat-item">
    <span className="explore-stat-value">{repo.commit_count}</span>
    <span className="explore-stat-label">COMMITS</span>
  </div>
  <div className="explore-stat-item explore-stat-branch">
    {branches.length > 0 ? (
      <BranchSelector
        branches={branches}
        currentBranch={selectedBranch === 'HEAD' ? repo.default_branch : selectedBranch}
        defaultBranch={repo.default_branch}
        onChange={handleBranchChange}
        disabled={branchLoading}
      />
    ) : (
      <span className="explore-stat-value">{repo.default_branch}</span>
    )}
    <span className="explore-stat-label">BRANCH</span>
  </div>
</div>
```

5. **Update Navigation Functions**
```typescript
const navigateToPath = async (path: string) => {
  if (!address || !name) return;
  const branchParam = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
  const res = await fetch(`/api/explorer/repos/${address}/${name}/tree/${path}${branchParam}`);
  // ... rest of implementation
};

const viewFile = async (filePath: string) => {
  if (!address || !name) return;
  const branchParam = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
  const res = await fetch(`/api/explorer/repos/${address}/${name}/blob/${filePath}${branchParam}`);
  // ... rest of implementation
};
```

## Edge Cases and Error Handling

### Repository Edge Cases

1. **Empty Repository**
   - No branches exist
   - Handle gracefully with disabled branch selector
   - Show appropriate empty state message

2. **Single Branch Repository**
   - Only default branch exists
   - Show branch selector but disable dropdown
   - Display "Only one branch" tooltip

3. **Corrupted Repository**
   - Git operations fail
   - Fallback to default branch
   - Show error message to user

4. **Large Repository**
   - Many branches (>100)
   - Implement search/filter in branch dropdown
   - Paginate branch list if necessary

### Branch-Specific Edge Cases

1. **Branch Doesn't Exist**
   - User manually edits URL with invalid branch
   - Redirect to default branch
   - Show error notification

2. **Empty Branch**
   - Branch exists but has no content
   - Show empty state for file tree, commits
   - Disable README tab if no README

3. **Deleted Branch**
   - Branch was deleted after page load
   - Handle API 404 gracefully
   - Refresh branch list and redirect to default

### Content Edge Cases

1. **Missing Files in Branch**
   - File exists in default but not selected branch
   - Show "File not found in this branch"
   - Provide option to switch to branch containing file

2. **Path Preservation**
   - User is viewing deep path when switching branches
   - Check if path exists in new branch
   - Navigate to root if path doesn't exist
   - Show notification about path change

3. **Large File Trees**
   - Branch has many files/directories
   - Implement virtual scrolling if needed
   - Show loading states for large operations

### Performance Edge Cases

1. **Slow Git Operations**
   - Show loading indicators
   - Implement request cancellation
   - Timeout handling for git commands

2. **Concurrent Branch Switches**
   - Debounce rapid branch changes
   - Cancel previous requests
   - Show loading state during transitions

## Testing Approach

### Unit Tests

#### Git Library Functions
**Test File**: `/home/xiko/repobox/web/src/lib/__tests__/git.test.ts`

```typescript
describe('Git Library - Branch Support', () => {
  test('getBranches returns list of branches', () => {
    // Mock git command output
    // Test branch parsing
    // Verify return format
  });
  
  test('getFileTree works with branch parameter', () => {
    // Test with different branches
    // Verify correct git ref is used
    // Test branch validation
  });
  
  test('branchExists validates branch correctly', () => {
    // Test existing and non-existing branches
    // Test invalid branch names
    // Test empty repository
  });
  
  test('getCommitHistory filters by branch', () => {
    // Test branch-specific commit history
    // Compare with default branch
    // Test merge commit handling
  });
});
```

#### BranchSelector Component
**Test File**: `/home/xiko/repobox/web/src/components/__tests__/BranchSelector.test.tsx`

```typescript
describe('BranchSelector Component', () => {
  test('renders branch list correctly', () => {
    // Test branch display
    // Test default branch marking
    // Test current branch highlighting
  });
  
  test('handles branch selection', () => {
    // Test onClick handlers
    // Verify onChange callback
    // Test dropdown closing
  });
  
  test('filters branches with search', () => {
    // Test filter functionality
    // Test case insensitive search
    // Test empty results
  });
  
  test('handles edge cases', () => {
    // Empty branch list
    // Single branch
    // Disabled state
  });
});
```

### Integration Tests

#### API Endpoints
**Test File**: `/home/xiko/repobox/web/src/app/api/explorer/__tests__/branches.test.ts`

```typescript
describe('Branches API', () => {
  test('GET /api/explorer/repos/[address]/[name]/branches', () => {
    // Test successful response
    // Test repository not found
    // Test empty repository
    // Test git errors
  });
  
  test('Branch parameter in existing endpoints', () => {
    // Test repo endpoint with branch param
    // Test tree endpoint with branch param
    // Test commits endpoint with branch param
    // Test blob endpoint with branch param
  });
});
```

#### End-to-End Tests
**Test File**: `/home/xiko/repobox/web/cypress/e2e/branch-selector.cy.ts`

```typescript
describe('Branch Selector E2E', () => {
  test('Full branch switching workflow', () => {
    // Visit repository page
    // Open branch selector
    // Select different branch
    // Verify content updates
    // Test file navigation
    // Test URL updates
  });
  
  test('Error handling', () => {
    // Test invalid branch in URL
    // Test network errors
    // Test git operation failures
  });
  
  test('Performance', () => {
    // Test rapid branch switching
    // Test large repositories
    // Test loading states
  });
});
```

### Manual Testing Checklist

#### Functionality
- [ ] Branch selector appears in repository stats
- [ ] Clicking selector opens dropdown with all branches
- [ ] Current branch is highlighted in dropdown
- [ ] Default branch is marked with badge
- [ ] Selecting branch updates all content areas
- [ ] URL updates to reflect selected branch (optional)
- [ ] File tree updates to show branch content
- [ ] Commit history filters to branch commits
- [ ] README loads from selected branch
- [ ] File viewer shows files from selected branch

#### Edge Cases
- [ ] Empty repository (no branches)
- [ ] Single branch repository
- [ ] Invalid branch in URL redirects properly
- [ ] Missing files handled gracefully
- [ ] Path preservation works correctly
- [ ] Large repositories perform well
- [ ] Network errors handled appropriately

#### Browser Compatibility
- [ ] Chrome/Chromium
- [ ] Firefox
- [ ] Safari
- [ ] Mobile browsers

#### Performance
- [ ] Branch switching is responsive (<500ms)
- [ ] No memory leaks with rapid switching
- [ ] Large file trees load efficiently
- [ ] Loading states provide feedback

## Deployment Considerations

### Database Migrations
- No database schema changes required
- Existing repositories work without modification

### Backward Compatibility
- Existing URLs continue to work (show default branch)
- API endpoints maintain existing response format
- Branch parameter is optional in all endpoints

### Performance Impact
- Minimal impact on existing functionality
- Additional git operations only when branches are accessed
- Branch list cached after first load

### Security Considerations
- Branch validation prevents directory traversal
- Existing repository permissions apply to all branches
- No additional authentication requirements

## Future Enhancements

### Phase 2 Features

1. **URL Integration**
   - Branch parameter in URL: `/explore/address/repo?branch=feature`
   - Deep linking to specific branch + path
   - Browser history integration

2. **Branch Metadata**
   - Last commit timestamp in dropdown
   - Ahead/behind commit counts vs default
   - Protection status indicators

3. **Branch Operations**
   - Create branch UI (for authorized users)
   - Branch comparison view
   - Merge request integration

4. **Performance Optimizations**
   - Branch list caching
   - Lazy loading of branch metadata
   - Virtual scrolling for large branch lists

### Advanced Features

1. **Search and Filtering**
   - Search across all files in all branches
   - Branch-specific search results
   - File history across branches

2. **Visual Enhancements**
   - Branch graph visualization
   - Commit timeline view
   - Diff view between branches

3. **Collaboration Features**
   - Branch subscriptions/notifications
   - Branch-specific discussions
   - Review status indicators

## Success Metrics

### Functional Metrics
- ✅ All test cases pass
- ✅ No regressions in existing functionality
- ✅ Performance requirements met (<500ms branch switch)

### User Experience Metrics
- Branch selector discoverable and intuitive
- Content updates feel seamless
- Error states are clear and actionable
- Loading states provide appropriate feedback

### Technical Metrics
- Code coverage >90% for new components
- No memory leaks detected
- Git operation efficiency maintained
- API response times within SLA

## Conclusion

This specification provides a comprehensive plan for implementing branch selector functionality in the repo.box explorer interface. The implementation is designed to be:

1. **Non-disruptive**: Maintains backward compatibility
2. **Performance-conscious**: Minimal impact on existing operations
3. **User-friendly**: Intuitive interface with proper error handling
4. **Extensible**: Foundation for future branch-related features
5. **Well-tested**: Comprehensive testing strategy

The phased implementation approach allows for incremental delivery and validation of each component before integration.