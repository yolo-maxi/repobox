# Explorer: Show Signer Address Per Commit

**Feature Specification**
**Status:** Implementation Ready  
**Author:** PM Agent  
**Created:** 2024-03-21  

## Overview

Currently, the repo.box explorer displays the repository owner address for all commits. This specification implements displaying the actual EVM address that signed each individual commit, allowing users to see which agent or team member created each commit.

## Current State Analysis

### Database Schema
The current database has:
- `repos` table: stores repository metadata including `owner_address`
- `push_log` table: stores push events but lacks commit-level signer information
- No dedicated commit storage with signer data

### Current Git Integration
- `getCommitHistory()` in `/web/src/lib/git.ts` uses format `%H|%an|%ae|%at|%s`
- Retrieves: hash, author name, author email, timestamp, message
- Missing: signature verification and signer address extraction

### Current Data Flow
```
Git Repository → git log command → GitCommit object → UI display
```

## Implementation Plan

### 1. Files to Modify

#### Backend/API Changes
- `/web/src/lib/git.ts` - Add signer address extraction
- `/web/src/app/api/explorer/repos/[address]/[name]/commits/route.ts` - Return signer data
- `/web/src/lib/database.ts` - Add commit storage functions

#### Frontend Changes
- `/web/src/app/explore/[address]/[name]/page.tsx` - Display signer addresses
- Create new commit display component with signer information

### 2. API Changes

#### Modified GitCommit Interface
```typescript
interface GitCommit {
  hash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
  signerAddress?: string;  // NEW: EVM address that signed the commit
  signatureValid?: boolean; // NEW: signature verification status
  ownerAddress: string;     // NEW: repository owner for comparison
}
```

#### New API Response Format
```json
{
  "commits": [
    {
      "hash": "abc123...",
      "author": "Agent Name", 
      "email": "agent@example.com",
      "timestamp": 1710000000,
      "message": "Add new feature",
      "signerAddress": "0x742d35Cc6635C0532925a3b8D093C7C85EC54C6",
      "signatureValid": true,
      "ownerAddress": "0x123...456"
    }
  ]
}
```

### 3. Database Schema Changes

#### New Table: commits_cache
```sql
CREATE TABLE IF NOT EXISTS commits_cache (
    repo_address TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    signer_address TEXT,
    signature_valid BOOLEAN,
    verified_at TEXT,
    PRIMARY KEY(repo_address, repo_name, commit_hash)
);

CREATE INDEX idx_commits_cache_repo ON commits_cache(repo_address, repo_name);
CREATE INDEX idx_commits_cache_signer ON commits_cache(signer_address);
```

**Rationale:** Cache signature verification results to avoid expensive cryptographic operations on each request.

### 4. Git Integration: Signer Address Extraction

#### New Function: `extractSignerAddress()`
```typescript
export function extractSignerAddress(
  address: string, 
  name: string, 
  commitHash: string
): { signerAddress?: string, signatureValid: boolean } {
  const repoPath = getRepoPath(address, name);
  
  try {
    // Get commit signature
    const sigOutput = gitCommand(repoPath, `cat-file commit ${commitHash}`);
    
    // Extract GPG signature block
    const gpgSigMatch = sigOutput.match(/-----BEGIN PGP SIGNATURE-----[\s\S]*?-----END PGP SIGNATURE-----/);
    if (!gpgSigMatch) {
      return { signatureValid: false };
    }
    
    // Verify signature using repobox CLI
    const repoboxPath = "/home/xiko/repobox/target/release/repobox";
    const verifyResult = execSync(
      `echo '${gpgSigMatch[0]}' | ${repoboxPath} verify-commit ${commitHash}`,
      { cwd: repoPath, encoding: 'utf-8' }
    );
    
    // Parse verification result for EVM address
    const addressMatch = verifyResult.match(/0x[a-fA-F0-9]{40}/);
    if (addressMatch) {
      return {
        signerAddress: addressMatch[0],
        signatureValid: true
      };
    }
    
    return { signatureValid: false };
  } catch (error) {
    console.warn(`Failed to extract signer for commit ${commitHash}:`, error);
    return { signatureValid: false };
  }
}
```

#### Updated `getCommitHistory()`
```typescript
export function getCommitHistory(address: string, name: string, limit: number = 50): GitCommit[] {
  const repoPath = getRepoPath(address, name);
  const repo = getRepo(address, name); // Get repo metadata for owner
  
  try {
    const output = gitCommand(repoPath, `log --format='%H|%an|%ae|%at|%s' -n ${limit}`);
    if (!output) return [];
    
    return output.split('\n').map(line => {
      const [hash, author, email, timestamp, message] = line.split('|');
      
      // Try to get cached signer data first
      let signerData = getCachedSignerData(address, name, hash);
      if (!signerData) {
        // Extract and cache signer address
        signerData = extractSignerAddress(address, name, hash);
        cacheSignerData(address, name, hash, signerData);
      }
      
      return {
        hash,
        author,
        email,
        timestamp: parseInt(timestamp),
        message,
        ownerAddress: repo?.owner_address || address,
        ...signerData
      };
    });
  } catch {
    return [];
  }
}
```

### 5. UI/UX Changes

#### Enhanced Commit Display Component
```tsx
// components/CommitDisplay.tsx
interface CommitDisplayProps {
  commit: GitCommit;
}

export function CommitDisplay({ commit }: CommitDisplayProps) {
  const isOwnerCommit = commit.signerAddress === commit.ownerAddress;
  const hasValidSignature = commit.signatureValid;
  
  return (
    <div className="commit-item">
      <div className="commit-header">
        <span className="commit-hash">{commit.hash.substring(0, 7)}</span>
        <span className="commit-message">{commit.message}</span>
      </div>
      
      <div className="commit-metadata">
        <div className="author-info">
          <span>Author: {commit.author}</span>
        </div>
        
        <div className="signer-info">
          {hasValidSignature ? (
            <div className={`signer-address ${isOwnerCommit ? 'owner' : 'collaborator'}`}>
              <span className="signer-label">
                Signed by: {isOwnerCommit ? 'Owner' : 'Collaborator'}
              </span>
              <span className="address" title={commit.signerAddress}>
                {formatAddress(commit.signerAddress)}
              </span>
              <span className="signature-status verified">✓ Verified</span>
            </div>
          ) : (
            <div className="signer-address unsigned">
              <span className="signature-status">⚠ Unsigned</span>
            </div>
          )}
        </div>
        
        <div className="timestamp">
          {formatTimestamp(commit.timestamp)}
        </div>
      </div>
    </div>
  );
}

function formatAddress(address?: string): string {
  if (!address) return 'Unknown';
  return `${address.substring(0, 6)}...${address.substring(38)}`;
}
```

#### Updated Explorer Page
```tsx
// app/explore/[address]/[name]/page.tsx
export default function ExplorePage({ params }: { params: { address: string, name: string } }) {
  // ... existing code ...
  
  return (
    <div className="repository-explorer">
      {/* ... existing sections ... */}
      
      <section className="recent-commits">
        <h3>Recent Commits</h3>
        {commits.map(commit => (
          <CommitDisplay key={commit.hash} commit={commit} />
        ))}
      </section>
    </div>
  );
}
```

### 6. Error Handling

#### Scenarios to Handle
1. **Unsigned commits**: Display "Unsigned" status clearly
2. **Invalid signatures**: Show "Invalid signature" warning
3. **Unknown signer**: Display "Unknown signer" for unrecognized addresses
4. **Verification failures**: Graceful degradation with logging

#### Error States Display
```tsx
function SignatureStatus({ commit }: { commit: GitCommit }) {
  if (!commit.signatureValid) {
    return (
      <div className="signature-status warning">
        <Icon name="warning" />
        <span>Unsigned or Invalid Signature</span>
      </div>
    );
  }
  
  if (!commit.signerAddress) {
    return (
      <div className="signature-status unknown">
        <Icon name="question" />
        <span>Unknown Signer</span>
      </div>
    );
  }
  
  return (
    <div className="signature-status verified">
      <Icon name="verified" />
      <span>Verified Signature</span>
    </div>
  );
}
```

### 7. Performance Considerations

#### Caching Strategy
- Cache signature verification results in database
- Implement TTL for cache entries (e.g., 24 hours)
- Background job to pre-verify signatures for active repositories

#### Optimization Techniques
```typescript
// Batch verification for multiple commits
export function batchVerifyCommits(
  address: string, 
  name: string, 
  commitHashes: string[]
): Record<string, { signerAddress?: string, signatureValid: boolean }> {
  // Implementation that processes multiple commits efficiently
}

// Lazy loading for commit details
export function getCommitHistoryWithPagination(
  address: string, 
  name: string, 
  page: number = 0, 
  limit: number = 20
): { commits: GitCommit[], hasMore: boolean } {
  // Implement pagination to avoid loading too many commits at once
}
```

#### Database Query Optimization
```sql
-- Efficient query for recent commits with signer data
SELECT 
  c.commit_hash,
  c.signer_address,
  c.signature_valid,
  r.owner_address
FROM commits_cache c
JOIN repos r ON c.repo_address = r.address AND c.repo_name = r.name
WHERE c.repo_address = ? AND c.repo_name = ?
ORDER BY c.verified_at DESC
LIMIT ?;
```

### 8. Testing Strategy

#### Unit Tests
```typescript
// tests/git.test.ts
describe('extractSignerAddress', () => {
  test('should extract valid signer address from signed commit', () => {
    const result = extractSignerAddress('0x123...', 'test-repo', 'abc123...');
    expect(result.signerAddress).toMatch(/^0x[a-fA-F0-9]{40}$/);
    expect(result.signatureValid).toBe(true);
  });
  
  test('should handle unsigned commits gracefully', () => {
    const result = extractSignerAddress('0x123...', 'test-repo', 'unsigned-commit');
    expect(result.signatureValid).toBe(false);
    expect(result.signerAddress).toBeUndefined();
  });
});
```

#### Integration Tests
```typescript
// tests/api/commits.test.ts
describe('/api/explorer/repos/[address]/[name]/commits', () => {
  test('should return commits with signer addresses', async () => {
    const response = await fetch('/api/explorer/repos/0x123.../test-repo/commits');
    const data = await response.json();
    
    expect(data.commits).toHaveLength(10);
    expect(data.commits[0]).toHaveProperty('signerAddress');
    expect(data.commits[0]).toHaveProperty('signatureValid');
  });
});
```

#### Manual Testing Scenarios
1. **Repository with mixed signers**: Test with commits from owner and collaborators
2. **Unsigned repository**: Test with traditional git commits (no signatures)
3. **Invalid signatures**: Test with corrupted signature data
4. **Performance**: Test with repositories having 1000+ commits

## Implementation Priority

### Phase 1: Core Functionality (High Priority)
- [ ] Implement `extractSignerAddress()` function
- [ ] Create `commits_cache` table
- [ ] Update `getCommitHistory()` to include signer data
- [ ] Modify commits API endpoint

### Phase 2: UI Enhancement (High Priority)
- [ ] Create `CommitDisplay` component
- [ ] Update explorer page to show signer addresses
- [ ] Add signature verification status indicators

### Phase 3: Performance & UX (Medium Priority)
- [ ] Implement caching layer
- [ ] Add pagination for commit history
- [ ] Batch verification optimization

### Phase 4: Polish & Testing (Medium Priority)
- [ ] Comprehensive error handling
- [ ] Unit and integration tests
- [ ] Performance monitoring

## Security Considerations

1. **Signature Verification**: Always verify signatures using the repobox CLI tool
2. **Input Validation**: Sanitize all git command inputs to prevent injection
3. **Rate Limiting**: Limit signature verification requests to prevent DoS
4. **Cache Security**: Ensure cache integrity and prevent poisoning attacks

## Rollback Plan

If issues arise:
1. **Immediate**: Feature flag to disable signer display, fall back to owner address
2. **Database**: `commits_cache` table can be dropped without affecting core functionality  
3. **API**: Backwards compatible - new fields are optional

## Success Metrics

1. **Functionality**: 100% of signed commits show correct signer address
2. **Performance**: Commit list loads within 2 seconds for repositories with 100+ commits
3. **Accuracy**: <1% false positive/negative rate for signature verification
4. **User Experience**: Clear visual distinction between owner and collaborator commits

## Timeline Estimate

- **Phase 1**: 2-3 days
- **Phase 2**: 1-2 days  
- **Phase 3**: 1-2 days
- **Phase 4**: 1 day
- **Total**: 5-8 days

This specification provides a comprehensive implementation plan for displaying signer addresses per commit in the repo.box explorer, enabling users to track which specific EVM address signed each commit rather than just showing the repository owner.