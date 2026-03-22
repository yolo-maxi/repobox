'use client';

import { useState, useEffect, useMemo } from 'react';
import Link from 'next/link';
import { formatTimeAgo } from '@/lib/utils';
import EmptyState from '@/components/EmptyState';
import { QuietActivity } from '@/components/illustrations';

interface Stats {
  totalRepos: number;
  totalOwners: number;
  totalCommits: number;
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

interface ExploreSidebarProps {
  showSort?: boolean;
  sortBy?: string;
  onSortChange?: (sort: string) => void;
  className?: string;
}

function truncateMessage(message: string, maxLength: number = 55): string {
  if (message.length <= maxLength) return message;
  return message.substring(0, maxLength).trim() + '...';
}

export default function ExploreSidebar({
  showSort = false,
  sortBy = 'latest',
  onSortChange,
  className = ''
}: ExploreSidebarProps) {
  const [stats, setStats] = useState<Stats | null>(null);
  const [activity, setActivity] = useState<Activity[]>([]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, activityRes] = await Promise.all([
          fetch('/api/explorer/stats'),
          fetch('/api/explorer/activity?limit=15')
        ]);
        if (statsRes.ok) setStats(await statsRes.json());
        if (activityRes.ok) {
          const data = await activityRes.json();
          setActivity(data.activity || []);
        }
      } catch (error) {
        console.error('Error fetching sidebar data:', error);
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
  }, []);

  // Deduplicate activity: one entry per repo
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
    <aside className={`rd-sidebar ${className}`}>
      {/* Stats */}
      {stats && (
        <div className="rd-sidebar-section">
          <h3 className="rd-sidebar-title">Overview</h3>
          <div className="rd-sidebar-stats">
            <div className="rd-sidebar-stat">
              <span className="rd-sidebar-stat-val">{stats.totalRepos}</span>
              <span className="rd-sidebar-stat-label">Repos</span>
            </div>
            <div className="rd-sidebar-stat">
              <span className="rd-sidebar-stat-val">{stats.totalOwners}</span>
              <span className="rd-sidebar-stat-label">Owners</span>
            </div>
            <div className="rd-sidebar-stat">
              <span className="rd-sidebar-stat-val">{stats.totalCommits}</span>
              <span className="rd-sidebar-stat-label">Commits</span>
            </div>
          </div>
        </div>
      )}

      {/* Sort */}
      {showSort && onSortChange && (
        <div className="rd-sidebar-section">
          <h3 className="rd-sidebar-title">Sort by</h3>
          <div className="rd-sidebar-sort">
            {[
              { key: 'latest', label: 'Recently updated' },
              { key: 'commits', label: 'Most commits' },
              { key: 'name', label: 'Name' }
            ].map(({ key, label }) => (
              <button
                key={key}
                onClick={() => onSortChange(key)}
                className={`rd-sidebar-sort-btn ${sortBy === key ? 'rd-sidebar-sort-btn--active' : ''}`}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Activity */}
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
                      {truncateMessage(item.commit_message)}
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
  );
}
