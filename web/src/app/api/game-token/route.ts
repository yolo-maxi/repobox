import { NextResponse } from 'next/server';
import { createHmac, randomBytes } from 'crypto';

// Generate a new secret on each restart - rotates automatically
const SECRET = randomBytes(32).toString('hex');

function makeToken(ts: number) {
  return createHmac('sha256', SECRET).update(String(ts)).digest('hex');
}

export async function GET() {
  const ts = Date.now();
  const token = makeToken(ts);
  
  return NextResponse.json({ token, ts });
}