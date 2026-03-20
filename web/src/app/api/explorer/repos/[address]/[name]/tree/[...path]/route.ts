import { NextRequest, NextResponse } from 'next/server';
import { runQueryOne } from '@/lib/database';
import { getFileTree } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string; path: string[] }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name, path } = await context.params;
    
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
    
    // Join path segments
    const treePath = path ? path.join('/') : '';
    
    try {
      const fileTree = getFileTree(address, name, treePath);
      
      return NextResponse.json({
        path: treePath,
        files: fileTree
      });
    } catch (gitError) {
      return NextResponse.json(
        { error: 'Path not found or repository is empty' },
        { status: 404 }
      );
    }
  } catch (error) {
    console.error('Error fetching tree:', error);
    return NextResponse.json(
      { error: 'Failed to fetch tree' },
      { status: 500 }
    );
  }
}