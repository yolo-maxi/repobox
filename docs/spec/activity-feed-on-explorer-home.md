# Activity Feed on Explorer Home

**Feature**: Wire up the existing push log to show actual recent activity on the explorer home page

**Status**: In Progress  
**Priority**: P1  
**Assignee**: PM Agent

## Problem Statement

The repo.box explorer home page currently shows "No recent activity" in the Recent Activity column, even though push events are being logged in the database. Users should see a live feed of recent commits and pushes across all repositories.

## Current Infrastructure Analysis

### ✅ What Already Exists

1. **Push Logging System**: 
   - `push_log` table with complete schema (`db.rs`)
   - `insert_push_log()` function called in all push routes (`routes.rs`)
   - Proper indexing for performance queries

2. **Frontend Components**:
   - Complete activity feed UI in `/explore` page (`page.tsx`)
   - Activity list rendering with proper formatting
   - Time formatting and address truncation utilities

3. **API Endpoint**:
   - `/api/explorer/activity` route implemented (`route.ts`)
   - Fetches from push_log table with limit parameter
   - Returns structured JSON with activity array

### ❌ Current Issues

1. **Database Connection**: Web frontend uses different database access pattern than Rust backend
2. **Database Location**: Frontend expects DB at `/var/lib/repobox/repos/repobox.db`, backend may use different path
3. **Table Synchronization**: Frontend creates `push_log` table via SQLite command, but may not match backend schema exactly

## Technical Specification

### 1. Database Path Consistency

**Problem**: Frontend and backend use different database locations

**Solution**: 
- Standardize database path via environment variable `REPOBOX_DATA_DIR`
- Default: `/var/lib/repobox/repos/repobox.db`
- Ensure both frontend and backend use the same path

```rust
// Backend (already exists in db.rs)
fn db_path() -> PathBuf {
    std::env::var("REPOBOX_DATA_DIR")
        .unwrap_or_else(|_| "/var/lib/repobox/repos".to_string())
        .into()
        .join("repobox.db")
}
```

```typescript
// Frontend (web/src/lib/database.ts)
const DATA_DIR = process.env.REPOBOX_DATA_DIR || '/var/lib/repobox/repos';
const DB_PATH = path.join(DATA_DIR, 'repobox.db');
```

### 2. Schema Synchronization

**Problem**: Frontend may create table with different schema than backend

**Solution**: Remove table creation from frontend, rely on backend initialization

```typescript
// Remove from web/src/lib/database.ts:
// CREATE TABLE IF NOT EXISTS push_log (...) 

// Backend db.rs already handles table creation properly
```

### 3. Enhanced Activity API

**Current API Response**:
```json
{
  "activity": [
    {
      "id": 1,
      "address": "0x123...",
      "name": "my-repo",
      "pusher_address": "0x123...", 
      "commit_hash": "abc123...",
      "commit_message": "Initial commit",
      "pushed_at": "1711234567"
    }
  ],
  "total": 1
}
```

**Enhancements Needed**:
1. Include repository metadata (owner info, description)
2. Better commit message truncation
3. More robust error handling

```typescript
// Enhanced query in route.ts
const pushLogs = await runQuery<EnhancedActivity>(`
  SELECT 
    p.id,
    p.address,
    p.name,
    p.pusher_address,
    p.commit_hash,
    p.commit_message,
    p.pushed_at,
    r.owner_address,
    r.created_at as repo_created_at
  FROM push_log p
  LEFT JOIN repos r ON (p.address = r.address AND p.name = r.name)
  ORDER BY p.pushed_at DESC 
  LIMIT ?
`, [limit]);
```

### 4. UI Component Improvements

**Current State**: UI components are well-implemented but can be enhanced

**Enhancements**:
1. **Commit Hash Links**: Link to specific commit pages
2. **Repository Icons**: Show repository type/language icons
3. **Activity Grouping**: Group multiple commits to same repo
4. **Real-time Updates**: Add polling or WebSocket updates

```tsx
// Enhanced activity item rendering
<div key={item.id} className="explore-activity-item">
  <div className="explore-activity-header">
    <Link href={`/explore/${item.address}/${item.name}`} 
          className="explore-activity-repo">
      {item.name}
    </Link>
    {item.commit_hash && (
      <Link href={`/explore/${item.address}/${item.name}/commit/${item.commit_hash}`}
            className="explore-activity-commit">
        {item.commit_hash.substring(0, 8)}
      </Link>
    )}
  </div>
  
  {item.commit_message && (
    <p className="explore-activity-message">
      {truncateMessage(item.commit_message, 120)}
    </p>
  )}
  
  <div className="explore-activity-meta">
    <span>{formatTimeAgo(item.pushed_at)}</span>
    {item.pusher_address && (
      <span>by <AddressLink address={item.pusher_address} /></span>
    )}
  </div>
</div>
```

## Implementation Phases

### Phase 1: Database Connection Fix (30 min)
1. **Verify Database Path**: Ensure backend and frontend use same database location
2. **Check Push Logging**: Verify `insert_push_log` calls are actually executing
3. **Test API Endpoint**: Manually test `/api/explorer/activity` endpoint
4. **Debug Database**: Check if push_log table has any data

**Acceptance Criteria**:
- `/api/explorer/activity` returns actual push log data
- Activity feed shows "No recent activity" only when no pushes exist
- Database path consistency between frontend and backend

### Phase 2: Enhanced Data & UI (45 min) 
1. **Enhanced Query**: Join with repos table for richer data
2. **Commit Links**: Add navigation to specific commits
3. **Better Formatting**: Improved commit message truncation
4. **Error Handling**: Graceful degradation on API failures

**Acceptance Criteria**:
- Activity items link to individual commits
- Repository owner information displayed
- Commit messages properly truncated with "..." 
- Activity feed shows loading states

### Phase 3: Real-time & Performance (30 min)
1. **Auto-refresh**: Poll activity endpoint every 30 seconds
2. **Pagination**: Support for "Load more" functionality  
3. **Performance**: Index optimization and query caching
4. **Visual Polish**: Loading states and animations

**Acceptance Criteria**:
- Activity feed updates without page refresh
- Supports > 100 activity items with pagination
- Smooth loading transitions
- < 200ms API response times

## Testing Strategy

### Unit Tests
```bash
# Test database operations
cargo test db::insert_push_log
cargo test db::push_log_indexes

# Test API endpoints  
npm test -- api/explorer/activity
```

### Integration Tests
```bash
# Test end-to-end push logging
./scripts/test-push-flow.sh
```

### Manual Testing
1. **Create Test Repository**:
   ```bash
   git clone <test-repo>
   git commit --allow-empty -S -m "Test commit for activity feed"
   git push
   ```

2. **Verify Activity Appears**: Check `/explore` page shows the commit

3. **Test Edge Cases**:
   - Empty repository (no commits)
   - Multiple rapid commits
   - Long commit messages
   - Special characters in commit messages

## Success Metrics

1. **Functional**: Activity feed shows actual push events instead of "No recent activity"
2. **Performance**: Activity API responds in < 200ms
3. **User Experience**: Users can click through from activity to commits/repositories
4. **Real-time**: Activity appears within 5 seconds of push completion

## Risk Assessment

**Low Risk**: 
- All infrastructure components already exist
- UI is already implemented and functional
- Database schema is established and indexed

**Medium Risk**:
- Database path/permission issues on production
- Performance with high activity volume  

**Mitigation**:
- Thorough testing of database connectivity
- Implement pagination and caching
- Monitor API performance metrics

## Rollback Plan

If issues arise:
1. **Immediate**: Return to "No recent activity" placeholder
2. **Database Issues**: Fallback to file-based activity logging
3. **API Failures**: Graceful degradation with cached data

---

## Appendix: Current Schema

```sql
-- push_log table (already implemented)
CREATE TABLE push_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL,
    name TEXT NOT NULL,
    pusher_address TEXT,
    commit_hash TEXT,
    commit_message TEXT,
    pushed_at TEXT NOT NULL
);

-- indexes (already implemented)
CREATE INDEX idx_push_log_timestamp ON push_log(pushed_at DESC);
CREATE INDEX idx_push_log_repo ON push_log(address, name);

-- repos table (for enhanced queries)
CREATE TABLE repos (
    address TEXT NOT NULL,
    name TEXT NOT NULL,
    owner_address TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY(address, name)
);
```

## Files Modified

1. `web/src/lib/database.ts` - Database path consistency
2. `web/src/app/api/explorer/activity/route.ts` - Enhanced queries  
3. `web/src/app/explore/page.tsx` - UI improvements (optional)
4. `repobox-server/src/db.rs` - Verify push logging (investigation)

## Dependencies

- SQLite3 (already installed)
- Next.js API routes (already implemented)
- Existing database schema (already established)

---

**Estimated Total Time**: 2 hours  
**Risk Level**: Low  
**Business Impact**: High (core feature for user engagement)