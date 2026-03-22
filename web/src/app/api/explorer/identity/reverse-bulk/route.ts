import { NextRequest } from 'next/server';

const GATEWAY_URL = process.env.ENS_GATEWAY_URL || 'http://localhost:3491';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { addresses } = body;

    if (!Array.isArray(addresses)) {
      return Response.json({ error: 'addresses must be an array' }, { status: 400 });
    }

    const res = await fetch(`${GATEWAY_URL}/reverse`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ addresses })
    });

    if (!res.ok) {
      return Response.json({}, { status: 200 }); // empty results on failure
    }

    const data = await res.json();
    return Response.json(data);
  } catch (error) {
    return Response.json({}, { status: 200 });
  }
}
