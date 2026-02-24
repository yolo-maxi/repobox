import crypto from 'crypto'
import store from './store'

export function makeToken(ts) {
  return crypto.createHmac('sha256', store.secret).update(String(ts)).digest('hex')
}

export function verifyToken(token, ts) {
  if (!token || !ts) return false
  const expected = makeToken(ts)
  return crypto.timingSafeEqual(Buffer.from(token, 'hex'), Buffer.from(expected, 'hex'))
}
