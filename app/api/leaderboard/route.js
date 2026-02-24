import { NextResponse } from 'next/server'
import { verifyToken } from '../../../lib/token'
import store from '../../../lib/store'

const GAME_TICK_MS = 125

export function GET(request) {
  const { searchParams } = new URL(request.url)
  const full = searchParams.get('full') === '1'
  const rows = [...store.leaderboard].sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
  return NextResponse.json({ ok: true, rows: full ? rows : rows.slice(0, 20) })
}

export async function POST(request) {
  try {
    const { name, score, token, ts } = await request.json()
    const cleanName = String(name || '').trim().slice(0, 24)
    const numScore = Number(score)

    if (!cleanName || !Number.isFinite(numScore) || numScore <= 0)
      return NextResponse.json({ error: 'Invalid name or score' }, { status: 400 })

    if (!token || !ts)
      return NextResponse.json({ error: 'Missing game token. Nice try 🐍' }, { status: 403 })

    try {
      if (!verifyToken(token, ts))
        return NextResponse.json({ error: 'Invalid token. Play the game! 🎮' }, { status: 403 })
    } catch {
      return NextResponse.json({ error: 'Bad token format' }, { status: 403 })
    }

    const tokenKey = token.slice(0, 16)
    if (store.usedTokens.has(tokenKey))
      return NextResponse.json({ error: 'Token already used.' }, { status: 403 })

    const elapsed = Date.now() - Number(ts)
    const minTime = Math.floor(numScore) * GAME_TICK_MS
    if (elapsed < minTime - 500)
      return NextResponse.json({ error: 'Too fast. 🤔' }, { status: 403 })
    if (elapsed > 600_000)
      return NextResponse.json({ error: 'Token expired. Start a new game.' }, { status: 403 })

    store.usedTokens.add(tokenKey)
    if (store.usedTokens.size > 10000) {
      const arr = [...store.usedTokens]; arr.splice(0, 5000)
      store.usedTokens.clear(); arr.forEach(t => store.usedTokens.add(t))
    }

    store.leaderboard.push({ name: cleanName, score: Math.floor(numScore), ts: new Date().toISOString() })
    store.leaderboard.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
    if (store.leaderboard.length > 200) store.leaderboard.length = 200

    return NextResponse.json({ ok: true })
  } catch {
    return NextResponse.json({ error: 'Bad request' }, { status: 400 })
  }
}
