'use client';

import { useState, useEffect } from 'react';
import EmptyState from '@/components/EmptyState';
import { EmptyTimeline } from '@/components/illustrations';

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

function formatAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
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

  return (
    <div className="contribution-chart">
      <div className="contribution-chart-header">
        <h4>Contribution Activity</h4>
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
          {activity.map((contributor) => (
            <div key={contributor.address} className="contribution-chart-row">
              <div className="contribution-chart-contributor">
                <code>{formatAddress(contributor.address)}</code>
                <span className="contribution-chart-total">
                  {contributor.totalCommits} commits
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
                      title={`${date}: ${commitCount} commits by ${formatAddress(contributor.address)}`}
                    />
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="contribution-chart-legend">
        <span>Less</span>
        <div className="contribution-chart-legend-colors">
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(22, 27, 34)' }} />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(14, 68, 41)' }} />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(0, 109, 50)' }} />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(38, 166, 65)' }} />
          <div className="contribution-chart-legend-color" style={{ backgroundColor: 'rgb(57, 211, 83)' }} />
        </div>
        <span>More</span>
      </div>
    </div>
  );
}