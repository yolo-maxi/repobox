import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getCommitHistory } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name } = await context.params;
    const { searchParams } = new URL(request.url);
    const limit = parseInt(searchParams.get('limit') || '50');
    
    if (!address || !name) {
      return NextResponse.json(
        { error: 'Address and name are required' },
        { status: 400 }
      );
    }
    
    // Verify repo exists
    const repo = await runQueryOne('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name]);
    
    if (!repo) {
      return NextResponse.json(
        { error: 'Repository not found' },
        { status: 404 }
      );
    }
    
    try {
      const commits = getCommitHistory(address, name, limit);
      
      return NextResponse.json({
        commits,
        total: commits.length
      });
    } catch (gitError) {
      // Empty repo or git error
      return NextResponse.json({
        commits: [],
        total: 0
      });
    }
  } catch (error) {
    console.error('Error fetching commits:', error);
    return NextResponse.json(
      { error: 'Failed to fetch commits' },
      { status: 500 }
    );
  }
}