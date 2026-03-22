import { NextRequest, NextResponse } from 'next/server';
import { reverseResolveENS } from '@/lib/ens';

interface RouteContext {
  params: Promise<{ address: string }>;
}

// Check aliases in DB
async function checkAlias(address: string): Promise<string | null> {
  try {
    const res = await fetch(`http://127.0.0.1:3490/api/aliases?address=${address}`, {
      signal: AbortSignal.timeout(3000),
    });
    if (res.ok) {
      const data = await res.json();
      if (data.aliases?.length > 0) return data.aliases[0].name;
    }
  } catch {}
  return null;
}

export async function GET(request: NextRequest, context: RouteContext) {
  const { address } = await context.params;
  
  if (!address || !/^0x[a-fA-F0-9]{40}$/i.test(address)) {
    return NextResponse.json({ error: 'Invalid address' }, { status: 400 });
  }

  // Try alias first (fast, local)
  const alias = await checkAlias(address);
  if (alias) {
    return NextResponse.json({ address, displayName: alias, type: 'alias' });
  }

  // Try ENS reverse
  const ensName = await reverseResolveENS(address);
  if (ensName) {
    return NextResponse.json({ address, displayName: ensName, type: 'ens' });
  }

  return NextResponse.json({ address, displayName: null, type: 'address' });
}
