import { NextRequest } from 'next/server';
import { runQuery } from '@/lib/database';

interface ActivityDay {
  day: string;
  count: number;
}

interface ProfileData {
  firstCommitDate: number | null;
  activeRepos: number;
  activity: ActivityDay[];
}

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ address: string }> }
) {
  try {
    const { address } = await params;
    
    if (!address || !/^0x[a-fA-F0-9]{40}$/i.test(address)) {
      return Response.json({ error: 'Invalid address' }, { status: 400 });
    }

    // Get the timestamps for 90 days ago and 30 days ago
    const now = Math.floor(Date.now() / 1000);
    const ninetyDaysAgo = now - (90 * 24 * 60 * 60);
    const thirtyDaysAgo = now - (30 * 24 * 60 * 60);

    // Activity heatmap data (last 90 days, grouped by date)
    const activity = runQuery<ActivityDay>(
      `SELECT date(pushed_at, 'unixepoch') as day, COUNT(*) as count 
       FROM push_log 
       WHERE (pusher_address = ? OR address = ?) 
       AND pushed_at > ?
       GROUP BY day 
       ORDER BY day`,
      [address, address, ninetyDaysAgo]
    );

    // First commit date
    const firstCommitResult = runQuery<{ first_commit: number }>(
      `SELECT MIN(pushed_at) as first_commit 
       FROM push_log 
       WHERE pusher_address = ?`,
      [address]
    );
    
    const firstCommitDate = firstCommitResult[0]?.first_commit || null;

    // Active repos (repos with commits in last 30 days)
    const activeReposResult = runQuery<{ active: number }>(
      `SELECT COUNT(DISTINCT name) as active 
       FROM push_log 
       WHERE pusher_address = ? 
       AND pushed_at > ?`,
      [address, thirtyDaysAgo]
    );
    
    const activeRepos = activeReposResult[0]?.active || 0;

    const profileData: ProfileData = {
      firstCommitDate,
      activeRepos,
      activity
    };

    return Response.json(profileData);

  } catch (error) {
    console.error('Profile API error:', error);
    return Response.json(
      { error: 'Failed to fetch profile data' },
      { status: 500 }
    );
  }
}