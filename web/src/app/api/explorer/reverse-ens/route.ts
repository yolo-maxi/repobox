import { NextRequest, NextResponse } from 'next/server';
import { reverseResolveENS } from '@/lib/ens';

export async function POST(request: NextRequest) {
  try {
    const { address } = await request.json();
    
    if (!address || !/^0x[a-fA-F0-9]{40}$/i.test(address)) {
      return NextResponse.json(
        { error: 'Valid address is required' },
        { status: 400 }
      );
    }
    
    const name = await reverseResolveENS(address);
    
    return NextResponse.json({
      address,
      name,
      found: !!name
    });
  } catch (error) {
    console.error('Reverse ENS resolution error:', error);
    return NextResponse.json(
      { error: 'Resolution failed' },
      { status: 500 }
    );
  }
}