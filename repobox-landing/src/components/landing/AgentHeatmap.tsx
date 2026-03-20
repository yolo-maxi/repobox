"use client";

import { useEffect, useState } from "react";

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

interface HeatmapCellProps {
  date: string;
  data: HeatmapData["data"][string];
  onHover: (info: string | null) => void;
}

const HeatmapCell: React.FC<HeatmapCellProps> = ({ date, data, onHover }) => {
  const getColor = (level: string) => {
    switch (level) {
      case "empty":
        return "bg-gray-100 dark:bg-gray-800";
      case "light":
        return "bg-green-200 dark:bg-green-900";
      case "medium":
        return "bg-green-400 dark:bg-green-700";
      case "intense":
        return "bg-green-600 dark:bg-green-500";
      default:
        return "bg-gray-100 dark:bg-gray-800";
    }
  };

  const formatTooltip = () => {
    if (data.count === 0) {
      return `${formatDate(date)}: No activity`;
    }
    const repoText = data.repoCount === 1 ? "repo" : "repos";
    return `${formatDate(date)}: ${data.count} commits across ${data.repoCount} ${repoText}`;
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  };

  return (
    <div
      className={`w-3 h-3 rounded-sm ${getColor(data.level)} cursor-pointer transition-all duration-150 hover:ring-2 hover:ring-gray-400`}
      onMouseEnter={() => onHover(formatTooltip())}
      onMouseLeave={() => onHover(null)}
      title={formatTooltip()}
    />
  );
};

export function AgentHeatmap() {
  const [heatmapData, setHeatmapData] = useState<HeatmapData | null>(null);
  const [hoveredInfo, setHoveredInfo] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch("/heatmap-data.json")
      .then((res) => res.json())
      .then((data) => {
        setHeatmapData(data);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load heatmap data:", err);
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <div className="w-full h-32 flex items-center justify-center">
        <div className="text-gray-500">Loading activity data...</div>
      </div>
    );
  }

  if (!heatmapData) {
    return (
      <div className="w-full h-32 flex items-center justify-center">
        <div className="text-gray-500">Failed to load activity data</div>
      </div>
    );
  }

  // Group dates into weeks
  const generateWeeks = () => {
    const weeks: string[][] = [];
    const dates = Object.keys(heatmapData.data).sort();
    
    let currentWeek: string[] = [];
    let currentWeekday = 0;

    // Find the starting weekday for proper alignment
    if (dates.length > 0) {
      const startDate = new Date(dates[0]);
      currentWeekday = startDate.getDay(); // 0 = Sunday, 6 = Saturday
      
      // Add empty cells for days before the start date
      for (let i = 0; i < currentWeekday; i++) {
        currentWeek.push("");
      }
    }

    dates.forEach((date) => {
      currentWeek.push(date);
      currentWeekday++;

      if (currentWeekday === 7) {
        weeks.push([...currentWeek]);
        currentWeek = [];
        currentWeekday = 0;
      }
    });

    // Add the final week if it has any dates
    if (currentWeek.length > 0) {
      weeks.push(currentWeek);
    }

    return weeks;
  };

  const weeks = generateWeeks();
  const totalCommits = Object.values(heatmapData.data).reduce((sum, day) => sum + day.count, 0);
  const activeDays = Object.values(heatmapData.data).filter(day => day.count > 0).length;

  return (
    <section>
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Agent Activity
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          {totalCommits} commits across {activeDays} days • Proof of work across repositories
        </p>
      </div>

      <div className="bg-white dark:bg-gray-900 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
        {/* Month labels */}
        <div className="mb-3">
          <div className="grid" style={{ gridTemplateColumns: `repeat(${Math.min(weeks.length, 53)}, 1fr)`, gap: "1px" }}>
            {weeks.slice(0, 53).map((_, weekIndex) => {
              // Show month label for first week of each month
              const firstDateInWeek = weeks[weekIndex]?.find(date => date !== "");
              if (firstDateInWeek) {
                const date = new Date(firstDateInWeek);
                const isFirstOfMonth = date.getDate() <= 7;
                if (isFirstOfMonth) {
                  return (
                    <div key={weekIndex} className="text-xs text-gray-500 text-center">
                      {date.toLocaleDateString("en-US", { month: "short" })}
                    </div>
                  );
                }
              }
              return <div key={weekIndex} />;
            })}
          </div>
        </div>

        {/* Day labels */}
        <div className="flex items-start">
          <div className="flex flex-col mr-3 text-xs text-gray-500" style={{ gap: "1px" }}>
            <div style={{ height: "12px" }}>Mon</div>
            <div style={{ height: "12px" }}></div>
            <div style={{ height: "12px" }}>Wed</div>
            <div style={{ height: "12px" }}></div>
            <div style={{ height: "12px" }}>Fri</div>
            <div style={{ height: "12px" }}></div>
            <div style={{ height: "12px" }}>Sun</div>
          </div>

          {/* Heatmap grid */}
          <div className="flex" style={{ gap: "1px" }}>
            {weeks.slice(0, 53).map((week, weekIndex) => (
              <div key={weekIndex} className="flex flex-col" style={{ gap: "1px" }}>
                {[0, 1, 2, 3, 4, 5, 6].map((dayIndex) => {
                  const date = week[dayIndex];
                  if (!date) {
                    return <div key={dayIndex} className="w-3 h-3" />;
                  }

                  const dayData = heatmapData.data[date];
                  if (!dayData) {
                    return <div key={dayIndex} className="w-3 h-3 bg-gray-100 rounded-sm" />;
                  }

                  return (
                    <HeatmapCell
                      key={date}
                      date={date}
                      data={dayData}
                      onHover={setHoveredInfo}
                    />
                  );
                })}
              </div>
            ))}
          </div>
        </div>

        {/* Legend */}
        <div className="mt-4 flex items-center justify-between">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            {hoveredInfo || "Hover over a cell for details"}
          </div>
          <div className="flex items-center space-x-1">
            <span className="text-xs text-gray-500 mr-2">Less</span>
            <div className="w-3 h-3 bg-gray-100 dark:bg-gray-800 rounded-sm" />
            <div className="w-3 h-3 bg-green-200 dark:bg-green-900 rounded-sm" />
            <div className="w-3 h-3 bg-green-400 dark:bg-green-700 rounded-sm" />
            <div className="w-3 h-3 bg-green-600 dark:bg-green-500 rounded-sm" />
            <span className="text-xs text-gray-500 ml-2">More</span>
          </div>
        </div>
      </div>
    </section>
  );
}