import { NextResponse } from 'next/server'
import Redis from 'ioredis'

export async function GET() {
  const url = process.env.KV_REDIS_URL || ''

  // Try with TLS
  try {
    const r = new Redis(url, {
      tls: {},
      connectTimeout: 5000,
      maxRetriesPerRequest: 1,
      retryStrategy: () => null,
    })

    const pong = await r.ping()
    await r.set('repobox:test', 'hello')
    const val = await r.get('repobox:test')
    await r.quit()
    return NextResponse.json({ status: 'connected_tls', pong, testVal: val })
  } catch (e) {
    return NextResponse.json({ status: 'tls_error', message: e.message })
  }
}
