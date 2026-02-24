import crypto from 'crypto'

const SUBS_KEY = 'repobox:subscribers'
const LEADERBOARD_KEY = 'repobox:leaderboard'

let redisClient = null

function getRedis() {
  if (!redisClient) {
    // Parse KV_REDIS_URL (redis://user:pass@host:port) into Upstash REST format
    const url = process.env.KV_REDIS_URL
    if (!url) throw new Error('KV_REDIS_URL not set')

    // Upstash REST API: https://HOST with token = password
    const match = url.match(/^rediss?:\/\/[^:]+:([^@]+)@(.+?)(?::(\d+))?$/)
    if (!match) throw new Error('Cannot parse KV_REDIS_URL')

    const [, password, host] = match
    const { Redis } = require('@upstash/redis')
    redisClient = new Redis({
      url: `https://${host}`,
      token: password,
    })
  }
  return redisClient
}

// In-memory token tracking (ephemeral is fine)
const usedTokens = new Set()
const secret = crypto.randomBytes(32).toString('hex')

export async function getSubscribers() {
  const data = await getRedis().get(SUBS_KEY)
  return data || []
}

export async function addSubscriber(email) {
  const subs = await getSubscribers()
  if (subs.find(s => s.email === email)) return { ok: true, msg: 'Already subscribed' }
  subs.push({ email, ts: new Date().toISOString() })
  await getRedis().set(SUBS_KEY, subs)
  return { ok: true }
}

export async function getLeaderboard(full = false) {
  const rows = (await getRedis().get(LEADERBOARD_KEY)) || []
  rows.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
  return full ? rows : rows.slice(0, 20)
}

export async function addLeaderboardEntry(name, score) {
  const rows = await getLeaderboard(true)
  rows.push({ name, score, ts: new Date().toISOString() })
  rows.sort((a, b) => b.score - a.score || (a.ts > b.ts ? 1 : -1))
  const trimmed = rows.slice(0, 200)
  await getRedis().set(LEADERBOARD_KEY, trimmed)
  return { ok: true }
}

export { usedTokens, secret }
