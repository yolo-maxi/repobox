import { NextRequest, NextResponse } from 'next/server';
import { runQuery, type Repo } from '@/lib/database';
import { getCommitCount, getContributorCount, getLastCommitDate, getReadmeFirstLine } from '@/lib/git';

interface RepoWithMetadata extends Repo {
  commit_count: number;
  contributor_count: number;
  last_commit_date: string | null;
  description: string | null;
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sort = searchParams.get('sort') || 'latest';
    const limit = parseInt(searchParams.get('limit') || '50');
    const owner = searchParams.get('owner');
    
    // Get repos, optionally filtered by owner
    let repos: Repo[];
    if (owner) {
      repos = await runQuery<Repo>(
        'SELECT * FROM repos WHERE LOWER(owner_address) = LOWER(?)',
        [owner]
      );
    } else {
      repos = await runQuery<Repo>('SELECT * FROM repos');
    }
    
    // Enrich with metadata
    const reposWithMetadata: RepoWithMetadata[] = [];
    
    for (const repo of repos) {
      try {
        const commitCount = getCommitCount(repo.address, repo.name);
        const contributorCount = getContributorCount(repo.address, repo.name);
        const lastCommitDate = getLastCommitDate(repo.address, repo.name);
        const description = getReadmeFirstLine(repo.address, repo.name);
        
        reposWithMetadata.push({
          ...repo,
          commit_count: commitCount,
          contributor_count: contributorCount,
          last_commit_date: lastCommitDate,
          description
        });
      } catch (error) {
        // Skip repos with errors but log them
        console.warn(`Error processing repo ${repo.address}/${repo.name}:`, error);
        reposWithMetadata.push({
          ...repo,
          commit_count: 0,
          contributor_count: 0,
          last_commit_date: null,
          description: null
        });
      }
    }
    
    // Sort repos
    let sortedRepos = reposWithMetadata;
    switch (sort) {
      case 'latest':
        sortedRepos = reposWithMetadata.sort((a, b) => {
          const dateA = a.last_commit_date ? new Date(a.last_commit_date).getTime() : 0;
          const dateB = b.last_commit_date ? new Date(b.last_commit_date).getTime() : 0;
          return dateB - dateA;
        });
        break;
      case 'commits':
        sortedRepos = reposWithMetadata.sort((a, b) => b.commit_count - a.commit_count);
        break;
      case 'stars':
        // No star data yet, fall back to latest
        sortedRepos = reposWithMetadata.sort((a, b) => {
          const dateA = a.last_commit_date ? new Date(a.last_commit_date).getTime() : 0;
          const dateB = b.last_commit_date ? new Date(b.last_commit_date).getTime() : 0;
          return dateB - dateA;
        });
        break;
    }
    
    // Apply limit
    const limitedRepos = sortedRepos.slice(0, limit);
    
    return NextResponse.json({
      repos: limitedRepos,
      total: reposWithMetadata.length,
      sort,
      limit
    });
  } catch (error) {
    console.error('Error fetching repos:', error);
    return NextResponse.json(
      { error: 'Failed to fetch repositories' },
      { status: 500 }
    );
  }
}