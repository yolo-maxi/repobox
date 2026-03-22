import { NextRequest, NextResponse } from 'next/server';

const GIT_SERVER = process.env.GIT_SERVER_URL || 'http://127.0.0.1:3490';

interface RouteContext {
  params: Promise<{ name: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { name } = await context.params;
    
    if (!name) {
      return NextResponse.json(
        { error: 'Name is required' },
        { status: 400 }
      );
    }
    
    // Proxy to the git server's /{name}/resolve endpoint
    // This handles both aliases (deep-blue-kraken) and ENS names
    const res = await fetch(`${GIT_SERVER}/${encodeURIComponent(name)}/resolve`, {
      signal: AbortSignal.timeout(8000),
    });
    
    if (!res.ok) {
      return NextResponse.json(
        { error: 'Name not found' },
        { status: 404 }
      );
    }
    
    const data = await res.json();
    return NextResponse.json({
      address: data.address,
      source: data.source || 'alias',
    });
  } catch (error) {
    console.error('Name resolution error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}