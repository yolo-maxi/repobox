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

interface StatCardProps {
  title: string;
  value: string;
  label: string;
  icon?: string;
  className?: string;
}

function StatCard({ title, value, label, icon, className = '' }: StatCardProps) {
  return (
    <div className={`repo-stat-card ${className}`}>
      {icon && <div className="repo-stat-card-icon">{icon}</div>}
      <div className="repo-stat-card-content">
        <div className="repo-stat-card-value">{value}</div>
        <div className="repo-stat-card-label">{label}</div>
      </div>
    </div>
  );
}

interface LanguageBarProps {
  languages: LanguageStats[];
  showOthers?: boolean;
}

function LanguageBar({ languages, showOthers = false }: LanguageBarProps) {
  if (languages.length === 0) return null;

  const displayLanguages = languages.slice(0, 5);
  const othersPercentage = showOthers && languages.length > 5 
    ? languages.slice(5).reduce((sum, lang) => sum + lang.percentage, 0)
    : 0;

  return (
    <div className="repo-language-bar-container">
      <div className="repo-language-bar">
        {displayLanguages.map((lang, index) => (
          <div
            key={lang.name}
            className="repo-language-segment"
            style={{ 
              width: `${lang.percentage}%`,
              backgroundColor: lang.color
            }}
            title={`${lang.name}: ${lang.percentage.toFixed(1)}% (${lang.lines.toLocaleString()} lines)`}
          />
        ))}
        {showOthers && othersPercentage > 0 && (
          <div
            className="repo-language-segment"
            style={{ 
              width: `${othersPercentage}%`,
              backgroundColor: '#cccccc'
            }}
            title={`Other: ${othersPercentage.toFixed(1)}%`}
          />
        )}
      </div>

      <div className="repo-language-labels">
        {displayLanguages.map((lang) => (
          <div key={lang.name} className="repo-language-label">
            <span 
              className="repo-language-color-dot" 
              style={{ backgroundColor: lang.color }}
            />
            <span className="repo-language-name">{lang.name}</span>
            <span className="repo-language-percentage">{lang.percentage.toFixed(1)}%</span>
          </div>
        ))}
        {showOthers && othersPercentage > 0 && (
          <div className="repo-language-label">
            <span 
              className="repo-language-color-dot" 
              style={{ backgroundColor: '#cccccc' }}
            />
            <span className="repo-language-name">Other</span>
            <span className="repo-language-percentage">{othersPercentage.toFixed(1)}%</span>
          </div>
        )}
      </div>
    </div>
  );
}

function RepoStatsCardsSkeleton() {
  return (
    <div className="repo-stats-cards" data-testid="stats-skeleton">
      <div className="repo-stats-skeleton">
        <div className="repo-stats-skeleton-bar"></div>
        <div className="repo-stats-skeleton-grid">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="repo-stats-skeleton-card"></div>
          ))}
        </div>
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
        
        const data = await response.json();
        setStats(data);
      } catch (err) {
        console.error('Stats fetch error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load statistics');
      } finally {
        setLoading(false);
      }
    }

    fetchStats();
  }, [address, name, branch]);

  if (loading) {
    return <RepoStatsCardsSkeleton />;
  }

  if (error || !stats) {
    return (
      <div className="repo-stats-cards">
        <div className="repo-stats-error">
          <p>Unable to load repository statistics</p>
          {error && <p className="repo-stats-error-detail">{error}</p>}
        </div>
      </div>
    );
  }

  const hasLanguages = stats.language_breakdown.length > 0;
  const topLanguage = hasLanguages ? stats.language_breakdown[0] : null;

  return (
    <div className="repo-stats-cards">
      {/* Language Breakdown Bar */}
      {hasLanguages && (
        <div className="repo-stats-section">
          <LanguageBar 
            languages={stats.language_breakdown} 
            showOthers={stats.language_breakdown.length > 5}
          />
        </div>
      )}

      {/* Statistics Cards Grid */}
      <div className="repo-stats-grid">
        <StatCard
          title="Total Lines"
          value={stats.total_lines.toLocaleString()}
          label="SLOC"
          icon="📊"
        />
        
        <StatCard
          title="Contributors"
          value={stats.unique_signers.toString()}
          label="SIGNERS"
          icon="👥"
        />
        
        <StatCard
          title="Repository Age"
          value={`${stats.repository_age_days} days`}
          label="AGE"
          icon="📅"
        />
        
        <StatCard
          title="Latest Language"
          value={topLanguage?.name || "Unknown"}
          label="RECENT LANG"
          icon="🔤"
        />
      </div>

      {/* Computation Time Footer */}
      <div className="repo-stats-footer">
        <small>
          Computed in {stats.computation_time_ms}ms • 
          Last updated {new Date(stats.last_computed).toLocaleTimeString()}
        </small>
      </div>
    </div>
  );
}