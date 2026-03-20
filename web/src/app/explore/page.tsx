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
  const [sortBy, setSortBy] = useState<'latest' | 'commits' | 'stars'>('latest');

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, reposRes, activityRes] = await Promise.all([
          fetch('/api/explorer/stats'),
          fetch(`/api/explorer/repos?sort=${sortBy}&limit=50`),
          fetch('/api/explorer/activity?limit=10')
        ]);

        if (statsRes.ok) {
          const statsData = await statsRes.json();
          setStats(statsData);
        }

        if (reposRes.ok) {
          const reposData = await reposRes.json();
          setRepos(reposData.repos || []);
        }

        if (activityRes.ok) {
          const activityData = await activityRes.json();
          setActivity(activityData.activity || []);
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

  if (loading) {
    return (
      <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)] p-8">
        <div className="max-w-6xl mx-auto">
          <div className="text-center py-20">
            <div className="animate-pulse text-[var(--bp-accent)]">Loading...</div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)]">
      {/* Header */}
      <header className="border-b border-[var(--bp-border)] bg-[var(--bp-surface)]">
        <div className="max-w-6xl mx-auto px-8 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-[var(--bp-heading)] mb-2">
                repo<span className="logo-dot">.</span>box explorer
              </h1>
              <p className="text-[var(--bp-dim)] text-sm">
                Browse decentralized git repositories
              </p>
            </div>
            
            <div className="flex items-center space-x-4">
              <div className="relative">
                <input
                  type="text"
                  placeholder="Search repos..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="bg-[var(--bp-bg)] border border-[var(--bp-border)] rounded px-3 py-2 text-sm w-64 focus:border-[var(--bp-accent)] outline-none"
                />
              </div>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-8 py-8">
        {/* Stats Bar */}
        {stats && (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-6">
              <div className="text-2xl font-bold text-[var(--bp-accent)]">
                {stats.totalRepos}
              </div>
              <div className="text-sm text-[var(--bp-dim)]">Total Repositories</div>
            </div>
            <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-6">
              <div className="text-2xl font-bold text-[var(--bp-accent)]">
                {stats.totalOwners}
              </div>
              <div className="text-sm text-[var(--bp-dim)]">Unique Owners</div>
            </div>
            <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-6">
              <div className="text-2xl font-bold text-[var(--bp-accent)]">
                {stats.totalCommits}
              </div>
              <div className="text-sm text-[var(--bp-dim)]">Total Commits</div>
            </div>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Repositories */}
          <div className="lg:col-span-2">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-[var(--bp-heading)]">Repositories</h2>
              <div className="flex space-x-2">
                {['latest', 'commits', 'stars'].map((sort) => (
                  <button
                    key={sort}
                    onClick={() => setSortBy(sort as any)}
                    className={`px-3 py-1 text-sm rounded border ${
                      sortBy === sort
                        ? 'bg-[var(--bp-accent)] text-[var(--bp-bg)] border-[var(--bp-accent)]'
                        : 'border-[var(--bp-border)] text-[var(--bp-dim)] hover:text-[var(--bp-text)]'
                    }`}
                  >
                    {sort}
                  </button>
                ))}
              </div>
            </div>

            {filteredRepos.length === 0 ? (
              <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-12 text-center">
                <div className="text-[var(--bp-dim)] text-lg mb-2">No repos yet</div>
                <div className="text-[var(--bp-dim)] text-sm">
                  Push your first repo to get started!
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {filteredRepos.map((repo) => (
                  <Link
                    key={`${repo.address}/${repo.name}`}
                    href={`/explore/${repo.address}/${repo.name}`}
                    className="block bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-6 hover:border-[var(--bp-accent)] transition-colors"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-2">
                          <h3 className="font-bold text-[var(--bp-heading)]">
                            {repo.name}
                          </h3>
                          <span className="text-[var(--bp-dim)] text-sm font-mono">
                            {formatAddress(repo.address)}
                          </span>
                        </div>
                        
                        {repo.description && (
                          <p className="text-[var(--bp-text)] text-sm mb-3 line-clamp-2">
                            {repo.description}
                          </p>
                        )}
                        
                        <div className="flex items-center space-x-4 text-xs text-[var(--bp-dim)]">
                          <span>{repo.commit_count} commits</span>
                          <span>Updated {formatTimeAgo(repo.last_commit_date)}</span>
                        </div>
                      </div>
                    </div>
                  </Link>
                ))}
              </div>
            )}
          </div>

          {/* Latest Activity */}
          <div>
            <h2 className="text-xl font-bold text-[var(--bp-heading)] mb-6">Latest Activity</h2>
            
            {activity.length === 0 ? (
              <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-8 text-center">
                <div className="text-[var(--bp-dim)] text-sm">No recent activity</div>
              </div>
            ) : (
              <div className="space-y-3">
                {activity.map((item) => (
                  <div
                    key={item.id}
                    className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-4"
                  >
                    <div className="flex items-start space-x-3">
                      <div className="text-[var(--bp-accent)]">📝</div>
                      <div className="flex-1 min-w-0">
                        <div className="text-sm">
                          <Link
                            href={`/explore/${item.address}/${item.name}`}
                            className="font-medium text-[var(--bp-heading)] hover:text-[var(--bp-accent)]"
                          >
                            {item.name}
                          </Link>
                        </div>
                        
                        {item.commit_message && (
                          <div className="text-xs text-[var(--bp-text)] mt-1 truncate">
                            {item.commit_message}
                          </div>
                        )}
                        
                        <div className="text-xs text-[var(--bp-dim)] mt-2">
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
      </div>
    </div>
  );
}