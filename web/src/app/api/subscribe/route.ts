import { NextRequest, NextResponse } from 'next/server';
import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

const SUBS_FILE = join(process.cwd(), 'data', 'subscribers.json');

function loadSubs() {
  try {
    return JSON.parse(readFileSync(SUBS_FILE, 'utf8'));
  } catch {
    return [];
  }
}

function saveSubs(subs: any[]) {
  writeFileSync(SUBS_FILE, JSON.stringify(subs, null, 2));
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { email } = body;

    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return NextResponse.json({ error: 'Invalid email' }, { status: 400 });
    }

    const subs = loadSubs();
    if (subs.find((s: any) => s.email === email.toLowerCase())) {
      return NextResponse.json({ ok: true, msg: 'Already subscribed' });
    }

    subs.push({ email: email.toLowerCase(), ts: new Date().toISOString() });
    saveSubs(subs);

    return NextResponse.json({ ok: true });
  } catch {
    return NextResponse.json({ error: 'Bad request' }, { status: 400 });
  }
}