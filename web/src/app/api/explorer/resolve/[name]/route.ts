import { NextRequest, NextResponse } from 'next/server';
import { resolveAddressDisplay } from '@/lib/addressResolver';

interface RouteContext {
  params: Promise<{ name: string }>;
}

// ENS Universal Resolver on Ethereum mainnet
const UNIVERSAL_RESOLVER = '0xce01f8eee7E479C928F8919abD53E553a36CeF67';
// Multiple fallback RPCs — free public RPCs are unreliable
const ETH_RPCS = [
  'https://eth.drpc.org',
  'https://ethereum-rpc.publicnode.com',
  'https://rpc.mevblocker.io',
];

async function resolveENSViaRPC(name: string): Promise<string | null> {
  // Use ethers for ENS resolution with fallback RPCs
  const { ethers } = await import('ethers');
  
  for (const rpc of ETH_RPCS) {
    try {
      const provider = new ethers.JsonRpcProvider(rpc);
      const address = await Promise.race([
        provider.resolveName(name),
        new Promise<null>((_, reject) => setTimeout(() => reject(new Error('timeout')), 8000))
      ]);
      if (address) return address;
    } catch (e) {
      console.warn(`ENS resolution failed on ${rpc}:`, (e as Error).message);
      continue;
    }
  }
  return null;
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
      address = await resolveENSViaRPC(name);
      type = 'ens';
    }
    // 3. Future: subdomain resolution
    else {
      type = 'subdomain';
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