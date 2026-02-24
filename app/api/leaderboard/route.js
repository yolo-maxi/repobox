import { NextResponse } from 'next/server'
import { verifyToken } from '../../../lib/token'
import { getLeaderboard, addLeaderboardEntry, usedTokens } from '../../../lib/store'

const GAME_TICK_MS = 125

export async function GET(request) {
  try {
    const { searchParams } = new URL(request.url)
    const full = searchParams.get('full') === '1'
    const rows = await getLeaderboard(full)
    return NextResponse.json({ ok: true, rows })
  } catch (e) {
    return NextResponse.json({ error: 'Server error' }, { status: 500 })
  }
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

    if (!verifyToken(token, ts))
      return NextResponse.json({ error: 'Invalid token. Play the game! 🎮' }, { status: 403 })

    const tokenKey = token.slice(0, 16)
    if (usedTokens.has(tokenKey))
      return NextResponse.json({ error: 'Token already used.' }, { status: 403 })

    const elapsed = Date.now() - Number(ts)
    const minTime = Math.floor(numScore) * GAME_TICK_MS
    if (elapsed < minTime - 500)
      return NextResponse.json({ error: 'Too fast. 🤔' }, { status: 403 })
    if (elapsed > 600_000)
      return NextResponse.json({ error: 'Token expired. Start a new game.' }, { status: 403 })

    usedTokens.add(tokenKey)
    if (usedTokens.size > 10000) {
      const arr = [...usedTokens]; arr.splice(0, 5000)
      usedTokens.clear(); arr.forEach(t => usedTokens.add(t))
    }

    const result = await addLeaderboardEntry(cleanName, Math.floor(numScore))
    return NextResponse.json(result)
  } catch (e) {
    return NextResponse.json({ error: 'Server error' }, { status: 500 })
  }
}
