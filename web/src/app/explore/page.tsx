'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import EmptyState from '@/components/EmptyState';
import { EmptyRepository, NoSearchResults, QuietActivity } from '@/components/illustrations';

function truncateMessage(message: string, maxLength: number = 80): string {
  if (message.length <= maxLength) return message;
  return message.substring(0, maxLength).trim() + '…';
}

function pluralize(count: number, singular: string, plural?: string): string {
  return count === 1 ? `${count} ${singular}` : `${count} ${plural || singular + 's'}`;
}

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

  const filteredRepos = repos.filter(repo =>
    repo.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    repo.address.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div style={{
      minHeight: '100vh',
      backgroundColor: '#0d1117',
      color: '#e6edf3',
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif',
    }}>
      {/* Header */}
      <header style={{
        borderBottom: '1px solid #21262d',
        backgroundColor: '#010409',
        padding: '12px 24px',
        position: 'sticky',
        top: 0,
        zIndex: 100,
      }}>
        <div style={{
          maxWidth: 1200,
          margin: '0 auto',
          display: 'flex',
          alignItems: 'center',
          gap: 24,
        }}>
          <Link href="/" style={{
            fontSize: 20,
            fontWeight: 600,
            color: '#e6edf3',
            textDecoration: 'none',
            fontFamily: '"JetBrains Mono", "Fira Code", monospace',
            letterSpacing: '-0.5px',
          }}>
            repo<span style={{ color: '#58a6ff' }}>.</span>box
          </Link>

          <nav style={{ display: 'flex', gap: 16, fontSize: 14 }}>
            <Link href="/" style={{ color: '#8b949e', textDecoration: 'none' }}>Home</Link>
            <Link href="/explore" style={{ color: '#e6edf3', textDecoration: 'none', fontWeight: 500 }}>Explore</Link>
            <Link href="/docs" style={{ color: '#8b949e', textDecoration: 'none' }}>Docs</Link>
            <Link href="/playground" style={{ color: '#8b949e', textDecoration: 'none' }}>Playground</Link>
          </nav>

          <div style={{ marginLeft: 'auto', position: 'relative', width: 280 }}>
            <svg style={{ position: 'absolute', left: 10, top: '50%', transform: 'translateY(-50%)', opacity: 0.4 }} width="16" height="16" viewBox="0 0 16 16" fill="#8b949e">
              <path d="M10.68 11.74a6 6 0 0 1-7.92-8.98 6 6 0 0 1 8.98 7.92l3.81 3.81a.75.75 0 0 1-1.06 1.06l-3.81-3.81zM6.5 11a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9z"/>
            </svg>
            <input
              type="text"
              placeholder="Search repositories…"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              style={{
                width: '100%',
                padding: '6px 12px 6px 32px',
                backgroundColor: '#0d1117',
                border: '1px solid #30363d',
                borderRadius: 6,
                color: '#e6edf3',
                fontSize: 14,
                outline: 'none',
              }}
            />
          </div>
        </div>
      </header>

      {/* Main layout */}
      <div style={{
        maxWidth: 1200,
        margin: '0 auto',
        padding: '24px 24px',
        display: 'grid',
        gridTemplateColumns: '260px 1fr',
        gap: 24,
      }}>
        {/* Sidebar */}
        <aside style={{ fontSize: 14 }}>
          {/* Stats */}
          {stats && (
            <div style={{
              padding: '16px',
              backgroundColor: '#161b22',
              border: '1px solid #21262d',
              borderRadius: 6,
              marginBottom: 16,
            }}>
              <h3 style={{ fontSize: 12, fontWeight: 600, color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: 12 }}>
                Overview
              </h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span style={{ color: '#8b949e' }}>Repositories</span>
                  <span style={{ fontWeight: 600 }}>{stats.totalRepos}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span style={{ color: '#8b949e' }}>Owners</span>
                  <span style={{ fontWeight: 600 }}>{stats.totalOwners}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span style={{ color: '#8b949e' }}>Commits</span>
                  <span style={{ fontWeight: 600 }}>{stats.totalCommits}</span>
                </div>
              </div>
            </div>
          )}

          {/* Sort */}
          <div style={{
            padding: '16px',
            backgroundColor: '#161b22',
            border: '1px solid #21262d',
            borderRadius: 6,
            marginBottom: 16,
          }}>
            <h3 style={{ fontSize: 12, fontWeight: 600, color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: 12 }}>
              Sort by
            </h3>
            {(['latest', 'commits', 'name'] as const).map(opt => (
              <button
                key={opt}
                onClick={() => setSortBy(opt)}
                style={{
                  display: 'block',
                  width: '100%',
                  textAlign: 'left',
                  padding: '6px 8px',
                  marginBottom: 2,
                  borderRadius: 4,
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: 14,
                  backgroundColor: sortBy === opt ? '#1f6feb22' : 'transparent',
                  color: sortBy === opt ? '#58a6ff' : '#e6edf3',
                  fontWeight: sortBy === opt ? 500 : 400,
                }}
              >
                {opt === 'latest' ? '🕐 Recently updated' : opt === 'commits' ? '📊 Most commits' : '🔤 Name'}
              </button>
            ))}
          </div>

          {/* Recent Activity - sidebar version */}
          <div style={{
            padding: '16px',
            backgroundColor: '#161b22',
            border: '1px solid #21262d',
            borderRadius: 6,
          }}>
            <h3 style={{ fontSize: 12, fontWeight: 600, color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: 12 }}>
              Recent Activity
            </h3>
            {activity.length === 0 ? (
              <EmptyState
                illustration={QuietActivity}
                title="No recent activity"
                size="sm"
              />
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                {activity.slice(0, 8).map(item => (
                  <div key={item.id} style={{ fontSize: 13 }}>
                    <Link
                      href={`/explore/${item.address}/${item.name}`}
                      style={{ color: '#58a6ff', textDecoration: 'none', fontWeight: 500 }}
                    >
                      {item.name}
                    </Link>
                    {item.commit_message && (
                      <p style={{ color: '#8b949e', margin: '2px 0 0', fontSize: 12, lineHeight: 1.4 }}>
                        {truncateMessage(item.commit_message, 60)}
                      </p>
                    )}
                    <span style={{ color: '#484f58', fontSize: 11 }}>
                      {formatTimeAgo(item.pushed_at)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </aside>

        {/* Main content */}
        <main>
          {loading ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {[1,2,3,4,5].map(i => (
                <div key={i} style={{
                  height: 72,
                  backgroundColor: '#161b22',
                  border: '1px solid #21262d',
                  borderRadius: 6,
                  animation: 'pulse 2s ease-in-out infinite',
                }} />
              ))}
            </div>
          ) : (
            <>
              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                marginBottom: 16,
                paddingBottom: 12,
                borderBottom: '1px solid #21262d',
              }}>
                <h2 style={{ fontSize: 16, fontWeight: 600 }}>
                  {searchTerm ? `Results for "${searchTerm}"` : 'Repositories'}
                </h2>
                <span style={{ fontSize: 13, color: '#8b949e' }}>
                  {filteredRepos.length} {filteredRepos.length === 1 ? 'repository' : 'repositories'}
                </span>
              </div>

              {filteredRepos.length === 0 ? (
                <EmptyState
                  illustration={searchTerm ? NoSearchResults : EmptyRepository}
                  title={searchTerm ? 'No matching repositories' : 'No repositories yet'}
                  description={searchTerm 
                    ? 'Try a different search term or browse all repositories' 
                    : 'Push your first signed commit to get started'}
                  size="lg"
                />
              ) : (
                <div style={{ display: 'flex', flexDirection: 'column' }}>
                  {filteredRepos.map((repo, idx) => (
                    <Link
                      key={`${repo.address}/${repo.name}`}
                      href={`/explore/${repo.address}/${repo.name}`}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '14px 16px',
                        borderBottom: idx < filteredRepos.length - 1 ? '1px solid #21262d' : 'none',
                        textDecoration: 'none',
                        color: 'inherit',
                        transition: 'background-color 0.15s',
                        gap: 16,
                      }}
                      onMouseEnter={e => (e.currentTarget.style.backgroundColor = '#161b22')}
                      onMouseLeave={e => (e.currentTarget.style.backgroundColor = 'transparent')}
                    >
                      {/* Left: name + description */}
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginBottom: 2 }}>
                          <span style={{ color: '#58a6ff', fontSize: 15, fontWeight: 600 }}>
                            {repo.name}
                          </span>
                          <AddressDisplay 
                            address={repo.owner_address}
                            size="sm"
                            showCopy={false}
                            linkable={true}
                            className="explore-repo-owner"
                          />
                        </div>
                        {repo.description && (
                          <p style={{ color: '#8b949e', fontSize: 13, margin: 0, lineHeight: 1.3, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                            {repo.description.replace(/\n/g, ' ').trim()}
                          </p>
                        )}
                      </div>

                      {/* Right: stats */}
                      <div style={{ display: 'flex', alignItems: 'center', gap: 20, flexShrink: 0, fontSize: 12, color: '#8b949e' }}>
                        {repo.contributor_count > 0 && (
                          <div style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: 4,
                            padding: '2px 8px',
                            backgroundColor: '#21262d',
                            borderRadius: 10,
                            fontSize: 12,
                            color: '#8b949e',
                            fontWeight: 500,
                          }}>
                            <span style={{ fontSize: 10 }}>👤</span> {repo.contributor_count}
                          </div>
                        )}
                        <div style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: 4,
                          padding: '2px 8px',
                          backgroundColor: repo.commit_count > 10 ? '#1f6feb22' : '#21262d',
                          borderRadius: 10,
                          fontSize: 12,
                          color: repo.commit_count > 10 ? '#58a6ff' : '#8b949e',
                          fontWeight: 500,
                        }}>
                          <span>⬡</span> {repo.commit_count}
                        </div>
                        {repo.last_commit_date && (
                          <span style={{ color: '#484f58', whiteSpace: 'nowrap', minWidth: 100, textAlign: 'right' }}>
                            {formatTimeAgo(repo.last_commit_date)}
                          </span>
                        )}
                      </div>
                    </Link>
                  ))}
                </div>
              )}
            </>
          )}
        </main>
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
        @media (max-width: 768px) {
          /* Stack sidebar below on mobile */
        }
      `}</style>
    </div>
  );
}
