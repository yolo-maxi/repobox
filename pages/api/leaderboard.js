import { verifyToken } from '../../lib/token'
import store from '../../lib/store'

const GAME_TICK_MS = 125

export default function handler(req, res) {
  if (req.method === 'GET') {
    const full = req.query.full === '1'
    const rows = [...store.leaderboard].sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
    return res.json({ ok: true, rows: full ? rows : rows.slice(0, 20) })
  }

  if (req.method !== 'POST') return res.status(405).end()

  try {
    const { name, score, token, ts } = req.body
    const cleanName = String(name || '').trim().slice(0, 24)
    const numScore = Number(score)

    if (!cleanName || !Number.isFinite(numScore) || numScore <= 0)
      return res.status(400).json({ error: 'Invalid name or score' })

    if (!token || !ts)
      return res.status(403).json({ error: 'Missing game token. Nice try 🐍' })

    try {
      if (!verifyToken(token, ts))
        return res.status(403).json({ error: 'Invalid token. Play the game! 🎮' })
    } catch {
      return res.status(403).json({ error: 'Bad token format' })
    }

    const tokenKey = token.slice(0, 16)
    if (store.usedTokens.has(tokenKey))
      return res.status(403).json({ error: 'Token already used.' })

    const elapsed = Date.now() - Number(ts)
    const minTime = Math.floor(numScore) * GAME_TICK_MS
    if (elapsed < minTime - 500)
      return res.status(403).json({ error: 'Too fast. 🤔' })
    if (elapsed > 600_000)
      return res.status(403).json({ error: 'Token expired. Start a new game.' })

    store.usedTokens.add(tokenKey)
    if (store.usedTokens.size > 10000) {
      const arr = [...store.usedTokens]; arr.splice(0, 5000)
      store.usedTokens.clear(); arr.forEach(t => store.usedTokens.add(t))
    }

    store.leaderboard.push({ name: cleanName, score: Math.floor(numScore), ts: new Date().toISOString() })
    store.leaderboard.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
    if (store.leaderboard.length > 200) store.leaderboard.length = 200

    return res.json({ ok: true })
  } catch {
    return res.status(400).json({ error: 'Bad request' })
  }
}
