import crypto from 'crypto'

// In-memory store — resets on cold start
// TODO: migrate to Vercel KV for persistence
const store = globalThis.__repoboxStore || (globalThis.__repoboxStore = {
  subscribers: [],
  leaderboard: [],
  usedTokens: new Set(),
  secret: crypto.randomBytes(32).toString('hex'),
})

export default store
