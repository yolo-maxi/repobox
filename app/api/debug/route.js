import { NextResponse } from 'next/server'
import Redis from 'ioredis'

export async function GET() {
  const url = process.env.KV_REDIS_URL || ''

  try {
    const r = new Redis(url, {
      connectTimeout: 5000,
      maxRetriesPerRequest: 1,
      retryStrategy: () => null,  // don't retry
    })

    const pong = await r.ping()
    await r.quit()
    return NextResponse.json({ status: 'connected', pong })
  } catch (e) {
    return NextResponse.json({ status: 'error', message: e.message, code: e.code })
  }
}
