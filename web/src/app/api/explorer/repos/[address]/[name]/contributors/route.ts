import { NextRequest, NextResponse } from 'next/server';
import { runQuery } from '@/lib/database';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
}

interface Contributor {
  address: string;
  pushCount: number;
  lastPush: string;
  isOwner: boolean;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name } = await context.params;
    
    if (!address || !name) {
      return NextResponse.json(
        { error: 'Address and name are required' },
        { status: 400 }
      );
    }
    
    // Get repo owner address
    const repoResult = await runQuery('SELECT owner_address FROM repos WHERE address = ? AND name = ?', [address, name]);
    
    if (!repoResult || repoResult.length === 0) {
      return NextResponse.json(
        { error: 'Repository not found' },
        { status: 404 }
      );
    }
    
    const ownerAddress = repoResult[0].owner_address;
    
    // Query push_log for unique contributors with their stats
    const contributorsData = await runQuery(
      `SELECT 
        pusher_address as address,
        COUNT(*) as pushCount,
        MAX(pushed_at) as lastPush
      FROM push_log 
      WHERE address = ? AND name = ? AND pusher_address IS NOT NULL
      GROUP BY pusher_address
      ORDER BY pushCount DESC`,
      [address, name]
    );
    
    // Transform and mark owners
    const contributors: Contributor[] = contributorsData.map((row: any) => ({
      address: row.address,
      pushCount: row.pushCount,
      lastPush: row.lastPush,
      isOwner: row.address === ownerAddress
    }));
    
    return NextResponse.json({ contributors });
  } catch (error) {
    console.error('Error fetching contributors:', error);
    return NextResponse.json(
      { error: 'Failed to fetch contributors' },
      { status: 500 }
    );
  }
}