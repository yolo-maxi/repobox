import { NextRequest } from 'next/server';

const GATEWAY_URL = process.env.ENS_GATEWAY_URL || 'http://localhost:3491';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ address: string }> }
) {
  const { address } = await params;

  try {
    const res = await fetch(`${GATEWAY_URL}/reverse/${address}`, {
      next: { revalidate: 300 } // cache 5 min
    });

    if (!res.ok) {
      return Response.json({ error: 'Not found' }, { status: 404 });
    }

    const data = await res.json();
    return Response.json(data);
  } catch (error) {
    return Response.json({ error: 'Gateway unavailable' }, { status: 502 });
  }
}
