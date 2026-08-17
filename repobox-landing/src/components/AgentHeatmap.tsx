"use client";

import { useState, useEffect } from 'react';

interface HeatmapData {
  startDate: string;
  endDate: string;
  data: {
    [date: string]: {
      count: number;
      level: "empty" | "light" | "medium" | "intense";
      repos: string;
      repoCount: number;
    };
  };
}

interface HeatmapDay {
  date: string;
  count: number;
  level: "empty" | "light" | "medium" | "intense";
  repos: string;
  repoCount: number;
}

const AgentHeatmap = () => {
  const [data, setData] = useState<HeatmapData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch('/api/activity-heatmap');
        if (!response.ok) {
          throw new Error(`Failed to fetch heatmap data: ${response.status}`);
        }
        const result: HeatmapData = await response.json();
        setData(result);
      } catch (err) {
        console.error('Error fetching heatmap data:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  if (loading) {
    return (
      <div className="w-full max-w-6xl mx-auto p-6">
        <div className="animate-pulse">
          <div className="h-6 bg-gray-200 rounded w-48 mb-4"></div>
          <div className="grid gap-1 grid-cols-53" style={{ gridTemplateColumns: 'repeat(53, 1fr)' }}>
            {Array.from({ length: 371 }).map((_, i) => (
              <div key={i} className="w-3 h-3 bg-gray-100 rounded-sm"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="w-full max-w-6xl mx-auto p-6">
        <div className="text-red-600 bg-red-50 p-4 rounded-lg">
          <h3 className="font-semibold mb-2">Failed to Load Activity Data</h3>
          <p className="text-sm">{error || 'Unknown error occurred'}</p>
        </div>
      </div>
    );
  }

  // Generate 365 days of data starting from startDate
  const generateDays = (): HeatmapDay[] => {
    const days: HeatmapDay[] = [];
    const start = new Date(data.startDate);
    
    for (let i = 0; i < 365; i++) {
      const currentDate = new Date(start);
      currentDate.setDate(start.getDate() + i);
      const dateStr = currentDate.toISOString().split('T')[0];
      
      const dayData = data.data[dateStr] || {
        count: 0,
        level: "empty" as const,
        repos: "",
        repoCount: 0
      };
      
      days.push({
        date: dateStr,
        ...dayData
      });
    }
    
    return days;
  };

  const days = generateDays();

  // Calculate summary statistics
  const totalCommits = Object.values(data.data).reduce((sum, day) => sum + day.count, 0);
  const activeDays = Object.values(data.data).filter(day => day.count > 0).length;

  // Color mapping for activity levels
  const getColorClass = (level: string): string => {
    switch (level) {
      case 'empty': return 'bg-gray-100 dark:bg-gray-800';
      case 'light': return 'bg-green-200 dark:bg-green-900';
      case 'medium': return 'bg-green-400 dark:bg-green-700';
      case 'intense': return 'bg-green-600 dark:bg-green-500';
      default: return 'bg-gray-100 dark:bg-gray-800';
    }
  };

  // Format date for display
  const formatDate = (dateStr: string): string => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  };

  return (
    <div className="w-full max-w-6xl mx-auto p-6">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Repository Activity
        </h2>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          {totalCommits.toLocaleString()} commits logged on {activeDays} days across tracked repos
        </p>
      </div>

      {/* Desktop heatmap */}
      <div className="hidden md:block mb-8">
        <div className="overflow-x-auto">
          <div 
            className="grid gap-1 min-w-max"
            style={{ 
              gridTemplateColumns: 'repeat(53, 12px)',
              gridTemplateRows: 'repeat(7, 12px)'
            }}
          >
            {/* Week labels */}
            <div className="col-span-53 grid grid-cols-53 gap-1 mb-2">
              {['Jan', '', '', '', 'Feb', '', '', '', 'Mar', '', '', '', 'Apr', '', '', '', 'May', '', '', '', 'Jun', '', '', '', 'Jul', '', '', '', 'Aug', '', '', '', 'Sep', '', '', '', 'Oct', '', '', '', 'Nov', '', '', '', 'Dec', '', '', '', '', '', '', '', '', ''].map((month, i) => (
                <div key={i} className="text-xs text-gray-500 text-center">
                  {month}
                </div>
              ))}
            </div>

            {/* Days grid */}
            {Array.from({ length: 7 }).map((_, dayOfWeek) => (
              Array.from({ length: 53 }).map((_, week) => {
                const dayIndex = week * 7 + dayOfWeek;
                const day = days[dayIndex];
                
                if (!day) {
                  return <div key={`${dayOfWeek}-${week}`} className="w-3 h-3" />;
                }
                
                return (
                  <div
                    key={`${dayOfWeek}-${week}`}
                    className={`w-3 h-3 rounded-sm cursor-pointer transition-all duration-200 hover:scale-110 ${getColorClass(day.level)}`}
                    title={`${formatDate(day.date)}\n${day.count} commit${day.count !== 1 ? 's' : ''}${day.repos ? `\nRepos: ${day.repos}` : ''}`}
                  />
                );
              })
            )).flat()}
          </div>
        </div>

        {/* Legend */}
        <div className="flex items-center justify-between mt-4">
          <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
            <span>Less</span>
            <div className="flex gap-1">
              <div className="w-3 h-3 rounded-sm bg-gray-100 dark:bg-gray-800" />
              <div className="w-3 h-3 rounded-sm bg-green-200 dark:bg-green-900" />
              <div className="w-3 h-3 rounded-sm bg-green-400 dark:bg-green-700" />
              <div className="w-3 h-3 rounded-sm bg-green-600 dark:bg-green-500" />
            </div>
            <span>More</span>
          </div>
        </div>
      </div>

      {/* Mobile simplified view */}
      <div className="md:hidden mb-8">
        <div className="grid grid-cols-12 gap-1">
          {/* Show recent 84 days (12 weeks) on mobile */}
          {days.slice(-84).map((day, index) => (
            <div
              key={day.date}
              className={`aspect-square rounded-sm cursor-pointer transition-all duration-200 ${getColorClass(day.level)}`}
              title={`${formatDate(day.date)}\n${day.count} commit${day.count !== 1 ? 's' : ''}${day.repos ? `\nRepos: ${day.repos}` : ''}`}
            />
          ))}
        </div>
        
        {/* Mobile legend */}
        <div className="flex items-center justify-center gap-2 mt-4 text-sm text-gray-600 dark:text-gray-400">
          <span>Less</span>
          <div className="flex gap-1">
            <div className="w-3 h-3 rounded-sm bg-gray-100 dark:bg-gray-800" />
            <div className="w-3 h-3 rounded-sm bg-green-200 dark:bg-green-900" />
            <div className="w-3 h-3 rounded-sm bg-green-400 dark:bg-green-700" />
            <div className="w-3 h-3 rounded-sm bg-green-600 dark:bg-green-500" />
          </div>
          <span>More</span>
        </div>
        
        <p className="text-center text-sm text-gray-500 mt-2">
          Last 12 weeks shown on mobile
        </p>
      </div>
    </div>
  );
};

export default AgentHeatmap;
