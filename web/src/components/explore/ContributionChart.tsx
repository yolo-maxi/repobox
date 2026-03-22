'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import EmptyState from '@/components/EmptyState';
import { EmptyTimeline } from '@/components/illustrations';
import { formatAddress } from '@/lib/utils';

interface ContributorActivity {
  address: string;
  dailyCommits: Record<string, number>; // YYYY-MM-DD -> count
  totalCommits: number;
  firstCommit: string;
  lastCommit: string;
}

interface ContributionChartProps {
  contributors: Array<{
    address: string;
    pushCount: number;
    lastPush: string;
    isOwner: boolean;
  }>;
  address: string;
  name: string;
  branch?: string;
  timeRange?: 'week' | 'month' | 'year';
}

interface TooltipState {
  visible: boolean;
  x: number;
  y: number;
  date: string;
  count: number;
  address: string;
}


function formatDateLabel(dateStr: string): string {
  const date = new Date(dateStr + 'T00:00:00');
  return date.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' });
}

function generateDateRange(range: 'week' | 'month' | 'year'): string[] {
  const dates: string[] = [];
  const today = new Date();
  
  let days: number;
  switch (range) {
    case 'week': days = 7; break;
    case 'month': days = 30; break;
    case 'year': days = 365; break;
    default: days = 30;
  }

  for (let i = days - 1; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    dates.push(date.toISOString().split('T')[0]);
  }
  
  return dates;
}

function getCommitColorIntensity(count: number, maxCount: number): string {
  if (count === 0) return 'rgb(22, 27, 34)'; // No activity
  
  const intensity = Math.min(count / maxCount, 1);
  
  if (intensity <= 0.25) return 'rgb(14, 68, 41)';   // Low activity
  if (intensity <= 0.5) return 'rgb(0, 109, 50)';    // Medium-low
  if (intensity <= 0.75) return 'rgb(38, 166, 65)';  // Medium-high  
  return 'rgb(57, 211, 83)';                         // High activity
}

export default function ContributionChart({ 
  contributors, 
  address, 
  name, 
  branch = 'HEAD',
  timeRange = 'month' 
}: ContributionChartProps) {
  const [activity, setActivity] = useState<ContributorActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedRange, setSelectedRange] = useState<'week' | 'month' | 'year'>(timeRange);
  const [tooltip, setTooltip] = useState<TooltipState>({ visible: false, x: 0, y: 0, date: '', count: 0, address: '' });
  const chartRef = useRef<HTMLDivElement>(null);

  const handleDayHover = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    const target = e.currentTarget;
    const rect = chartRef.current?.getBoundingClientRect();
    const cellRect = target.getBoundingClientRect();
    if (!rect) return;
    const x = cellRect.left + cellRect.width / 2 - rect.left;
    const y = cellRect.top - rect.top;
    const date = target.dataset.date || '';
    const count = parseInt(target.dataset.count || '0', 10);
    const addr = target.dataset.addr || '';
    setTooltip({ visible: true, x, y, date, count, address: addr });
  }, []);

  const handleDayLeave = useCallback(() => {
    setTooltip(prev => ({ ...prev, visible: false }));
  }, []);

  useEffect(() => {
    async function fetchActivity() {
      try {
        setLoading(true);
        setError(null);
        
        const branchParam = branch !== 'HEAD' ? `?branch=${encodeURIComponent(branch)}` : '';
        const response = await fetch(`/api/explorer/repos/${encodeURIComponent(address)}/${encodeURIComponent(name)}/activity${branchParam}`);
        
        if (!response.ok) {
          throw new Error(`Failed to fetch activity: ${response.status}`);
        }
        
        const data = await response.json();
        setActivity(data.activity || []);
      } catch (err) {
        console.error('Error fetching contribution activity:', err);
        setError(err instanceof Error ? err.message : 'Failed to load activity data');
      } finally {
        setLoading(false);
      }
    }

    fetchActivity();
  }, [address, name, branch, selectedRange]);

  const dateRange = generateDateRange(selectedRange);
  
  // Calculate max commits per day for color scaling
  const maxCommitsPerDay = Math.max(
    1, // Ensure at least 1 for division
    ...activity.flatMap(contrib => 
      Object.values(contrib.dailyCommits)
    )
  );

  if (loading) {
    return (
      <div className="contribution-chart">
        <div className="contribution-chart-header">
          <h4>Contribution Activity</h4>
          <div className="contribution-chart-loading">Loading activity data...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="contribution-chart">
        <div className="contribution-chart-header">
          <h4>Contribution Activity</h4>
          <div className="contribution-chart-error">
            Unable to load activity data: {error}
          </div>
        </div>
      </div>
    );
  }

  // Generate month labels for the date range
  const monthLabels: Array<{ label: string; index: number }> = [];
  let lastMonth = '';
  dateRange.forEach((date, i) => {
    const month = new Date(date + 'T00:00:00').toLocaleDateString('en-US', { month: 'short' });
    if (month !== lastMonth) {
      monthLabels.push({ label: month, index: i });
      lastMonth = month;
    }
  });

  // Total commits across all contributors
  const totalCommits = activity.reduce((sum, c) => sum + c.totalCommits, 0);

  return (
    <div className="contribution-chart" ref={chartRef}>
      <div className="contribution-chart-header">
        <div>
          <h4>Contribution Activity</h4>
          <p className="contribution-chart-subtitle">
            {totalCommits} commit{totalCommits !== 1 ? 's' : ''} by {activity.length} contributor{activity.length !== 1 ? 's' : ''} in the last {selectedRange}. Each square is one day — darker means more commits.
          </p>
        </div>
        <div className="contribution-chart-range-controls">
          {['week', 'month', 'year'].map(range => (
            <button
              key={range}
              onClick={() => setSelectedRange(range as typeof selectedRange)}
              className={`contribution-chart-range-btn ${selectedRange === range ? 'active' : ''}`}
            >
              {range}
            </button>
          ))}
        </div>
      </div>

      {activity.length === 0 ? (
        <EmptyState
          illustration={EmptyTimeline}
          title="No contribution activity"
          description={`No commits found for the selected ${selectedRange}.`}
          size="md"
        />
      ) : (
        <div className="contribution-chart-grid">
          {/* Month labels row */}
          {selectedRange !== 'week' && (
            <div className="contribution-chart-row contribution-chart-month-row">
              <div className="contribution-chart-contributor" />
              <div className="contribution-chart-months">
                {monthLabels.map(({ label, index }) => (
                  <span
                    key={`${label}-${index}`}
                    className="contribution-chart-month-label"
                    style={{ left: `${(index / dateRange.length) * 100}%` }}
                  >
                    {label}
                  </span>
                ))}
              </div>
            </div>
          )}

          {activity.map((contributor) => (
            <div key={contributor.address} className="contribution-chart-row">
              <div className="contribution-chart-contributor">
                <code>{formatAddress(contributor.address)}</code>
                <span className="contribution-chart-total">
                  {contributor.totalCommits} commit{contributor.totalCommits !== 1 ? 's' : ''}
                </span>
              </div>
              <div className="contribution-chart-days">
                {dateRange.map((date) => {
                  const commitCount = contributor.dailyCommits[date] || 0;
                  const backgroundColor = getCommitColorIntensity(commitCount, maxCommitsPerDay);
                  
                  return (
                    <div
                      key={date}
                      className="contribution-chart-day"
                      style={{ backgroundColor }}
                      data-date={date}
                      data-count={commitCount}
                      data-addr={contributor.address}
                      onMouseEnter={handleDayHover}
                      onMouseLeave={handleDayLeave}
                    />
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Floating tooltip */}
      {tooltip.visible && (
        <div
          className="contribution-chart-tooltip"
          style={{
            left: tooltip.x,
            top: tooltip.y,
          }}
        >
          <strong>{tooltip.count} commit{tooltip.count !== 1 ? 's' : ''}</strong>
          <span>{formatDateLabel(tooltip.date)}</span>
          <span className="contribution-chart-tooltip-addr">{formatAddress(tooltip.address)}</span>
        </div>
      )}

      <div className="contribution-chart-legend">
        <span className="contribution-chart-legend-label">No commits</span>
        <div className="contribution-chart-legend-colors">
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(22, 27, 34)' }} title="No activity" />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(14, 68, 41)' }} title="1-2 commits" />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(0, 109, 50)' }} title="3-5 commits" />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(38, 166, 65)' }} title="6-9 commits" />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(57, 211, 83)' }} title="10+ commits" />
        </div>
        <span className="contribution-chart-legend-label">Many commits</span>
      </div>
    </div>
  );
}