// In-memory store — resets on cold start
// TODO: migrate to Vercel KV for persistence

function makeSecret() {
  const crypto = require('crypto')
  return crypto.randomBytes(32).toString('hex')
}

const store = globalThis.__repoboxStore || (globalThis.__repoboxStore = {
  subscribers: [],
  leaderboard: [],
  usedTokens: new Set(),
  secret: makeSecret(),
})

export default store
