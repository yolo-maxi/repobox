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
  const [showDemo, setShowDemo] = useState(false);

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

  const realRepos = repos.filter(r => !r.name.startsWith('demo-hackathon-') && r.name !== 'private-test');
  const demoRepos = repos.filter(r => r.name.startsWith('demo-hackathon-') || r.name === 'private-test');
  const displayRepos = showDemo ? repos : realRepos;
  const filteredRepos = displayRepos.filter(repo =>
    repo.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    repo.address.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Deduplicate activity by repo name (show latest per repo)
  const deduped = activity.reduce((acc, item) => {
    if (!acc.find(a => a.name === item.name)) acc.push(item);
    return acc;
  }, [] as Activity[]);

  return (
    <div style={{
      minHeight: '100vh',
      backgroundColor: '#0d1117',
      color: '#e6edf3',
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif',
    }}>
      {/* Nav */}
      <header style={{
        borderBottom: '1px solid #21262d',
        backgroundColor: '#010409',
        padding: '12px 32px',
        position: 'sticky',
        top: 0,
        zIndex: 100,
      }}>
        <div style={{
          maxWidth: 1280,
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
        </div>
      </header>

      {/* Hero — full width */}
      <div style={{
        borderBottom: '1px solid #21262d',
        padding: '48px 32px 40px',
        background: 'linear-gradient(180deg, #0d1117 0%, #161b22 100%)',
      }}>
        <div style={{ maxWidth: 1280, margin: '0 auto' }}>
          <h1 style={{
            fontSize: 32,
            fontWeight: 700,
            letterSpacing: '-0.5px',
            marginBottom: 8,
            lineHeight: 1.2,
          }}>
            Explore repositories
          </h1>
          <p style={{ fontSize: 16, color: '#8b949e', marginBottom: 28, maxWidth: 540 }}>
            Every commit EVM-signed. Every push permission-checked on-chain.
          </p>

          {/* Stats row */}
          {stats && (
            <div style={{ display: 'flex', gap: 32 }}>
              {[
                { label: 'Repositories', value: stats.totalRepos, icon: '📦' },
                { label: 'Developers', value: stats.totalOwners, icon: '👤' },
                { label: 'Signed Commits', value: stats.totalCommits, icon: '⬡' },
              ].map(s => (
                <div key={s.label} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontSize: 14 }}>{s.icon}</span>
                  <span style={{ fontSize: 24, fontWeight: 700, fontVariantNumeric: 'tabular-nums' }}>{s.value}</span>
                  <span style={{ fontSize: 13, color: '#8b949e' }}>{s.label}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Content grid */}
      <div style={{
        maxWidth: 1280,
        margin: '0 auto',
        padding: '24px 32px 64px',
        display: 'grid',
        gridTemplateColumns: '1fr 320px',
        gap: 32,
        alignItems: 'start',
      }}>
        {/* Main — repo list */}
        <main>
          {/* Toolbar */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            marginBottom: 16,
            paddingBottom: 16,
            borderBottom: '1px solid #21262d',
          }}>
            {/* Search */}
            <div style={{ position: 'relative', flex: 1 }}>
              <svg style={{ position: 'absolute', left: 10, top: '50%', transform: 'translateY(-50%)', opacity: 0.4 }} width="16" height="16" viewBox="0 0 16 16" fill="#8b949e">
                <path d="M10.68 11.74a6 6 0 0 1-7.92-8.98 6 6 0 0 1 8.98 7.92l3.81 3.81a.75.75 0 0 1-1.06 1.06l-3.81-3.81zM6.5 11a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9z"/>
              </svg>
              <input
                type="text"
                placeholder="Find a repository…"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                style={{
                  width: '100%',
                  padding: '8px 12px 8px 34px',
                  backgroundColor: '#010409',
                  border: '1px solid #30363d',
                  borderRadius: 6,
                  color: '#e6edf3',
                  fontSize: 14,
                  outline: 'none',
                }}
              />
            </div>
            {/* Sort tabs */}
            <div style={{ display: 'flex', gap: 2, backgroundColor: '#21262d', borderRadius: 6, padding: 2 }}>
              {(['latest', 'commits', 'name'] as const).map(opt => (
                <button
                  key={opt}
                  onClick={() => setSortBy(opt)}
                  style={{
                    padding: '6px 12px',
                    borderRadius: 4,
                    border: 'none',
                    cursor: 'pointer',
                    fontSize: 12,
                    fontWeight: 500,
                    backgroundColor: sortBy === opt ? '#30363d' : 'transparent',
                    color: sortBy === opt ? '#e6edf3' : '#8b949e',
                    transition: 'all 0.15s',
                  }}
                >
                  {opt === 'latest' ? 'Recent' : opt === 'commits' ? 'Commits' : 'Name'}
                </button>
              ))}
            </div>
          </div>

          {/* Repo list */}
          {loading ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              {[1,2,3,4,5].map(i => (
                <div key={i} style={{
                  height: 72,
                  backgroundColor: '#161b22',
                  borderBottom: '1px solid #21262d',
                  animation: 'pulse 2s ease-in-out infinite',
                }} />
              ))}
            </div>
          ) : filteredRepos.length === 0 ? (
            <EmptyState
              illustration={searchTerm ? NoSearchResults : EmptyRepository}
              title={searchTerm ? 'No matching repositories' : 'No repositories yet'}
              description={searchTerm 
                ? 'Try a different search term' 
                : 'Push your first signed commit to get started'}
              size="lg"
            />
          ) : (
            <>
              <div style={{
                border: '1px solid #21262d',
                borderRadius: 8,
                overflow: 'hidden',
                backgroundColor: '#0d1117',
              }}>
                {filteredRepos.map((repo, idx) => (
                  <Link
                    key={`${repo.address}/${repo.name}`}
                    href={`/explore/${repo.address}/${repo.name}`}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      padding: '16px 20px',
                      borderBottom: idx < filteredRepos.length - 1 ? '1px solid #21262d' : 'none',
                      textDecoration: 'none',
                      color: 'inherit',
                      transition: 'background-color 0.12s ease',
                      gap: 16,
                    }}
                    onMouseEnter={e => (e.currentTarget.style.backgroundColor = '#161b22')}
                    onMouseLeave={e => (e.currentTarget.style.backgroundColor = 'transparent')}
                  >
                    {/* Repo icon */}
                    <div style={{
                      width: 32, height: 32,
                      borderRadius: 6,
                      backgroundColor: '#21262d',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      flexShrink: 0,
                      fontSize: 14,
                    }}>
                      📦
                    </div>

                    {/* Name + desc */}
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginBottom: 2 }}>
                        <span style={{ color: '#58a6ff', fontSize: 15, fontWeight: 600 }}>
                          {repo.name}
                        </span>
                        <span style={{ color: '#484f58', fontSize: 12 }}>
                          {formatAddress(repo.owner_address)}
                        </span>
                      </div>
                      {repo.description && (
                        <p style={{
                          color: '#8b949e', fontSize: 13, margin: 0,
                          whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
                        }}>
                          {repo.description.replace(/\n/g, ' ').trim()}
                        </p>
                      )}
                    </div>

                    {/* Stats pills */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexShrink: 0, fontSize: 12, color: '#8b949e' }}>
                      {repo.contributor_count > 0 && (
                        <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                          <span style={{ fontSize: 10 }}>👤</span> {repo.contributor_count}
                        </span>
                      )}
                      <span style={{
                        display: 'flex', alignItems: 'center', gap: 4,
                        color: repo.commit_count > 10 ? '#58a6ff' : '#8b949e',
                      }}>
                        ⬡ {repo.commit_count}
                      </span>
                      {repo.last_commit_date && (
                        <span style={{ color: '#484f58', whiteSpace: 'nowrap' }}>
                          {formatTimeAgo(repo.last_commit_date)}
                        </span>
                      )}
                    </div>
                  </Link>
                ))}
              </div>

              {/* Show demo repos toggle */}
              {demoRepos.length > 0 && (
                <button
                  onClick={() => setShowDemo(!showDemo)}
                  style={{
                    display: 'block',
                    margin: '12px auto 0',
                    padding: '6px 16px',
                    backgroundColor: 'transparent',
                    border: '1px solid #30363d',
                    borderRadius: 20,
                    color: '#8b949e',
                    fontSize: 12,
                    cursor: 'pointer',
                    transition: 'all 0.15s',
                  }}
                >
                  {showDemo ? 'Hide demo repos' : `Show all (+${demoRepos.length} demo)`}
                </button>
              )}
            </>
          )}
        </main>

        {/* Sidebar — right */}
        <aside style={{ fontSize: 14 }}>
          {/* Recent Activity */}
          <div style={{
            padding: '16px',
            backgroundColor: '#161b22',
            border: '1px solid #21262d',
            borderRadius: 8,
            marginBottom: 16,
          }}>
            <h3 style={{
              fontSize: 12, fontWeight: 600, color: '#8b949e',
              textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: 12,
            }}>
              Recent Activity
            </h3>
            {deduped.length === 0 ? (
              <p style={{ color: '#484f58', fontSize: 13 }}>No recent activity</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                {deduped.slice(0, 8).map(item => (
                  <div key={item.id} style={{ lineHeight: 1.4 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <div style={{
                        width: 6, height: 6, borderRadius: '50%',
                        backgroundColor: '#3fb950', flexShrink: 0,
                      }} />
                      <Link
                        href={`/explore/${item.address}/${item.name}`}
                        style={{ color: '#58a6ff', textDecoration: 'none', fontWeight: 500, fontSize: 13 }}
                      >
                        {item.name}
                      </Link>
                    </div>
                    {item.commit_message && (
                      <p style={{ color: '#8b949e', margin: '2px 0 0 12px', fontSize: 12 }}>
                        {truncateMessage(item.commit_message, 55)}
                      </p>
                    )}
                    <span style={{ color: '#484f58', fontSize: 11, marginLeft: 12 }}>
                      {formatTimeAgo(item.pushed_at)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* About */}
          <div style={{
            padding: '16px',
            backgroundColor: '#161b22',
            border: '1px solid #21262d',
            borderRadius: 8,
          }}>
            <h3 style={{
              fontSize: 12, fontWeight: 600, color: '#8b949e',
              textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: 12,
            }}>
              About repo.box
            </h3>
            <p style={{ color: '#8b949e', fontSize: 13, lineHeight: 1.5, margin: 0 }}>
              Git hosting where every commit is EVM-signed and every push is permission-checked. 
              Built for AI agents with on-chain identity.
            </p>
            <div style={{ marginTop: 12, display: 'flex', flexDirection: 'column', gap: 6 }}>
              <Link href="/docs" style={{ color: '#58a6ff', textDecoration: 'none', fontSize: 13 }}>
                → Documentation
              </Link>
              <Link href="/playground" style={{ color: '#58a6ff', textDecoration: 'none', fontSize: 13 }}>
                → Try the playground
              </Link>
            </div>
          </div>
        </aside>
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
        @media (max-width: 900px) {
          /* Stack on mobile/tablet */
        }
      `}</style>
    </div>
  );
}
