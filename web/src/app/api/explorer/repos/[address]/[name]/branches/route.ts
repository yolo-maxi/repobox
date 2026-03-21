import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getBranches, getDefaultBranch } from '@/lib/git';

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
    
    // Verify repo exists
    const repo = await runQueryOne('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name]);
    
    if (!repo) {
      return NextResponse.json(
        { error: 'Repository not found' },
        { status: 404 }
      );
    }
    
    try {
      const branches = getBranches(address, name);
      const defaultBranch = getDefaultBranch(address, name);
      
      // Mark default branch
      const branchesWithDefault = branches.map(branch => ({
        ...branch,
        is_default: branch.name === defaultBranch
      }));
      
      return NextResponse.json({
        default_branch: defaultBranch,
        branches: branchesWithDefault
      });
    } catch (gitError) {
      // Empty repo or no commits
      return NextResponse.json({
        default_branch: 'main',
        branches: []
      });
    }
  } catch (error) {
    console.error('Error fetching branches:', error);
    return NextResponse.json(
      { error: 'Failed to fetch branches' },
      { status: 500 }
    );
  }
}