'use client';

import { useState, useEffect, useMemo } from 'react';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import EmptyState from '@/components/EmptyState';
import { EmptyRepository, NoSearchResults, QuietActivity } from '@/components/illustrations';

function truncateMessage(message: string, maxLength: number = 80): string {
  if (message.length <= maxLength) return message;
  return message.substring(0, maxLength).trim() + '...';
}

const LANGUAGE_COLORS: Record<string, string> = {
  TypeScript: '#3178c6', JavaScript: '#f1e05a', Python: '#3572a5',
  Go: '#00add8', Rust: '#dea584', Java: '#b07219', Solidity: '#aa6746',
  Markdown: '#083fa1', JSON: '#292929', YAML: '#cb171e', Shell: '#89e051',
  CSS: '#563d7c', HTML: '#e34c26', TOML: '#9c4221', Other: '#8b949e',
};

interface Stats {
  totalRepos: number;
  totalOwners: number;
  totalCommits: number;
}

interface Repo {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
  commit_count: number;
  contributor_count: number;
  last_commit_date: string | null;
  description: string | null;
}

interface Activity {
  id: number;
  address: string;
  name: string;
  pusher_address?: string;
  commit_hash?: string;
  commit_message?: string;
  pushed_at: string;
}

export default function ExplorePage() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [repos, setRepos] = useState<Repo[]>([]);
  const [activity, setActivity] = useState<Activity[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<'latest' | 'commits' | 'name'>('latest');
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, reposRes, activityRes] = await Promise.all([
          fetch('/api/explorer/stats'),
          fetch(`/api/explorer/repos?sort=${sortBy}&limit=50`),
          fetch('/api/explorer/activity?limit=15')
        ]);
        if (statsRes.ok) setStats(await statsRes.json());
        if (reposRes.ok) {
          const data = await reposRes.json();
          setRepos(data.repos || []);
        }
        if (activityRes.ok) {
          const data = await activityRes.json();
          setActivity(data.activity || []);
        }
      } catch (error) {
        console.error('Error fetching data:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchData();

    const interval = setInterval(() => {
      fetch('/api/explorer/activity?limit=15')
        .then(res => res.ok ? res.json() : null)
        .then(data => { if (data?.activity) setActivity(data.activity); })
        .catch(() => {});
    }, 30000);
    return () => clearInterval(interval);
  }, [sortBy]);

  const filteredRepos = useMemo(() => {
    let result = repos.filter(repo =>
      repo.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      repo.address.toLowerCase().includes(searchTerm.toLowerCase())
    );
    if (!showAll) {
      result = result.filter(r => !r.name.startsWith('demo-hackathon-') && r.name !== 'private-test');
    }
    return result;
  }, [repos, searchTerm, showAll]);

  const hiddenCount = repos.length - repos.filter(r => !r.name.startsWith('demo-hackathon-') && r.name !== 'private-test').length;

  // Deduplicate activity: one entry per repo, most recent
  const deduplicatedActivity = useMemo(() => {
    const seen = new Map<string, Activity>();
    for (const item of activity) {
      const key = `${item.address}/${item.name}`;
      if (!seen.has(key)) {
        seen.set(key, item);
      }
    }
    return Array.from(seen.values()).slice(0, 6);
  }, [activity]);

  return (
    <div className="rd-explore-page">
      {/* Header */}
      <header className="rd-header">
        <div className="rd-header-inner">
          <Link href="/" className="rd-logo">
            repo<span className="rd-logo-dot">.</span>box
          </Link>
          <nav className="rd-nav">
            <Link href="/" className="rd-nav-link">Home</Link>
            <Link href="/explore" className="rd-nav-link rd-nav-link--active">Explore</Link>
            <Link href="/docs" className="rd-nav-link">Docs</Link>
            <Link href="/playground" className="rd-nav-link">Playground</Link>
          </nav>
        </div>
      </header>

      {/* Hero */}
      <section className="rd-hero">
        <div className="rd-hero-inner">
          <h1 className="rd-hero-title">
            Explore <span className="rd-hero-accent">on-chain</span> repositories
          </h1>
          <p className="rd-hero-subtitle">
            Git hosting with EVM-signed commits. Every push is cryptographically verified.
          </p>
          {stats && (
            <div className="rd-hero-stats">
              <div className="rd-hero-stat">
                <span className="rd-hero-stat-value">{stats.totalRepos}</span>
                <span className="rd-hero-stat-label">Repositories</span>
              </div>
              <div className="rd-hero-stat-divider" />
              <div className="rd-hero-stat">
                <span className="rd-hero-stat-value">{stats.totalOwners}</span>
                <span className="rd-hero-stat-label">Developers</span>
              </div>
              <div className="rd-hero-stat-divider" />
              <div className="rd-hero-stat">
                <span className="rd-hero-stat-value">{stats.totalCommits.toLocaleString()}</span>
                <span className="rd-hero-stat-label">Signed Commits</span>
              </div>
            </div>
          )}
        </div>
      </section>

      {/* Main layout */}
      <div className="rd-main">
        {/* Repo list */}
        <div className="rd-content">
          {/* Search + filter bar */}
          <div className="rd-toolbar">
            <div className="rd-search-wrap">
              <svg className="rd-search-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M10.68 11.74a6 6 0 0 1-7.92-8.98 6 6 0 0 1 8.98 7.92l3.81 3.81a.75.75 0 0 1-1.06 1.06l-3.81-3.81zM6.5 11a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9z"/>
              </svg>
              <input
                type="text"
                placeholder="Search repositories..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="rd-search-input"
              />
            </div>
            <div className="rd-sort-btns">
              {([['latest', 'Recent'], ['commits', 'Most commits'], ['name', 'Name']] as const).map(([key, label]) => (
                <button
                  key={key}
                  onClick={() => setSortBy(key)}
                  className={`rd-sort-btn ${sortBy === key ? 'rd-sort-btn--active' : ''}`}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>

          {/* Repo count + show all toggle */}
          <div className="rd-list-header">
            <span className="rd-list-count">
              {searchTerm ? `Results for "${searchTerm}"` : 'Repositories'}
              <span className="rd-list-count-num">{filteredRepos.length}</span>
            </span>
            {hiddenCount > 0 && (
              <button
                onClick={() => setShowAll(!showAll)}
                className="rd-show-all-btn"
              >
                {showAll ? 'Hide demo repos' : `Show all (+${hiddenCount} demo)`}
              </button>
            )}
          </div>

          {/* Repo list */}
          {loading ? (
            <div className="rd-skeleton-list">
              {[1,2,3,4,5].map(i => (
                <div key={i} className="rd-skeleton-card" />
              ))}
            </div>
          ) : filteredRepos.length === 0 ? (
            <EmptyState
              illustration={searchTerm ? NoSearchResults : EmptyRepository}
              title={searchTerm ? 'No matching repositories' : 'No repositories yet'}
              description={searchTerm
                ? 'Try a different search term or browse all repositories'
                : 'Push your first signed commit to get started'}
              size="lg"
            />
          ) : (
            <div className="rd-repo-list">
              {filteredRepos.map((repo) => (
                <Link
                  key={`${repo.address}/${repo.name}`}
                  href={`/explore/${repo.address}/${repo.name}`}
                  className="rd-repo-card"
                >
                  <div className="rd-repo-card-main">
                    <div className="rd-repo-card-title-row">
                      <svg className="rd-repo-card-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.249.249 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z"/>
                      </svg>
                      <span className="rd-repo-card-name">{repo.name}</span>
                      <AddressDisplay
                        address={repo.owner_address}
                        size="sm"
                        showCopy={false}
                        linkable={true}
                        className="rd-repo-card-owner"
                      />
                    </div>
                    {repo.description && (
                      <p className="rd-repo-card-desc">
                        {repo.description.replace(/\n/g, ' ').trim()}
                      </p>
                    )}
                  </div>
                  <div className="rd-repo-card-meta">
                    {repo.contributor_count > 0 && (
                      <span className="rd-repo-card-badge">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                          <path d="M10.561 8.073a6.005 6.005 0 0 1 3.432 5.142.75.75 0 1 1-1.498.07 4.5 4.5 0 0 0-8.99 0 .75.75 0 0 1-1.498-.07 6.004 6.004 0 0 1 3.431-5.142 3.999 3.999 0 1 1 5.123 0ZM10.5 5a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z"/>
                        </svg>
                        {repo.contributor_count}
                      </span>
                    )}
                    <span className={`rd-repo-card-badge ${repo.commit_count > 10 ? 'rd-repo-card-badge--highlight' : ''}`}>
                      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M11.93 8.5a4.002 4.002 0 0 1-7.86 0H.75a.75.75 0 0 1 0-1.5h3.32a4.002 4.002 0 0 1 7.86 0h3.32a.75.75 0 0 1 0 1.5Zm-1.43-.25a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z"/>
                      </svg>
                      {repo.commit_count}
                    </span>
                    {repo.last_commit_date && (
                      <span className="rd-repo-card-time">
                        {formatTimeAgo(repo.last_commit_date)}
                      </span>
                    )}
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>

        {/* Sidebar */}
        <aside className="rd-sidebar">
          {/* Recent Activity */}
          <div className="rd-sidebar-section">
            <h3 className="rd-sidebar-title">Recent Activity</h3>
            {deduplicatedActivity.length === 0 ? (
              <EmptyState
                illustration={QuietActivity}
                title="No recent activity"
                size="sm"
              />
            ) : (
              <div className="rd-activity-list">
                {deduplicatedActivity.map(item => (
                  <div key={item.id} className="rd-activity-item">
                    <div className="rd-activity-dot" />
                    <div className="rd-activity-content">
                      <Link
                        href={`/explore/${item.address}/${item.name}`}
                        className="rd-activity-repo"
                      >
                        {item.name}
                      </Link>
                      {item.commit_message && (
                        <p className="rd-activity-msg">
                          {truncateMessage(item.commit_message, 55)}
                        </p>
                      )}
                      <span className="rd-activity-time">
                        {formatTimeAgo(item.pushed_at)}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </aside>
      </div>
    </div>
  );
}
