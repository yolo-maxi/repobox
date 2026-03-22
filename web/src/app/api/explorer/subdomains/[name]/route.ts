import { NextRequest, NextResponse } from 'next/server';

const ENS_GATEWAY = process.env.ENS_GATEWAY_URL || 'http://localhost:3491';

interface RouteContext {
  params: Promise<{ name: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { name } = await context.params;
    const normalized = name.toLowerCase().endsWith('.repobox.eth')
      ? name.slice(0, -'.repobox.eth'.length)
      : name;

    if (!normalized) {
      return NextResponse.json(
        { error: 'Name is required' },
        { status: 400 }
      );
    }
    
    // Forward resolve via ENS gateway
    const res = await fetch(`${ENS_GATEWAY}/resolve/${encodeURIComponent(normalized)}`, {
      signal: AbortSignal.timeout(5000),
      next: { revalidate: 300 } // Cache for 5 min
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
      source: data.tier || 'auto-alias',
    });
  } catch (error) {
    console.error('Name resolution error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
