import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getCommitCount, getDefaultBranch, getFileTree, getReadmeContent } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
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
    
    // Get repo from database
    const repo = await runQueryOne('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name]);
    
    if (!repo) {
      return NextResponse.json(
        { error: 'Repository not found' },
        { status: 404 }
      );
    }
    
    try {
      // Get git metadata
      const commitCount = getCommitCount(address, name);
      const defaultBranch = getDefaultBranch(address, name);
      const fileTree = getFileTree(address, name);
      const readmeContent = getReadmeContent(address, name);
      
      return NextResponse.json({
        ...repo,
        commit_count: commitCount,
        default_branch: defaultBranch,
        file_tree: fileTree,
        readme_content: readmeContent
      });
    } catch (gitError) {
      // Repo exists in DB but has git issues (empty repo, etc.)
      return NextResponse.json({
        ...repo,
        commit_count: 0,
        default_branch: 'main',
        file_tree: [],
        readme_content: null
      });
    }
  } catch (error) {
    console.error('Error fetching repo details:', error);
    return NextResponse.json(
      { error: 'Failed to fetch repository details' },
      { status: 500 }
    );
  }
}