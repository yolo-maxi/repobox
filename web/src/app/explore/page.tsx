'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import EmptyState from '@/components/EmptyState';
import { EmptyRepository, NoSearchResults, QuietActivity } from '@/components/illustrations';
import { SiteNav } from '@/components/SiteNav';
import AddressDisplay from '@/components/AddressDisplay';
import EnsSubdomainModal from '@/components/EnsSubdomainModal';

function truncateMsg(msg: string, max = 60): string {
  if (msg.length <= max) return msg;
  return msg.substring(0, max).trim() + '…';
}

interface Stats { totalRepos: number; totalOwners: number; totalCommits: number }
interface Repo {
  address: string; name: string; owner_address: string; created_at: string;
  commit_count: number; contributor_count: number; last_commit_date: string | null;
  description: string | null;
}
interface Activity {
  id: number; address: string; name: string; pusher_address?: string;
  commit_hash?: string; commit_message?: string; pushed_at: string;
}

export default function ExplorePage() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [repos, setRepos] = useState<Repo[]>([]);
  const [activity, setActivity] = useState<Activity[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<'latest' | 'commits' | 'name'>('commits');
  const [showDemo, setShowDemo] = useState(true);
  const [shouldAutoOpenMint, setShouldAutoOpenMint] = useState(false);
  const [prefillMintName, setPrefillMintName] = useState('');

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, reposRes, activityRes] = await Promise.all([
          fetch('/api/explorer/stats'),
          fetch(`/api/explorer/repos?sort=${sortBy}&limit=50`),
          fetch('/api/explorer/activity?limit=15')
        ]);
        if (statsRes.ok) setStats(await statsRes.json());
        if (reposRes.ok) { const d = await reposRes.json(); setRepos(d.repos || []); }
        if (activityRes.ok) { const d = await activityRes.json(); setActivity(d.activity || []); }
      } catch (e) { console.error(e); }
      finally { setLoading(false); }
    };
    fetchData();
    const iv = setInterval(() => {
      fetch('/api/explorer/activity?limit=15')
        .then(r => r.ok ? r.json() : null)
        .then(d => { if (d?.activity) setActivity(d.activity); })
        .catch(() => {});
    }, 30000);
    return () => clearInterval(iv);
  }, [sortBy]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const params = new URLSearchParams(window.location.search);
    const mintParam = (params.get('mint') || '').toLowerCase();
    setShouldAutoOpenMint(['1', 'true', 'yes', 'open'].includes(mintParam));
    setPrefillMintName(params.get('name') || '');
  }, []);

  const realRepos = repos.filter(r => !r.name.startsWith('demo-hackathon-') && r.name !== 'private-test');
  const demoRepos = repos.filter(r => r.name.startsWith('demo-hackathon-') || r.name === 'private-test');
  const displayRepos = showDemo ? repos : realRepos;
  const filtered = displayRepos.filter(r =>
    r.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    r.address.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Deduplicate activity by repo name
  const deduped = activity.reduce((acc, item) => {
    if (!acc.find(a => a.name === item.name)) acc.push(item);
    return acc;
  }, [] as Activity[]);

  return (
    <div className="explore-root">
      <SiteNav />

      {/* Layout: sidebar left, content right */}
      <div className="explore-layout">
        {/* Sidebar */}
        <aside className="explore-sidebar">
          {/* Stats */}
          {stats && (
            <div className="explore-card">
              <div className="explore-card-label">Overview</div>
              <div className="explore-stat-row">
                <span>Repositories</span>
                <span className="explore-stat-val">{stats.totalRepos}</span>
              </div>
              <div className="explore-stat-row">
                <span>Developers</span>
                <span className="explore-stat-val">{stats.totalOwners}</span>
              </div>
              <div className="explore-stat-row">
                <span>Signed Commits</span>
                <span className="explore-stat-val">{stats.totalCommits}</span>
              </div>
            </div>
          )}

          {/* Sort */}
          <div className="explore-card">
            <div className="explore-card-label">Sort by</div>
            {(['latest', 'commits', 'name'] as const).map(opt => (
              <button
                key={opt}
                onClick={() => setSortBy(opt)}
                className={`explore-sort-btn ${sortBy === opt ? 'active' : ''}`}
              >
                {opt === 'latest' ? 'Recently updated' : opt === 'commits' ? 'Most commits' : 'Name'}
              </button>
            ))}
          </div>

          {/* Activity */}
          <div className="explore-card">
            <div className="explore-card-label">Recent Activity</div>
            {deduped.length === 0 ? (
              <p className="explore-empty-text">No recent activity</p>
            ) : (
              <div className="explore-activity-list">
                {deduped.slice(0, 8).map(item => (
                  <div key={item.id} className="explore-activity-item">
                    <div className="explore-activity-dot" />
                    <div>
                      <Link href={`/${item.address}/${item.name}`} className="explore-activity-name">
                        {item.name}
                      </Link>
                      {item.commit_message && (
                        <p className="explore-activity-msg">{truncateMsg(item.commit_message)}</p>
                      )}
                      <span className="explore-activity-time">{formatTimeAgo(item.pushed_at)}</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </aside>

        {/* Main content */}
        <main className="explore-main">
          {/* Header bar */}
          <div className="explore-main-header">
            <h1 className="explore-title">Repositories</h1>
            <div className="explore-main-header-actions">
              <EnsSubdomainModal
                triggerLabel="Mint ENS name"
                initialOpen={shouldAutoOpenMint}
                prefillName={prefillMintName}
              />
              <div className="explore-search-wrap">
                <svg className="explore-search-icon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M10.68 11.74a6 6 0 0 1-7.92-8.98 6 6 0 0 1 8.98 7.92l3.81 3.81a.75.75 0 0 1-1.06 1.06l-3.81-3.81zM6.5 11a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9z"/>
                </svg>
                <input
                  type="text"
                  placeholder="Search…"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="explore-search"
                />
              </div>
            </div>
          </div>

          {/* Repo list */}
          <div className="explore-repo-list">
            {loading ? (
              <>
                {[1,2,3,4,5].map(i => (
                  <div key={i} className="explore-repo-skeleton" />
                ))}
              </>
            ) : filtered.length === 0 ? (
              <div className="explore-repo-empty">
                <EmptyState
                  illustration={searchTerm ? NoSearchResults : EmptyRepository}
                  title={searchTerm ? 'No matching repositories' : 'No repositories yet'}
                  description={searchTerm ? 'Try a different search term' : 'Push your first signed commit to get started'}
                  size="lg"
                />
              </div>
            ) : (
              <>
                {filtered.map((repo) => (
                  <Link
                    key={`${repo.address}/${repo.name}`}
                    href={`/${repo.address}/${repo.name}`}
                    className="explore-repo-row"
                  >
                    <div className="explore-repo-info">
                      <div className="explore-repo-name-row">
                        <span className="explore-repo-name">{repo.name}</span>
                        <AddressDisplay address={repo.owner_address} size="sm" showCopy={false} linkable={false} />
                      </div>
                      {repo.description && (
                        <p className="explore-repo-desc">{repo.description.replace(/\n/g, ' ').trim()}</p>
                      )}
                    </div>
                    <div className="explore-repo-stats">
                      {repo.contributor_count > 0 && (
                        <span className="explore-repo-pill">
                          <span style={{ fontSize: 10 }}>👤</span> {repo.contributor_count}
                        </span>
                      )}
                      <span className={`explore-repo-pill ${repo.commit_count > 10 ? 'highlight' : ''}`}>
                        ⬡ {repo.commit_count}
                      </span>
                      {repo.last_commit_date && (
                        <span className="explore-repo-time">{formatTimeAgo(repo.last_commit_date)}</span>
                      )}
                    </div>
                  </Link>
                ))}
              </>
            )}
          </div>

          {/* Demo toggle */}
          {demoRepos.length > 0 && !searchTerm && (
            <button onClick={() => setShowDemo(!showDemo)} className="explore-demo-toggle">
              {showDemo ? 'Hide demo repos' : `Show all (+${demoRepos.length} demo)`}
            </button>
          )}
        </main>
      </div>

      <style>{`
        .explore-root {
          min-height: 100vh;
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono, 'JetBrains Mono', 'Fira Code', monospace);
          font-size: 13px;
          line-height: 20px;
        }

        /* Layout */
        .explore-layout {
          display: grid;
          grid-template-columns: 280px 1fr;
          gap: 0;
          max-width: 1400px;
          margin: 0 auto;
          min-height: calc(100vh - 53px);
        }

        /* Sidebar */
        .explore-sidebar {
          padding: 24px 20px;
          border-right: 1px solid var(--bp-border);
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .explore-card {
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 8px;
          padding: 16px;
        }
        .explore-card-label {
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          color: var(--bp-dim);
          margin-bottom: 12px;
        }

        .explore-stat-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 4px 0;
          font-size: 12px;
          color: var(--bp-dim);
        }
        .explore-stat-val {
          font-weight: 700;
          color: var(--bp-gold);
          font-variant-numeric: tabular-nums;
        }

        .explore-sort-btn {
          display: block;
          width: 100%;
          text-align: left;
          padding: 6px 8px;
          margin-bottom: 2px;
          border-radius: 4px;
          border: none;
          cursor: pointer;
          font-size: 12px;
          font-family: inherit;
          background: transparent;
          color: var(--bp-dim);
          transition: all 0.12s;
        }
        .explore-sort-btn:hover {
          color: var(--bp-text);
          background: rgba(79, 195, 247, 0.05);
        }
        .explore-sort-btn.active {
          color: var(--bp-accent);
          background: rgba(79, 195, 247, 0.1);
          font-weight: 500;
        }

        .explore-activity-list {
          display: flex;
          flex-direction: column;
          gap: 10px;
        }
        .explore-activity-item {
          display: flex;
          gap: 8px;
          align-items: flex-start;
        }
        .explore-activity-dot {
          width: 6px;
          height: 6px;
          border-radius: 50%;
          background: var(--bp-accent);
          flex-shrink: 0;
          margin-top: 7px;
          opacity: 0.7;
        }
        .explore-activity-name {
          color: var(--bp-accent);
          text-decoration: none;
          font-weight: 500;
          font-size: 12px;
        }
        .explore-activity-name:hover { opacity: 0.8; }
        .explore-activity-msg {
          color: var(--bp-dim);
          margin: 2px 0 0;
          font-size: 11px;
          line-height: 1.4;
        }
        .explore-activity-time {
          color: rgba(122, 154, 180, 0.5);
          font-size: 10px;
        }
        .explore-empty-text {
          color: var(--bp-dim);
          font-size: 12px;
          opacity: 0.6;
        }

        /* Main content */
        .explore-main {
          padding: 28px 40px;
          display: flex;
          flex-direction: column;
        }

        .explore-main-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 0;
          padding: 16px 20px;
          background: rgba(20, 40, 65, 0.6);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-radius: 8px 8px 0 0;
          border-bottom: 1px solid var(--bp-border);
        }
        .explore-title {
          font-size: 16px;
          font-weight: 600;
          color: var(--bp-heading);
          letter-spacing: -0.3px;
        }
        .explore-main-header-actions {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        .explore-search-wrap {
          position: relative;
          width: 220px;
        }
        .explore-search-icon {
          position: absolute;
          left: 10px;
          top: 50%;
          transform: translateY(-50%);
          color: var(--bp-dim);
          opacity: 0.5;
        }
        .explore-search {
          width: 100%;
          padding: 7px 12px 7px 30px;
          background: var(--bp-bg);
          border: 1px solid var(--bp-border);
          border-radius: 6px;
          color: var(--bp-text);
          font-size: 12px;
          font-family: inherit;
          outline: none;
          transition: border-color 0.15s;
        }
        .explore-search:focus {
          border-color: rgba(79, 195, 247, 0.4);
        }
        .explore-search::placeholder {
          color: var(--bp-dim);
          opacity: 0.5;
        }

        /* Repo list */
        .explore-repo-list {
          background: var(--bp-surface);
          border: 1px solid rgba(60, 120, 180, 0.2);
          border-top: none;
          border-radius: 0 0 8px 8px;
          overflow: hidden;
          flex: 1;
        }
        .explore-repo-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 14px 20px;
          text-decoration: none;
          color: inherit;
          border-bottom: 1px solid var(--bp-border);
          transition: background 0.12s;
          gap: 16px;
        }
        .explore-repo-row:last-child {
          border-bottom: none;
        }
        .explore-repo-row:hover {
          background: rgba(79, 195, 247, 0.03);
        }

        .explore-repo-info {
          flex: 1;
          min-width: 0;
        }
        .explore-repo-name-row {
          display: flex;
          align-items: baseline;
          gap: 8px;
          margin-bottom: 2px;
        }
        .explore-repo-name {
          color: var(--bp-accent);
          font-size: 14px;
          font-weight: 600;
        }
        .explore-repo-addr {
          color: var(--bp-dim);
          font-size: 11px;
          opacity: 0.6;
        }
        .explore-repo-desc {
          color: var(--bp-dim);
          font-size: 12px;
          margin: 0;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
          opacity: 0.8;
        }

        .explore-repo-stats {
          display: flex;
          align-items: center;
          gap: 10px;
          flex-shrink: 0;
          font-size: 11px;
          color: var(--bp-dim);
        }
        .explore-repo-pill {
          display: flex;
          align-items: center;
          gap: 4px;
          padding: 2px 8px;
          background: rgba(50, 100, 160, 0.15);
          border-radius: 10px;
          font-weight: 500;
        }
        .explore-repo-pill.highlight {
          color: var(--bp-accent);
          background: rgba(79, 195, 247, 0.12);
        }
        .explore-repo-time {
          color: var(--bp-dim);
          opacity: 0.5;
          white-space: nowrap;
        }

        .explore-repo-skeleton {
          height: 52px;
          border-bottom: 1px solid var(--bp-border);
          animation: pulse 2s ease-in-out infinite;
        }
        .explore-repo-empty {
          padding: 48px 20px;
          text-align: center;
        }

        .explore-demo-toggle {
          display: block;
          margin: 16px auto 0;
          padding: 6px 20px;
          background: transparent;
          border: 1px solid var(--bp-border);
          border-radius: 20px;
          color: var(--bp-dim);
          font-size: 11px;
          font-family: inherit;
          cursor: pointer;
          transition: all 0.15s;
        }
        .explore-demo-toggle:hover {
          border-color: rgba(79, 195, 247, 0.3);
          color: var(--bp-accent);
        }

        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.3; }
        }

        @media (max-width: 900px) {
          .explore-layout {
            grid-template-columns: 1fr;
          }
          .explore-sidebar {
            order: 2;
            border-right: none;
            padding: 16px;
            gap: 12px;
          }
          .explore-main {
            order: 1;
            padding: 16px 16px;
          }
          .explore-main-header {
            flex-direction: column;
            gap: 10px;
            align-items: flex-start;
          }
          .explore-main-header-actions {
            width: 100%;
            flex-direction: column;
            align-items: stretch;
          }
          .explore-search-wrap {
            width: 100%;
          }
          /* Compact stats on mobile — inline */
          .explore-sidebar .explore-card:first-child {
            display: flex;
            flex-wrap: wrap;
            gap: 0;
            padding: 12px 16px;
          }
          .explore-sidebar .explore-card:first-child .explore-card-label {
            width: 100%;
            margin-bottom: 8px;
          }
          .explore-sidebar .explore-card:first-child .explore-stat-row {
            flex: 1;
            min-width: 100px;
            flex-direction: column;
            text-align: center;
            gap: 2px;
            padding: 4px 8px;
          }
          .explore-sidebar .explore-card:first-child .explore-stat-val {
            font-size: 18px;
          }
          /* Hide sort + activity on mobile */
          .explore-sidebar .explore-card:nth-child(2),
          .explore-sidebar .explore-card:nth-child(3) {
            display: none;
          }
          .explore-repo-stats {
            flex-wrap: wrap;
            gap: 6px;
          }
        }
      `}</style>
    </div>
  );
}
