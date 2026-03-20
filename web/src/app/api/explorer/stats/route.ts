import { NextResponse } from 'next/server';
import { runQueryOne, runQuery } from '@/lib/database';
import { getCommitCount } from '@/lib/git';

export async function GET() {
  try {
    // Get total repos count
    const repoCountResult = await runQueryOne<{ count: number }>('SELECT COUNT(*) as count FROM repos');
    const totalRepos = repoCountResult?.count || 0;
    
    // Get total unique owners
    const ownerCountResult = await runQueryOne<{ count: number }>('SELECT COUNT(DISTINCT owner_address) as count FROM repos');
    const totalOwners = ownerCountResult?.count || 0;
    
    // Get all repos to calculate total commits
    const repos = await runQuery<{ address: string; name: string }>('SELECT address, name FROM repos');
    
    let totalCommits = 0;
    for (const repo of repos) {
      try {
        totalCommits += getCommitCount(repo.address, repo.name);
      } catch (error) {
        // Skip repos with errors (empty or corrupted repos)
        console.warn(`Error counting commits for ${repo.address}/${repo.name}:`, error);
      }
    }
    
    return NextResponse.json({
      totalRepos,
      totalOwners,
      totalCommits
    });
  } catch (error) {
    console.error('Error fetching stats:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stats' },
      { status: 500 }
    );
  }
}