import store from '../../lib/store'

export default function handler(req, res) {
  if (req.method !== 'POST') return res.status(405).end()
  try {
    const { email } = req.body
    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return res.status(400).json({ error: 'Invalid email' })
    }
    const lower = email.toLowerCase()
    if (store.subscribers.find(s => s.email === lower)) {
      return res.json({ ok: true, msg: 'Already subscribed' })
    }
    store.subscribers.push({ email: lower, ts: new Date().toISOString() })
    return res.json({ ok: true })
  } catch {
    return res.status(400).json({ error: 'Bad request' })
  }
}
