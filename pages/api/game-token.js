import { makeToken } from '../../lib/token'

export default function handler(req, res) {
  if (req.method !== 'GET') return res.status(405).end()
  const ts = Date.now()
  return res.json({ token: makeToken(ts), ts })
}
