'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';

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
  const [sortBy, setSortBy] = useState<'latest' | 'commits'>('latest');

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, reposRes, activityRes] = await Promise.all([
          fetch('/api/explorer/stats'),
          fetch(`/api/explorer/repos?sort=${sortBy}&limit=50`),
          fetch('/api/explorer/activity?limit=10')
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
  }, [sortBy]);

  const filteredRepos = repos.filter(repo =>
    repo.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    repo.address.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="p-6 md:p-10">
      {/* Header */}
      <div className="mb-10">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 className="text-2xl md:text-3xl font-bold text-[var(--bp-heading)] mb-1">
              repo<span className="logo-dot">.</span>box
              <span className="text-[var(--bp-dim)] font-normal text-lg ml-3">explorer</span>
            </h1>
          </div>
          
          <input
            type="text"
            placeholder="Search repos or addresses..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="glass-input rounded-lg px-4 py-2.5 text-sm w-72 font-mono"
          />
        </div>
      </div>

      {loading ? (
        <div className="text-center py-20">
          <div className="animate-pulse text-[var(--bp-accent)] text-sm">Loading...</div>
        </div>
      ) : (
        <>
          {/* Stats */}
          {stats && (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-10">
              {[
                { label: 'Repositories', value: stats.totalRepos, icon: '📦' },
                { label: 'Unique Owners', value: stats.totalOwners, icon: '🔑' },
                { label: 'Total Commits', value: stats.totalCommits, icon: '📝' },
              ].map((stat) => (
                <div key={stat.label} className="glass-stat rounded-xl p-5">
                  <div className="flex items-center gap-3">
                    <span className="text-xl">{stat.icon}</span>
                    <div>
                      <div className="text-2xl font-bold text-[var(--bp-accent)] font-mono">
                        {stat.value.toLocaleString()}
                      </div>
                      <div className="text-xs text-[var(--bp-dim)] uppercase tracking-wider">
                        {stat.label}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {/* Repos */}
            <div className="lg:col-span-2">
              <div className="flex items-center justify-between mb-5">
                <h2 className="text-lg font-semibold text-[var(--bp-heading)]">Repositories</h2>
                <div className="flex gap-1.5">
                  {(['latest', 'commits'] as const).map((sort) => (
                    <button
                      key={sort}
                      onClick={() => setSortBy(sort)}
                      className={`glass-tab px-3 py-1.5 text-xs rounded-lg capitalize ${
                        sortBy === sort ? 'glass-tab-active' : ''
                      }`}
                    >
                      {sort}
                    </button>
                  ))}
                </div>
              </div>

              {filteredRepos.length === 0 ? (
                <div className="glass-panel-inner rounded-xl p-14 text-center">
                  <div className="text-4xl mb-4">🪸</div>
                  <div className="text-[var(--bp-heading)] text-lg mb-2">No repos yet</div>
                  <div className="text-[var(--bp-dim)] text-sm max-w-xs mx-auto">
                    Push your first signed commit to get started.
                  </div>
                  <pre className="mt-6 text-xs text-[var(--bp-dim)] font-mono bg-[rgba(0,0,0,0.2)] rounded-lg p-4 inline-block text-left">
{`repobox init
git add . && git commit -S -m "init"
git push`}
                  </pre>
                </div>
              ) : (
                <div className="space-y-3">
                  {filteredRepos.map((repo) => (
                    <Link
                      key={`${repo.address}/${repo.name}`}
                      href={`/explore/${repo.address}/${repo.name}`}
                      className="glass-card block rounded-xl p-5"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2.5 mb-1.5">
                            <h3 className="font-semibold text-[var(--bp-heading)]">
                              {repo.name}
                            </h3>
                            <span className="text-[var(--bp-dim)] text-xs font-mono">
                              {formatAddress(repo.owner_address)}
                            </span>
                          </div>
                          
                          {repo.description && (
                            <p className="text-[var(--bp-text)] text-sm mb-2 line-clamp-1 opacity-80">
                              {repo.description}
                            </p>
                          )}
                          
                          <div className="flex items-center gap-4 text-xs text-[var(--bp-dim)]">
                            <span className="font-mono">{repo.commit_count} commits</span>
                            {repo.last_commit_date && (
                              <span>Updated {formatTimeAgo(repo.last_commit_date)}</span>
                            )}
                          </div>
                        </div>
                      </div>
                    </Link>
                  ))}
                </div>
              )}
            </div>

            {/* Activity */}
            <div>
              <h2 className="text-lg font-semibold text-[var(--bp-heading)] mb-5">Latest Activity</h2>
              
              {activity.length === 0 ? (
                <div className="glass-panel-inner rounded-xl p-8 text-center">
                  <div className="text-[var(--bp-dim)] text-sm">No recent activity</div>
                </div>
              ) : (
                <div className="space-y-2.5">
                  {activity.map((item) => (
                    <div key={item.id} className="glass-card rounded-lg p-4">
                      <div className="flex items-start gap-3">
                        <div className="text-[var(--bp-accent)] text-sm mt-0.5">⟫</div>
                        <div className="flex-1 min-w-0">
                          <Link
                            href={`/explore/${item.address}/${item.name}`}
                            className="text-sm font-medium text-[var(--bp-heading)] hover:text-[var(--bp-accent)] transition-colors"
                          >
                            {item.name}
                          </Link>
                          
                          {item.commit_message && (
                            <div className="text-xs text-[var(--bp-text)] mt-1 truncate opacity-70">
                              {item.commit_message}
                            </div>
                          )}
                          
                          <div className="text-xs text-[var(--bp-dim)] mt-1.5 font-mono">
                            {formatTimeAgo(item.pushed_at)}
                            {item.pusher_address && (
                              <span className="ml-2">
                                by {formatAddress(item.pusher_address)}
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
