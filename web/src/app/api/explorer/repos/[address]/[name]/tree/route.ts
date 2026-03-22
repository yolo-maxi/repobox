import { NextRequest, NextResponse } from 'next/server';
import { getFileTree } from '@/lib/git';
import { runQueryOne } from '@/lib/database';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
}

export async function GET(request: NextRequest, context: RouteContext) {
  const { address, name } = await context.params;
  const path = request.nextUrl.searchParams.get('path') || '';
  const branch = request.nextUrl.searchParams.get('branch') || 'HEAD';

  const repo = runQueryOne('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name]);
  if (!repo) {
    return NextResponse.json({ error: 'Repository not found' }, { status: 404 });
  }

  try {
    const files = getFileTree(address, name, path, branch);
    return NextResponse.json({ path, files });
  } catch (e) {
    return NextResponse.json({ error: 'Failed to read tree' }, { status: 500 });
  }
}
