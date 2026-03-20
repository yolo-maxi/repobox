import { NextRequest, NextResponse } from 'next/server';
import { runQuery, type PushLog } from '@/lib/database';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const limit = parseInt(searchParams.get('limit') || '20');
    
    // Get latest push activity
    const pushLogs = await runQuery<PushLog>(`
      SELECT * FROM push_log 
      ORDER BY pushed_at DESC 
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