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
    <div className="explore-page">
      {/* Header */}
      <header className="explore-header">
        <div className="explore-header-content">
          <h1 className="explore-title">
            repo<span className="explore-title-dot">.</span>box
          </h1>
          <div className="explore-search">
            <input
              type="text"
              placeholder="Search repositories..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="explore-search-input"
            />
          </div>
        </div>
      </header>

      {loading ? (
        <div className="explore-loading">
          <div className="explore-loading-spinner"></div>
          <p>Loading repositories...</p>
        </div>
      ) : (
        <>
          {/* Stats */}
          {stats && (
            <section className="explore-stats">
              <div className="explore-stat-card">
                <div className="explore-stat-number">{stats.totalRepos.toLocaleString()}</div>
                <div className="explore-stat-label">Repositories</div>
              </div>
              <div className="explore-stat-card">
                <div className="explore-stat-number">{stats.totalOwners.toLocaleString()}</div>
                <div className="explore-stat-label">Owners</div>
              </div>
              <div className="explore-stat-card">
                <div className="explore-stat-number">{stats.totalCommits.toLocaleString()}</div>
                <div className="explore-stat-label">Commits</div>
              </div>
            </section>
          )}

          {/* Main Content */}
          <div className="explore-content">
            {/* Repositories */}
            <div className="explore-repos">
              <div className="explore-section-header">
                <h2 className="explore-section-title">Repositories</h2>
                <div className="explore-sort-tabs">
                  <button
                    onClick={() => setSortBy('latest')}
                    className={`explore-sort-tab ${sortBy === 'latest' ? 'active' : ''}`}
                  >
                    Latest
                  </button>
                  <button
                    onClick={() => setSortBy('commits')}
                    className={`explore-sort-tab ${sortBy === 'commits' ? 'active' : ''}`}
                  >
                    Commits
                  </button>
                </div>
              </div>

              {filteredRepos.length === 0 ? (
                <div className="explore-empty">
                  <h3>No repos yet</h3>
                  <p>Push your first signed commit to get started</p>
                  <div className="explore-code-snippet">
                    <code>
                      repobox init<br />
                      git add . && git commit -S -m "init"<br />
                      git push
                    </code>
                  </div>
                </div>
              ) : (
                <div className="explore-repo-list">
                  {filteredRepos.map((repo) => (
                    <Link
                      key={`${repo.address}/${repo.name}`}
                      href={`/explore/${repo.address}/${repo.name}`}
                      className="explore-repo-card"
                    >
                      <div className="explore-repo-header">
                        <h3 className="explore-repo-name">{repo.name}</h3>
                        <span className="explore-repo-owner">{formatAddress(repo.owner_address)}</span>
                      </div>
                      
                      {repo.description && (
                        <p className="explore-repo-description">
                          {repo.description.replace(/\n/g, ' ').trim()}
                        </p>
                      )}
                      
                      <div className="explore-repo-meta">
                        <span className="explore-repo-commits">{repo.commit_count} commits</span>
                        {repo.last_commit_date && (
                          <span className="explore-repo-updated">
                            Updated {formatTimeAgo(repo.last_commit_date)}
                          </span>
                        )}
                      </div>
                    </Link>
                  ))}
                </div>
              )}
            </div>

            {/* Activity Feed */}
            <div className="explore-activity">
              <h2 className="explore-section-title">Recent Activity</h2>
              
              {activity.length === 0 ? (
                <div className="explore-activity-empty">
                  <p>No recent activity</p>
                </div>
              ) : (
                <div className="explore-activity-list">
                  {activity.map((item) => (
                    <div key={item.id} className="explore-activity-item">
                      <Link
                        href={`/explore/${item.address}/${item.name}`}
                        className="explore-activity-repo"
                      >
                        {item.name}
                      </Link>
                      
                      {item.commit_message && (
                        <p className="explore-activity-message">{item.commit_message}</p>
                      )}
                      
                      <div className="explore-activity-meta">
                        <span>{formatTimeAgo(item.pushed_at)}</span>
                        {item.pusher_address && (
                          <span>by {formatAddress(item.pusher_address)}</span>
                        )}
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