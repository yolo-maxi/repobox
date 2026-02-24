import { NextResponse } from 'next/server'

export function GET() {
  const keys = Object.keys(process.env).filter(k => k.includes('KV') || k.includes('REDIS') || k.includes('UPSTASH'))
  const info = {}
  for (const k of keys) {
    const v = process.env[k] || ''
    info[k] = v.startsWith('redis') ? v.slice(0, 30) + '...' : v.slice(0, 20) + '...'
  }
  return NextResponse.json(info)
}
