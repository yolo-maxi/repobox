import { NextResponse } from 'next/server'
import store from '../../../lib/store'

export async function POST(request) {
  try {
    const { email } = await request.json()
    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return NextResponse.json({ error: 'Invalid email' }, { status: 400 })
    }
    const lower = email.toLowerCase()
    if (store.subscribers.find(s => s.email === lower)) {
      return NextResponse.json({ ok: true, msg: 'Already subscribed' })
    }
    store.subscribers.push({ email: lower, ts: new Date().toISOString() })
    return NextResponse.json({ ok: true })
  } catch {
    return NextResponse.json({ error: 'Bad request' }, { status: 400 })
  }
}
