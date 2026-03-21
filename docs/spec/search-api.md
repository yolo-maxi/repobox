# Technical Specification: Global Search API

**Version:** 1.0  
**Author:** PM Agent  
**Date:** 2026-03-21  
**Status:** Draft  

## Overview

This specification defines the implementation of a global search feature for repo.box that allows users to search across repository names, owner addresses, and commit messages. The current explorer has client-side filtering for repository names and addresses, but lacks server-side full-text search capabilities for commit content.

## Current State Analysis

### Existing Architecture
- **Database**: SQLite with `repos` and `push_log` tables
- **API Layer**: Next.js API routes in `/api/explorer/`
- **Frontend**: React components with client-side filtering
- **Data Access**: Direct git command execution via `lib/git.ts`

### Current Search Implementation
```typescript
// Client-side filtering in explore/page.tsx
const filteredRepos = repos.filter(repo =>
  repo.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
  repo.address.toLowerCase().includes(searchTerm.toLowerCase())
);
```

### Limitations
1. No commit message searching
2. No full-text search ranking
3. Performance issues with large repositories
4. No search result pagination
5. Limited to exact substring matching

## Requirements

### Functional Requirements
1. **FR1**: Search across repository names (case-insensitive)
2. **FR2**: Search across owner addresses (partial matching)
3. **FR3**: Search across commit messages (full-text search)
4. **FR4**: Return ranked search results
5. **FR5**: Support pagination for large result sets
6. **FR6**: Maintain sub-200ms response times for typical queries
7. **FR7**: Graceful degradation when git repositories are unavailable

### Non-Functional Requirements
1. **NFR1**: Maximum 500ms response time for 95th percentile queries
2. **NFR2**: Support for 1000+ repositories in the index
3. **NFR3**: Index updates within 30 seconds of new commits
4. **NFR4**: Backward compatibility with existing API endpoints

## API Design

### New Endpoints

#### Search API Endpoint
```
GET /api/explorer/search
```

**Query Parameters:**
- `q` (required): Search query string (min: 2 characters, max: 100)
- `limit` (optional): Number of results to return (default: 20, max: 100)
- `offset` (optional): Pagination offset (default: 0)
- `type` (optional): Filter by result type (`repo|commit|all`, default: `all`)
- `sort` (optional): Sort order (`relevance|date|activity`, default: `relevance`)

**Response Format:**
```typescript
interface SearchResponse {
  query: string;
  results: SearchResult[];
  pagination: {
    total: number;
    limit: number;
    offset: number;
    hasMore: boolean;
  };
  searchTime: number; // milliseconds
  filters: {
    type: string;
    sort: string;
  };
}

interface SearchResult {
  type: 'repo' | 'commit';
  id: string;
  score: number; // relevance score 0-100
  
  // For repo results
  repo?: {
    address: string;
    name: string;
    owner_address: string;
    description?: string;
    commit_count: number;
    last_commit_date: string | null;
    created_at: string;
  };
  
  // For commit results  
  commit?: {
    hash: string;
    short_hash: string;
    message: string;
    author: string;
    timestamp: number;
    repo_address: string;
    repo_name: string;
    highlighted_message?: string; // message with search terms highlighted
  };
}
```

**Example Request:**
```bash
GET /api/explorer/search?q=superfluid&limit=10&type=all&sort=relevance
```

**Example Response:**
```json
{
  "query": "superfluid",
  "results": [
    {
      "type": "repo",
      "id": "0x123...abc/superfluid-core",
      "score": 95,
      "repo": {
        "address": "0x123...abc",
        "name": "superfluid-core",
        "owner_address": "0x123...abc",
        "description": "Superfluid Protocol core contracts",
        "commit_count": 142,
        "last_commit_date": "2026-03-20T15:30:00Z",
        "created_at": "2026-01-15T10:00:00Z"
      }
    },
    {
      "type": "commit",
      "id": "0x456...def/token-contracts:7a8b9c0d",
      "score": 78,
      "commit": {
        "hash": "7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b",
        "short_hash": "7a8b9c0",
        "message": "Add Superfluid integration for streaming payments",
        "author": "Alice Developer",
        "timestamp": 1711123456,
        "repo_address": "0x456...def", 
        "repo_name": "token-contracts",
        "highlighted_message": "Add <mark>Superfluid</mark> integration for streaming payments"
      }
    }
  ],
  "pagination": {
    "total": 15,
    "limit": 10,
    "offset": 0,
    "hasMore": true
  },
  "searchTime": 45,
  "filters": {
    "type": "all",
    "sort": "relevance"
  }
}
```

### Search Suggestions API (Future Enhancement)
```
GET /api/explorer/search/suggestions?q={partial_query}
```

## Database Schema Changes

### New Tables

#### Search Index Table
```sql
CREATE TABLE search_index (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  type TEXT NOT NULL CHECK (type IN ('repo', 'commit')),
  entity_id TEXT NOT NULL, -- repo: "address/name", commit: "address/name:hash"
  content TEXT NOT NULL, -- searchable content
  metadata TEXT, -- JSON with additional fields
  indexed_at TEXT NOT NULL,
  UNIQUE(type, entity_id)
);

-- Full-text search index
CREATE VIRTUAL TABLE search_index_fts USING fts5(
  content,
  metadata,
  content='search_index',
  content_rowid='id'
);

-- Performance indexes
CREATE INDEX idx_search_index_type ON search_index(type);
CREATE INDEX idx_search_index_indexed_at ON search_index(indexed_at DESC);
```

#### Search Analytics Table (Optional)
```sql
CREATE TABLE search_analytics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  query TEXT NOT NULL,
  result_count INTEGER NOT NULL,
  response_time_ms INTEGER NOT NULL,
  searched_at TEXT NOT NULL,
  user_agent TEXT,
  ip_hash TEXT -- hashed IP for privacy
);

CREATE INDEX idx_search_analytics_searched_at ON search_analytics(searched_at DESC);
```

### Modified Tables

No changes to existing `repos` and `push_log` tables. The search index will be populated from existing data.

## Indexing Strategy

### Repository Indexing
```typescript
interface RepoIndexEntry {
  type: 'repo';
  entity_id: string; // "address/name"
  content: string; // "name description owner_address"
  metadata: {
    address: string;
    name: string;
    owner_address: string;
    description?: string;
    commit_count: number;
    last_commit_date: string | null;
    created_at: string;
  };
}
```

### Commit Indexing
```typescript
interface CommitIndexEntry {
  type: 'commit';
  entity_id: string; // "address/name:hash"
  content: string; // "message author"
  metadata: {
    hash: string;
    short_hash: string;
    message: string;
    author: string;
    email: string;
    timestamp: number;
    repo_address: string;
    repo_name: string;
  };
}
```

### Index Population Process

1. **Initial Population**:
   ```typescript
   async function populateSearchIndex() {
     // Index all repositories
     const repos = await runQuery<Repo>('SELECT * FROM repos');
     for (const repo of repos) {
       await indexRepository(repo);
     }
     
     // Index recent commits (last 1000 per repo to start)
     for (const repo of repos) {
       await indexRecentCommits(repo.address, repo.name, 1000);
     }
   }
   ```

2. **Incremental Updates**:
   - Hook into push events in `routes.rs` to trigger index updates
   - Add webhook to update index when new repositories are created
   - Run periodic cleanup to remove orphaned entries

## Search Algorithm

### Ranking Factors

1. **Content Match Quality** (40% weight):
   - Exact phrase matches: +20 points
   - Word boundary matches: +15 points
   - Prefix matches: +10 points
   - Partial matches: +5 points

2. **Repository Activity** (30% weight):
   - Recent commits (< 30 days): +15 points
   - High commit count (>100): +10 points
   - Recent creation: +5 points

3. **Match Location** (20% weight):
   - Repository name match: +20 points
   - Repository description match: +15 points
   - Commit message match: +10 points
   - Author name match: +5 points

4. **Repository Quality** (10% weight):
   - Has README: +5 points
   - Multiple contributors: +3 points
   - Active development: +2 points

### Search Query Processing

```typescript
function processSearchQuery(query: string): SearchTerms {
  // Normalize query
  const normalized = query.trim().toLowerCase();
  
  // Split into terms
  const terms = normalized.split(/\s+/).filter(term => term.length >= 2);
  
  // Build FTS5 query
  const ftsQuery = terms.map(term => {
    if (term.length >= 3) {
      return `"${term}"* OR ${term}*`; // Exact phrase + prefix matching
    }
    return `"${term}"`; // Exact matching for short terms
  }).join(' AND ');
  
  return { normalized, terms, ftsQuery };
}
```

## Implementation Plan

### Phase 1: Core Search Infrastructure (Week 1)

1. **Database Schema Setup**:
   - Create search index tables
   - Set up FTS5 virtual table
   - Add database migration script

2. **Indexing Service**:
   - Implement repository indexing
   - Implement commit indexing
   - Create initial population script

3. **Basic Search API**:
   - Create `/api/explorer/search` endpoint
   - Implement basic FTS5 querying
   - Add response formatting

### Phase 2: Search Quality & Performance (Week 2)

1. **Ranking Algorithm**:
   - Implement scoring system
   - Add relevance-based sorting
   - Optimize query performance

2. **Index Management**:
   - Implement incremental updates
   - Add cleanup procedures
   - Monitor index health

3. **API Enhancements**:
   - Add pagination support
   - Implement result type filtering
   - Add search time metrics

### Phase 3: Frontend Integration (Week 3)

1. **UI Components**:
   - Enhanced search bar with suggestions
   - Search results page
   - Result highlighting

2. **User Experience**:
   - Real-time search suggestions
   - Search history
   - Advanced filters

### Phase 4: Optimization & Analytics (Week 4)

1. **Performance Tuning**:
   - Query optimization
   - Caching strategies
   - Index maintenance automation

2. **Analytics & Monitoring**:
   - Search analytics collection
   - Performance monitoring
   - Usage insights

## File Structure

```
/home/xiko/repobox/
├── repobox-server/src/
│   ├── search.rs              # Search service implementation
│   └── routes.rs              # Updated with search endpoints
├── web/src/
│   ├── app/api/explorer/
│   │   └── search/
│   │       ├── route.ts       # Main search API endpoint
│   │       └── suggestions/
│   │           └── route.ts   # Search suggestions API
│   ├── lib/
│   │   ├── search.ts          # Search utilities and types
│   │   └── indexing.ts        # Index management utilities
│   └── components/explorer/
│       ├── SearchBar.tsx      # Enhanced search component
│       └── SearchResults.tsx  # Search results display
├── scripts/
│   ├── populate-search-index.sh  # Initial index population
│   └── maintain-search-index.sh  # Maintenance scripts
└── docs/spec/
    └── search-api.md          # This specification
```

## Performance Considerations

### Query Performance
- **Target**: < 200ms for 95% of queries
- **FTS5 Optimization**: Use contentless tables for better performance
- **Index Size**: Limit commit history indexing (1000 recent commits per repo)
- **Caching**: Implement Redis cache for frequent queries

### Index Maintenance
- **Batch Updates**: Process multiple commits in single transaction
- **Background Processing**: Use queue for index updates to avoid blocking pushes
- **Index Rebuilding**: Weekly full reindex during low-traffic periods

### Memory Usage
- **Index Size Estimation**: ~1KB per commit, 500 bytes per repo
- **For 1000 repos with 1000 commits each**: ~1GB index size
- **SQLite Performance**: Enable WAL mode for concurrent access

## Testing Strategy

### Unit Tests
1. **Search Query Processing**:
   ```typescript
   describe('Search Query Processing', () => {
     test('normalizes search terms correctly', () => {
       expect(processSearchQuery('Superfluid  Protocol')).toEqual({
         normalized: 'superfluid protocol',
         terms: ['superfluid', 'protocol'],
         ftsQuery: '"superfluid"* OR superfluid* AND "protocol"* OR protocol*'
       });
     });
   });
   ```

2. **Ranking Algorithm**:
   ```typescript
   describe('Search Ranking', () => {
     test('prioritizes repository name matches', () => {
       const results = rankSearchResults(mockSearchResults, 'superfluid');
       expect(results[0].repo?.name).toContain('superfluid');
     });
   });
   ```

### Integration Tests
1. **API Endpoints**:
   ```typescript
   describe('Search API', () => {
     test('returns paginated results', async () => {
       const response = await request(app)
         .get('/api/explorer/search?q=test&limit=5')
         .expect(200);
       
       expect(response.body.results).toHaveLength(5);
       expect(response.body.pagination.hasMore).toBe(true);
     });
   });
   ```

2. **Index Updates**:
   ```typescript
   describe('Search Indexing', () => {
     test('updates index on new commits', async () => {
       // Simulate push event
       await simulatePush('0x123...abc', 'test-repo', 'abc123', 'Add search feature');
       
       // Verify index was updated
       const results = await searchCommits('search feature');
       expect(results).toHaveLength(1);
       expect(results[0].commit?.hash).toBe('abc123');
     });
   });
   ```

### Performance Tests
1. **Load Testing**:
   ```bash
   # Test concurrent search requests
   ab -n 1000 -c 50 "http://localhost:3000/api/explorer/search?q=superfluid"
   ```

2. **Index Performance**:
   ```sql
   -- Verify FTS5 query performance
   .timer on
   SELECT * FROM search_index_fts WHERE search_index_fts MATCH 'superfluid*' LIMIT 20;
   ```

## Error Handling

### API Error Responses
```typescript
interface SearchError {
  error: string;
  code: 'INVALID_QUERY' | 'INDEX_UNAVAILABLE' | 'TIMEOUT' | 'INTERNAL_ERROR';
  message: string;
  details?: any;
}
```

### Common Error Scenarios
1. **Query too short** (< 2 characters):
   ```json
   {
     "error": "INVALID_QUERY",
     "code": "INVALID_QUERY",
     "message": "Search query must be at least 2 characters long"
   }
   ```

2. **Index unavailable**:
   ```json
   {
     "error": "INDEX_UNAVAILABLE", 
     "code": "INDEX_UNAVAILABLE",
     "message": "Search index is currently being rebuilt. Please try again in a few minutes."
   }
   ```

3. **Query timeout**:
   ```json
   {
     "error": "TIMEOUT",
     "code": "TIMEOUT", 
     "message": "Search query timed out. Please try a more specific search."
   }
   ```

## Security Considerations

### Input Validation
- Sanitize search queries to prevent FTS5 injection
- Limit query length to prevent DoS attacks
- Rate limiting: 100 requests per minute per IP

### Access Control
- Search results respect repository permissions
- No sensitive information in index (tokens, private keys)
- Audit logging for search analytics

### Data Privacy
- Hash IP addresses in analytics
- No storage of personal search history
- GDPR compliance for EU users

## Monitoring & Observability

### Metrics to Track
1. **Performance Metrics**:
   - Search response time (p50, p95, p99)
   - Index update latency
   - Query throughput

2. **Usage Metrics**:
   - Most popular search terms
   - Search result click-through rates
   - Zero-result queries

3. **System Health**:
   - Index size growth
   - Failed index updates
   - Database connection health

### Logging Strategy
```typescript
interface SearchLog {
  timestamp: string;
  query: string;
  resultCount: number;
  responseTime: number;
  userAgent?: string;
  ipHash?: string;
  errors?: string[];
}
```

## Migration & Rollback Plan

### Deployment Steps
1. Deploy database schema changes
2. Run initial index population (can take 10-30 minutes)
3. Deploy API endpoints with feature flag
4. Deploy frontend changes
5. Enable search feature for all users

### Rollback Strategy
- Feature flag to disable search API
- Fallback to client-side filtering
- Database migration rollback scripts
- Index cleanup procedures

## Future Enhancements

### Phase 2 Features
1. **Advanced Search**:
   - Search operators (`AND`, `OR`, `NOT`)
   - Field-specific search (`author:alice`, `repo:superfluid`)
   - Date range filtering

2. **Search Suggestions**:
   - Autocomplete based on popular queries
   - Typo correction using Levenshtein distance
   - Related search recommendations

3. **Search Analytics Dashboard**:
   - Popular search terms
   - Search performance metrics
   - Zero-result queries analysis

### Long-term Vision
1. **Semantic Search**: Vector embeddings for meaning-based search
2. **Code Search**: Search within file contents using AST parsing
3. **Fuzzy Matching**: Support for typos and approximate matches
4. **Saved Searches**: Allow users to save and subscribe to search queries

---

## Conclusion

This specification provides a comprehensive plan for implementing global search across repositories, addresses, and commit messages in repo.box. The phased approach ensures incremental delivery of value while maintaining system stability and performance.

The implementation leverages SQLite's FTS5 for full-text search capabilities, providing a performant and scalable solution that can handle the expected load without requiring additional infrastructure dependencies.

**Next Steps**: 
1. Review and approve this specification
2. Begin Phase 1 implementation
3. Set up monitoring and testing infrastructure
4. Plan user acceptance testing with early adopters