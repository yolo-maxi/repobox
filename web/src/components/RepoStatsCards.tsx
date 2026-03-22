'use client';

import { useState, useEffect } from 'react';

interface LanguageStats {
  name: string;
  lines: number;
  files: number;
  percentage: number;
  color: string;
  extensions: string[];
}

interface RepoStats {
  language_breakdown: LanguageStats[];
  total_lines: number;
  total_files: number;
  unique_signers: number;
  repository_age_days: number;
  last_computed: string;
  computation_time_ms: number;
  branch: string;
}

interface RepoStatsCardsProps {
  address: string;
  name: string;
  branch?: string;
}

function LanguageBar({ languages }: { languages: LanguageStats[] }) {
  if (languages.length === 0) return null;
  const display = languages.slice(0, 6);

  return (
    <div className="rd-lang-section">
      {/* Horizontal bar */}
      <div className="rd-lang-bar">
        {display.map((lang) => (
          <div
            key={lang.name}
            className="rd-lang-segment"
            style={{ width: `${lang.percentage}%`, backgroundColor: lang.color }}
            title={`${lang.name}: ${lang.percentage.toFixed(1)}%`}
          />
        ))}
      </div>
      {/* Labels */}
      <div className="rd-lang-labels">
        {display.map((lang) => (
          <span key={lang.name} className="rd-lang-label">
            <span className="rd-lang-dot" style={{ backgroundColor: lang.color }} />
            <span className="rd-lang-name">{lang.name}</span>
            <span className="rd-lang-pct">{lang.percentage.toFixed(1)}%</span>
          </span>
        ))}
      </div>
    </div>
  );
}

function Skeleton() {
  return (
    <div className="rd-stats-skeleton">
      <div className="rd-stats-skeleton-bar" />
      <div className="rd-stats-skeleton-row">
        <div className="rd-stats-skeleton-chip" />
        <div className="rd-stats-skeleton-chip" />
        <div className="rd-stats-skeleton-chip" />
      </div>
    </div>
  );
}

export default function RepoStatsCards({ address, name, branch = 'HEAD' }: RepoStatsCardsProps) {
  const [stats, setStats] = useState<RepoStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchStats() {
      try {
        setLoading(true);
        setError(null);
        const branchParam = branch !== 'HEAD' ? `?branch=${encodeURIComponent(branch)}` : '';
        const response = await fetch(`/api/explorer/repos/${encodeURIComponent(address)}/${encodeURIComponent(name)}/stats${branchParam}`);
        if (!response.ok) {
          const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
          throw new Error(errorData.error || `HTTP ${response.status}`);
        }
        setStats(await response.json());
      } catch (err) {
        console.error('Stats fetch error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load statistics');
      } finally {
        setLoading(false);
      }
    }
    fetchStats();
  }, [address, name, branch]);

  if (loading) return <Skeleton />;

  if (error || !stats) {
    return (
      <div className="rd-stats-error">
        <p>Unable to load repository statistics</p>
        {error && <p className="rd-stats-error-detail">{error}</p>}
      </div>
    );
  }

  return (
    <div className="rd-stats-container">
      <LanguageBar languages={stats.language_breakdown} />
      <div className="rd-stats-row">
        <span className="rd-stat-chip">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M1.5 1.75V13.5h13.75a.75.75 0 0 1 0 1.5H.75a.75.75 0 0 1-.75-.75V1.75a.75.75 0 0 1 1.5 0Zm14.28 2.53-5.25 5.25a.75.75 0 0 1-1.06 0L7 7.06 4.28 9.78a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042l3.25-3.25a.75.75 0 0 1 1.06 0L10 7.94l4.72-4.72a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042Z"/>
          </svg>
          {stats.total_lines.toLocaleString()} SLOC
        </span>
        <span className="rd-stat-chip">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M10.561 8.073a6.005 6.005 0 0 1 3.432 5.142.75.75 0 1 1-1.498.07 4.5 4.5 0 0 0-8.99 0 .75.75 0 0 1-1.498-.07 6.004 6.004 0 0 1 3.431-5.142 3.999 3.999 0 1 1 5.123 0ZM10.5 5a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z"/>
          </svg>
          {stats.unique_signers} Signers
        </span>
        <span className="rd-stat-chip">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4.75 0a.75.75 0 0 1 .75.75V2h5V.75a.75.75 0 0 1 1.5 0V2h1.25c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 13.25 16H2.75A1.75 1.75 0 0 1 1 14.25V3.75C1 2.784 1.784 2 2.75 2H4V.75A.75.75 0 0 1 4.75 0ZM2.5 7.5v6.75c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25V7.5Zm10.75-4H2.75a.25.25 0 0 0-.25.25V6h11V3.75a.25.25 0 0 0-.25-.25Z"/>
          </svg>
          {stats.repository_age_days}d old
        </span>
      </div>
    </div>
  );
}
