import { NextRequest, NextResponse } from 'next/server';
import { execSync } from 'child_process';
import { runQueryOne, getRepoPath } from '@/lib/database';
import { branchExists, sanitizeBranchName } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string; path: string[] }>;
}

type BlameLine = {
  lineNumber: number;
  content: string;
  commitHash: string;
  summary: string;
  author: string;
  authorTime: number;
  signer: string | null;
};

function getSignerMap(address: string, name: string, hashes: string[]): Map<string, string> {
  const map = new Map<string, string>();
  const uniqueHashes = Array.from(new Set(hashes.map((h) => h.toLowerCase()).filter((h) => /^[a-f0-9]{40}$/.test(h))));
  if (uniqueHashes.length === 0) return map;

  try {
    const escapedAddress = address.replace(/'/g, "''");
    const escapedName = name.replace(/'/g, "''");
    const inClause = uniqueHashes.map((h) => `'${h}'`).join(',');

    const sql = [
      "SELECT LOWER(commit_hash) AS hash, COALESCE(NULLIF(pusher_address, ''), '') AS signer",
      'FROM push_log',
      `WHERE address='${escapedAddress}' AND name='${escapedName}' AND LOWER(commit_hash) IN (${inClause})`,
      'ORDER BY CAST(pushed_at AS INTEGER) DESC;'
    ].join(' ');

    const raw = execSync(
      `sqlite3 -separator '|' /var/lib/repobox/repos/repobox.db \"${sql}\"`,
      { timeout: 3000, encoding: 'utf8' }
    );

    for (const line of raw.split('\n').filter(Boolean)) {
      const [hash, signer] = line.split('|');
      if (hash && signer && !map.has(hash)) map.set(hash, signer);
    }
  } catch {
    // best effort only
  }

  return map;
}

export async function GET(request: NextRequest, context: RouteContext) {
  try {
    const { address, name, path } = await context.params;
    const { searchParams } = new URL(request.url);
    let branch = searchParams.get('branch') || 'HEAD';

    if (!address || !name || !path || path.length === 0) {
      return NextResponse.json({ error: 'Address, name and file path are required' }, { status: 400 });
    }

    if (branch !== 'HEAD') {
      try {
        branch = sanitizeBranchName(branch);
      } catch {
        return NextResponse.json({ error: 'Invalid branch name' }, { status: 400 });
      }
    }

    const repo = await runQueryOne('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name]);
    if (!repo) return NextResponse.json({ error: 'Repository not found' }, { status: 404 });

    if (branch !== 'HEAD' && !branchExists(address, name, branch)) {
      return NextResponse.json({ error: `Branch '${branch}' does not exist` }, { status: 404 });
    }

    const filePath = path.join('/');
    const repoPath = getRepoPath(address, name);
    const ref = branch === 'HEAD' ? 'HEAD' : `refs/heads/${branch}`;

    const output = execSync(
      `git --git-dir="${repoPath}" blame --line-porcelain ${ref} -- "${filePath.replace(/"/g, '\\"')}"`,
      { timeout: 10000, encoding: 'utf8', maxBuffer: 20 * 1024 * 1024 }
    );

    const lines = output.split('\n');
    const blameLines: Omit<BlameLine, 'signer'>[] = [];

    let i = 0;
    while (i < lines.length) {
      const header = lines[i];
      const m = header.match(/^([a-f0-9]{40})\s+\d+\s+(\d+)(?:\s+(\d+))?$/i);
      if (!m) {
        i++;
        continue;
      }

      const commitHash = m[1];
      const startingLine = Number(m[2]);
      const groupCount = Number(m[3] || 1);
      i++;

      let author = 'Unknown';
      let authorTime = 0;
      let summary = '';

      while (i < lines.length && !lines[i].startsWith('\t')) {
        const line = lines[i];
        if (line.startsWith('author ')) author = line.slice(7);
        else if (line.startsWith('author-time ')) authorTime = Number(line.slice(12)) || 0;
        else if (line.startsWith('summary ')) summary = line.slice(8);
        i++;
      }

      for (let j = 0; j < groupCount && i < lines.length; j++) {
        if (!lines[i].startsWith('\t')) break;
        blameLines.push({
          lineNumber: startingLine + j,
          content: lines[i].slice(1),
          commitHash,
          summary,
          author,
          authorTime,
        });
        i++;
      }
    }

    const signerMap = getSignerMap(address, name, blameLines.map((l) => l.commitHash));
    const enriched: BlameLine[] = blameLines.map((l) => ({
      ...l,
      signer: signerMap.get(l.commitHash.toLowerCase()) || null,
    }));

    return NextResponse.json({
      path: filePath,
      branch,
      totalLines: enriched.length,
      lines: enriched,
    });
  } catch (error: any) {
    const msg = String(error?.message || 'Failed to load blame');
    if (msg.includes('no such path') || msg.includes('fatal: no such path')) {
      return NextResponse.json({ error: 'File not found at this ref' }, { status: 404 });
    }
    return NextResponse.json({ error: 'Failed to load blame' }, { status: 500 });
  }
}
