'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { formatTimeAgo } from '@/lib/utils';

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

function truncateMessage(message: string, maxLength: number = 60): string {
  if (message.length <= maxLength) return message;
  return message.substring(0, maxLength).trim() + '…';
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
          fetch('/api/explorer/activity?limit=8')
        ]);
        
        if (statsRes.ok) {
          setStats(await statsRes.json());
        }
        
        if (activityRes.ok) {
          const data = await activityRes.json();
          setActivity(data.activity || []);
        }
      } catch (error) {
        console.error('Error fetching sidebar data:', error);
      }
    };

    fetchData();

    // Refresh activity every 30 seconds
    const interval = setInterval(() => {
      fetch('/api/explorer/activity?limit=8')
        .then(res => res.ok ? res.json() : null)
        .then(data => { if (data?.activity) setActivity(data.activity); })
        .catch(() => {});
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  return (
    <aside className={`explore-sidebar ${className}`}>
      {/* Stats Overview */}
      {stats && (
        <div className="explore-sidebar-section">
          <h3 className="explore-sidebar-title">Overview</h3>
          <div className="explore-sidebar-stats">
            <div className="explore-sidebar-stat">
              <span className="explore-sidebar-stat-label">Repositories</span>
              <span className="explore-sidebar-stat-value">{stats.totalRepos}</span>
            </div>
            <div className="explore-sidebar-stat">
              <span className="explore-sidebar-stat-label">Owners</span>
              <span className="explore-sidebar-stat-value">{stats.totalOwners}</span>
            </div>
            <div className="explore-sidebar-stat">
              <span className="explore-sidebar-stat-label">Commits</span>
              <span className="explore-sidebar-stat-value">{stats.totalCommits}</span>
            </div>
          </div>
        </div>
      )}

      {/* Sort Options */}
      {showSort && onSortChange && (
        <div className="explore-sidebar-section">
          <h3 className="explore-sidebar-title">Sort by</h3>
          <div className="explore-sidebar-sort">
            {[
              { key: 'latest', label: '🕐 Recently updated' },
              { key: 'commits', label: '📊 Most commits' },
              { key: 'name', label: '🔤 Name' }
            ].map(({ key, label }) => (
              <button
                key={key}
                onClick={() => onSortChange(key)}
                className={`explore-sidebar-sort-btn ${sortBy === key ? 'active' : ''}`}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Recent Activity */}
      <div className="explore-sidebar-section">
        <h3 className="explore-sidebar-title">Recent Activity</h3>
        {activity.length === 0 ? (
          <p className="explore-sidebar-empty">No recent activity</p>
        ) : (
          <div className="explore-sidebar-activity">
            {activity.map(item => (
              <div key={item.id} className="explore-sidebar-activity-item">
                <Link
                  href={`/explore/${item.address}/${item.name}`}
                  className="explore-sidebar-activity-link"
                >
                  {item.name}
                </Link>
                {item.commit_message && (
                  <p className="explore-sidebar-activity-message">
                    {truncateMessage(item.commit_message)}
                  </p>
                )}
                <span className="explore-sidebar-activity-time">
                  {formatTimeAgo(item.pushed_at)}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </aside>
  );
}