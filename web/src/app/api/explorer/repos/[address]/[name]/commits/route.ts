import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getCommitHistory, branchExists } from '@/lib/git';
import { execSync } from 'child_process';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
}

function isHash(value: string): boolean {
  return /^[a-f0-9]{40}$/i.test(value);
}

function getSignerMap(address: string, name: string, hashes: string[]): Map<string, string> {
  const map = new Map<string, string>();
  const validHashes = hashes.filter(isHash);
  if (validHashes.length === 0) return map;

  try {
    const escapedAddress = address.replace(/'/g, "''");
    const escapedName = name.replace(/'/g, "''");
    const inClause = validHashes.map((h) => `'${h.toLowerCase()}'`).join(',');
    const sql = [
      "SELECT LOWER(commit_hash) AS hash, COALESCE(NULLIF(pusher_address, ''), '') AS signer",
      'FROM push_log',
      `WHERE address='${escapedAddress}' AND name='${escapedName}' AND LOWER(commit_hash) IN (${inClause})`,
      'ORDER BY pushed_at DESC;'
    ].join(' ');

    const raw = execSync(
      `sqlite3 -separator '|' /var/lib/repobox/repos/repobox.db \"${sql}\"`,
      { timeout: 2000, encoding: 'utf8' }
    );

    for (const line of raw.split('\n').filter(Boolean)) {
      const [hash, signer] = line.split('|');
      if (hash && signer && !map.has(hash)) {
        map.set(hash, signer);
      }
    }
  } catch {
    // Best effort only; UI should gracefully fallback.
  }

  return map;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name } = await context.params;
    const { searchParams } = new URL(request.url);
    const limit = parseInt(searchParams.get('limit') || '50');
    const branch = searchParams.get('branch') || 'HEAD';
    
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
    
    // Validate branch if specified
    if (branch !== 'HEAD' && !branchExists(address, name, branch)) {
      return NextResponse.json(
        { error: `Branch '${branch}' does not exist` },
        { status: 404 }
      );
    }
    
    try {
      const commits = getCommitHistory(address, name, limit, branch);
      const signerMap = getSignerMap(address, name, commits.map((c) => c.hash));

      const enrichedCommits = commits.map((c) => ({
        ...c,
        signer: signerMap.get(c.hash.toLowerCase()) || null,
      }));

      return NextResponse.json({
        commits: enrichedCommits,
        total: enrichedCommits.length
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