import { NextRequest, NextResponse } from 'next/server';
import { resolveENS } from '@/lib/ens';
import { resolveAddressDisplay } from '@/lib/addressResolver';

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
    
    let address: string | null = null;
    let type: string = 'unknown';
    
    // 1. Check if it's already an address
    if (/^0x[a-fA-F0-9]{40}$/i.test(name)) {
      address = name;
      type = 'address';
    }
    // 2. Try ENS resolution
    else if (name.endsWith('.eth')) {
      address = await resolveENS(name);
      type = 'ens';
    }
    // 3. Future: subdomain resolution
    else {
      type = 'subdomain';
      // For now, subdomains are not implemented
      return NextResponse.json(
        { error: 'Subdomain resolution not yet implemented' },
        { status: 501 }
      );
    }
    
    if (!address) {
      return NextResponse.json(
        { error: 'Name not found or not resolved' },
        { status: 404 }
      );
    }
    
    // Get display name for the resolved address
    const displayName = await resolveAddressDisplay(address);
    
    return NextResponse.json({
      name,
      address,
      displayName,
      type
    });
  } catch (error) {
    console.error('Resolve endpoint error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}