import { NextRequest, NextResponse } from 'next/server';
import { runQuery } from '@/lib/database';

interface SearchRepo {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
}

interface SearchCommit {
  id: number;
  address: string;
  name: string;
  pusher_address?: string;
  commit_hash?: string;
  commit_message?: string;
  pushed_at: string;
}

interface SearchResults {
  repos: SearchRepo[];
  commits: SearchCommit[];
  query: string;
  total_repos: number;
  total_commits: number;
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const query = searchParams.get('q');
    
    if (!query || query.trim().length === 0) {
      return NextResponse.json({
        repos: [],
        commits: [],
        query: '',
        total_repos: 0,
        total_commits: 0
      } as SearchResults);
    }

    // Sanitize the query - escape single quotes and remove dangerous characters
    const sanitizedQuery = query.trim().replace(/'/g, "''").replace(/[;]/g, '').replace(/--/g, '');
    
    if (sanitizedQuery.length === 0) {
      return NextResponse.json({
        repos: [],
        commits: [],
        query: query.trim(),
        total_repos: 0,
        total_commits: 0
      } as SearchResults);
    }

    // Search repos (name and owner_address)
    const repoResults = await runQuery<SearchRepo>(
      `SELECT address, name, owner_address, created_at 
       FROM repos 
       WHERE name LIKE ? OR owner_address LIKE ?
       ORDER BY created_at DESC
       LIMIT 20`,
      [`%${sanitizedQuery}%`, `%${sanitizedQuery}%`]
    );

    // Search commit messages
    const commitResults = await runQuery<SearchCommit>(
      `SELECT id, address, name, pusher_address, commit_hash, commit_message, pushed_at
       FROM push_log 
       WHERE commit_message LIKE ?
       ORDER BY pushed_at DESC
       LIMIT 20`,
      [`%${sanitizedQuery}%`]
    );

    return NextResponse.json({
      repos: repoResults,
      commits: commitResults,
      query: query.trim(),
      total_repos: repoResults.length,
      total_commits: commitResults.length
    } as SearchResults);

  } catch (error) {
    console.error('Search error:', error);
    return NextResponse.json(
      { error: 'Search failed' },
      { status: 500 }
    );
  }
}