import Redis from 'ioredis'
import crypto from 'crypto'

const SUBS_KEY = 'repobox:subscribers'
const LEADERBOARD_KEY = 'repobox:leaderboard'

let redis = null

function getRedis() {
  if (!redis) {
    const url = process.env.KV_REDIS_URL
    if (!url) throw new Error('KV_REDIS_URL not set')
    redis = new Redis(url, { lazyConnect: true, maxRetriesPerRequest: 1 })
  }
  return redis
}

// In-memory token tracking (ephemeral is fine — prevents replay within a function instance)
const usedTokens = new Set()
const secret = crypto.randomBytes(32).toString('hex')

export async function getSubscribers() {
  const data = await getRedis().get(SUBS_KEY)
  return data ? JSON.parse(data) : []
}

export async function addSubscriber(email) {
  const subs = await getSubscribers()
  if (subs.find(s => s.email === email)) return { ok: true, msg: 'Already subscribed' }
  subs.push({ email, ts: new Date().toISOString() })
  await getRedis().set(SUBS_KEY, JSON.stringify(subs))
  return { ok: true }
}

export async function getLeaderboard(full = false) {
  const data = await getRedis().get(LEADERBOARD_KEY)
  const rows = data ? JSON.parse(data) : []
  rows.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
  return full ? rows : rows.slice(0, 20)
}

export async function addLeaderboardEntry(name, score) {
  const rows = await getLeaderboard(true)
  rows.push({ name, score, ts: new Date().toISOString() })
  rows.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
  const trimmed = rows.slice(0, 200)
  await getRedis().set(LEADERBOARD_KEY, JSON.stringify(trimmed))
  return { ok: true }
}

export { usedTokens, secret }
