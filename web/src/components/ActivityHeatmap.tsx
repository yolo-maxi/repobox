interface ActivityDay {
  day: string;
  count: number;
}

interface ActivityHeatmapProps {
  activity: ActivityDay[];
  className?: string;
}

function formatDateTooltip(date: string, count: number): string {
  const d = new Date(date);
  const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
    'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  
  const day = d.getDate();
  const month = monthNames[d.getMonth()];
  const year = d.getFullYear();
  
  if (count === 0) {
    return `No commits on ${month} ${day}, ${year}`;
  } else if (count === 1) {
    return `1 commit on ${month} ${day}, ${year}`;
  } else {
    return `${count} commits on ${month} ${day}, ${year}`;
  }
}

function getIntensityLevel(count: number): number {
  if (count === 0) return 0;
  if (count <= 2) return 1;
  if (count <= 4) return 2;
  if (count <= 7) return 3;
  return 4;
}

export default function ActivityHeatmap({ activity, className = '' }: ActivityHeatmapProps) {
  // Generate the last 90 days grid
  const today = new Date();
  const ninetyDaysAgo = new Date(today);
  ninetyDaysAgo.setDate(today.getDate() - 89); // 90 days including today
  
  // Create a map of activity data for quick lookup
  const activityMap = new Map<string, number>();
  activity.forEach(day => {
    activityMap.set(day.day, day.count);
  });
  
  // Generate all days for the heatmap
  const days: { date: string; count: number; level: number }[] = [];
  const currentDate = new Date(ninetyDaysAgo);
  
  for (let i = 0; i < 90; i++) {
    const dateStr = currentDate.toISOString().split('T')[0];
    const count = activityMap.get(dateStr) || 0;
    const level = getIntensityLevel(count);
    
    days.push({
      date: dateStr,
      count,
      level
    });
    
    currentDate.setDate(currentDate.getDate() + 1);
  }
  
  // Organize days into weeks
  const weeks: Array<Array<{ date: string; count: number; level: number }>> = [];
  let currentWeek: Array<{ date: string; count: number; level: number }> = [];
  
  days.forEach((day, index) => {
    const date = new Date(day.date);
    const dayOfWeek = date.getDay(); // 0 = Sunday, 1 = Monday, etc.
    
    if (index === 0 && dayOfWeek !== 0) {
      // Fill the beginning of the first week with empty cells
      for (let i = 0; i < dayOfWeek; i++) {
        currentWeek.push({ date: '', count: 0, level: 0 });
      }
    }
    
    currentWeek.push(day);
    
    if (dayOfWeek === 6 || index === days.length - 1) { // Saturday or last day
      // Fill the end of the last week with empty cells if needed
      while (currentWeek.length < 7) {
        currentWeek.push({ date: '', count: 0, level: 0 });
      }
      weeks.push(currentWeek);
      currentWeek = [];
    }
  });
  
  const dayLabels = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
  const totalCommits = activity.reduce((sum, day) => sum + day.count, 0);
  
  return (
    <div className={`explore-activity-heatmap ${className}`}>
      <div className="explore-activity-heatmap-header">
        <h3 className="explore-activity-heatmap-title">
          {totalCommits} contributions in the last 90 days
        </h3>
      </div>
      
      <div className="explore-activity-heatmap-grid">
        {/* Day labels */}
        <div className="explore-activity-heatmap-day-labels">
          {dayLabels.map((label, index) => (
            <div key={index} className="explore-activity-heatmap-day-label">
              {label}
            </div>
          ))}
        </div>
        
        {/* Weeks grid */}
        <div className="explore-activity-heatmap-weeks">
          {weeks.map((week, weekIndex) => (
            <div key={weekIndex} className="explore-activity-heatmap-week">
              {week.map((day, dayIndex) => (
                <div
                  key={`${weekIndex}-${dayIndex}`}
                  className={`explore-activity-heatmap-day explore-activity-heatmap-day--level-${day.level}`}
                  title={day.date ? formatDateTooltip(day.date, day.count) : ''}
                  data-count={day.count}
                  data-date={day.date}
                />
              ))}
            </div>
          ))}
        </div>
      </div>
      
      {/* Legend */}
      <div className="explore-activity-heatmap-legend">
        <span className="explore-activity-heatmap-legend-text">Less</span>
        <div className="explore-activity-heatmap-legend-levels">
          {[0, 1, 2, 3, 4].map(level => (
            <div
              key={level}
              className={`explore-activity-heatmap-legend-level explore-activity-heatmap-day--level-${level}`}
            />
          ))}
        </div>
        <span className="explore-activity-heatmap-legend-text">More</span>
      </div>
    </div>
  );
}