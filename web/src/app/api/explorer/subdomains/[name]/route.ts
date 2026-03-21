import { NextRequest, NextResponse } from 'next/server';

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
        { error: 'Subdomain name is required' },
        { status: 400 }
      );
    }
    
    // For now, return 404 - will be implemented with subdomain system
    return NextResponse.json(
      { error: 'Subdomain registry not implemented' },
      { status: 404 }
    );
  } catch (error) {
    console.error('Subdomain resolution error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}