import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getCommitDetail } from '@/lib/git';
import { execSync } from 'child_process';

interface RouteContext {
  params: Promise<{ address: string; name: string; hash: string }>;
}

function getSignerForCommit(address: string, name: string, hash: string): string | null {
  try {
    const escapedAddress = address.replace(/'/g, "''");
    const escapedName = name.replace(/'/g, "''");
    const escapedHash = hash.toLowerCase().replace(/'/g, "''");
    const sql = [
      "SELECT COALESCE(NULLIF(pusher_address, ''), '') AS signer",
      "FROM push_log",
      `WHERE address='${escapedAddress}' AND name='${escapedName}' AND LOWER(commit_hash)='${escapedHash}'`,
      'ORDER BY pushed_at DESC LIMIT 1;'
    ].join(' ');

    const out = execSync(
      `sqlite3 /var/lib/repobox/repos/repobox.db \"${sql}\"`,
      { timeout: 2000, encoding: 'utf8' }
    ).trim();

    return out || null;
  } catch {
    return null;
  }
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name, hash } = await context.params;
    
    if (!address || !name || !hash) {
      return NextResponse.json(
        { error: 'Address, name, and hash are required' },
        { status: 400 }
      );
    }
    
    // Validate commit hash format (7-40 hex characters)
    if (!/^[a-f0-9]{7,40}$/i.test(hash)) {
      return NextResponse.json(
        { error: 'Invalid commit hash format' },
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
      const commitDetail = getCommitDetail(address, name, hash);
      
      if (!commitDetail) {
        return NextResponse.json(
          { error: 'Commit not found' },
          { status: 404 }
        );
      }
      
      const signer = getSignerForCommit(address, name, commitDetail.hash);
      return NextResponse.json({ ...commitDetail, signer });
    } catch (gitError) {
      console.error('Git error:', gitError);
      return NextResponse.json(
        { error: 'Failed to fetch commit details' },
        { status: 500 }
      );
    }
  } catch (error) {
    console.error('Error fetching commit detail:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}