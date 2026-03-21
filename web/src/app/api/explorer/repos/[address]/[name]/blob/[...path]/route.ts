import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getFileContent, branchExists } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string; path: string[] }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name, path } = await context.params;
    const { searchParams } = new URL(request.url);
    const branch = searchParams.get('branch') || 'HEAD';
    
    if (!address || !name || !path || path.length === 0) {
      return NextResponse.json(
        { error: 'Address, name and file path are required' },
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
    
    // Join path segments
    const filePath = path.join('/');
    
    try {
      const content = getFileContent(address, name, filePath, branch);
      
      if (content === null) {
        return NextResponse.json(
          { error: 'File not found' },
          { status: 404 }
        );
      }
      
      return NextResponse.json({
        path: filePath,
        content
      });
    } catch (gitError) {
      return NextResponse.json(
        { error: 'File not found or not accessible' },
        { status: 404 }
      );
    }
  } catch (error) {
    console.error('Error fetching file content:', error);
    return NextResponse.json(
      { error: 'Failed to fetch file content' },
      { status: 500 }
    );
  }
}