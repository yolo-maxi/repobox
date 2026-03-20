import { NextRequest, NextResponse } from 'next/server';
import { resolveENS } from '@/lib/ens';

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
    
    // Only try to resolve if it looks like an ENS name
    if (!name.endsWith('.eth')) {
      return NextResponse.json(
        { error: 'Only .eth names are supported' },
        { status: 400 }
      );
    }
    
    try {
      const address = await resolveENS(name);
      
      if (address) {
        return NextResponse.json({
          name,
          address
        });
      } else {
        return NextResponse.json(
          { error: 'Name not found or not resolved' },
          { status: 404 }
        );
      }
    } catch (error) {
      console.error('ENS resolution error:', error);
      return NextResponse.json(
        { error: 'Failed to resolve name' },
        { status: 500 }
      );
    }
  } catch (error) {
    console.error('Error in resolve endpoint:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}