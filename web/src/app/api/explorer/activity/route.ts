import { NextRequest, NextResponse } from 'next/server';
import { runQuery, type PushLog } from '@/lib/database';

interface EnhancedActivity extends PushLog {
  owner_address?: string;
  repo_created_at?: string;
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const limit = parseInt(searchParams.get('limit') || '20');
    
    // Get latest push activity with repository metadata
    const pushLogs = await runQuery<EnhancedActivity>(`
      SELECT 
        p.id,
        p.address,
        p.name,
        p.pusher_address,
        p.commit_hash,
        p.commit_message,
        p.pushed_at,
        r.owner_address,
        r.created_at as repo_created_at
      FROM push_log p
      LEFT JOIN repos r ON (p.address = r.address AND p.name = r.name)
      ORDER BY p.pushed_at DESC 
      LIMIT ?
    `, [limit]);
    
    return NextResponse.json({
      activity: pushLogs,
      total: pushLogs.length
    });
  } catch (error) {
    console.error('Error fetching activity:', error);
    return NextResponse.json(
      { error: 'Failed to fetch activity' },
      { status: 500 }
    );
  }
}