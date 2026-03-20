import { NextRequest, NextResponse } from 'next/server';
import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';
import { createHmac, timingSafeEqual, randomBytes } from 'crypto';

const LEADERBOARD_FILE = join(process.cwd(), 'data', 'leaderboard.json');
const SECRET = randomBytes(32).toString('hex'); // rotates on restart
const GAME_TICK_MS = 125;
const usedTokens = new Set<string>();

function loadLeaderboard() {
  try {
    return JSON.parse(readFileSync(LEADERBOARD_FILE, 'utf8'));
  } catch {
    return [];
  }
}

function saveLeaderboard(rows: any[]) {
  writeFileSync(LEADERBOARD_FILE, JSON.stringify(rows, null, 2));
}

function makeToken(ts: number) {
  return createHmac('sha256', SECRET).update(String(ts)).digest('hex');
}

function verifyToken(token: string, ts: number) {
  if (!token || !ts) return false;
  const expected = makeToken(ts);
  try {
    return timingSafeEqual(Buffer.from(token, 'hex'), Buffer.from(expected, 'hex'));
  } catch {
    return false;
  }
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const full = searchParams.get('full') === '1';
  
  const rows = loadLeaderboard()
    .sort((a: any, b: any) => b.score - a.score || (a.ts > b.ts ? 1 : -1));
  
  return NextResponse.json({ ok: true, rows: full ? rows : rows.slice(0, 20) });
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { name, score, token, ts } = body;
    
    const cleanName = String(name || '').trim().slice(0, 24);
    const numScore = Number(score);

    if (!cleanName || !Number.isFinite(numScore) || numScore <= 0) {
      return NextResponse.json({ error: 'Invalid name or score' }, { status: 400 });
    }

    // Validate game token
    if (!token || !ts) {
      return NextResponse.json({ error: 'Missing game token. Nice try 🐍' }, { status: 403 });
    }

    if (!verifyToken(token, ts)) {
      return NextResponse.json({ error: 'Invalid token. Play the game! 🎮' }, { status: 403 });
    }

    // Single-use token
    const tokenKey = token.slice(0, 16);
    if (usedTokens.has(tokenKey)) {
      return NextResponse.json({ error: 'Token already used. One game, one score.' }, { status: 403 });
    }

    // Time validation: did enough real time pass?
    const elapsed = Date.now() - Number(ts);
    const minTime = Math.floor(numScore) * GAME_TICK_MS;
    
    if (elapsed < minTime - 500) { // 500ms grace for network latency
      const needed = Math.ceil(minTime / 1000);
      return NextResponse.json({
        error: `Too fast. Score ${Math.floor(numScore)} needs ${needed}s of gameplay. You took ${(elapsed/1000).toFixed(1)}s. 🤔`
      }, { status: 403 });
    }

    // Max session 10 minutes (score ~4800 theoretical max)
    if (elapsed > 600_000) {
      return NextResponse.json({ error: 'Token expired. Start a new game.' }, { status: 403 });
    }

    // All good — burn the token and save
    usedTokens.add(tokenKey);
    
    // Cleanup old tokens periodically (keep set small)
    if (usedTokens.size > 10000) {
      const arr = [...usedTokens];
      arr.splice(0, 5000);
      usedTokens.clear();
      arr.forEach(t => usedTokens.add(t));
    }

    const rows = loadLeaderboard();
    rows.push({
      name: cleanName,
      score: Math.floor(numScore),
      ts: new Date().toISOString()
    });

    rows.sort((a: any, b: any) => b.score - a.score || (a.ts > b.ts ? 1 : -1));
    saveLeaderboard(rows.slice(0, 200));

    return NextResponse.json({ ok: true });
  } catch {
    return NextResponse.json({ error: 'Bad request' }, { status: 400 });
  }
}