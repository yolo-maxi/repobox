import { NextResponse } from 'next/server'

export function GET() {
  // Find ALL redis/kv related env vars
  const keys = Object.keys(process.env).filter(k =>
    k.includes('KV') || k.includes('REDIS') || k.includes('UPSTASH') || k.includes('STORAGE')
  )
  const result = {}
  for (const k of keys) {
    const v = process.env[k] || ''
    result[k] = v.slice(0, 40) + (v.length > 40 ? '...' : '')
  }
  result._total_env_count = Object.keys(process.env).length
  return NextResponse.json(result)
}
