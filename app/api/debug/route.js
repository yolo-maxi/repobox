import { NextResponse } from 'next/server'

export function GET() {
  const url = process.env.KV_REDIS_URL || ''
  // Show scheme + host portion only (mask password)
  const match = url.match(/^(rediss?):\/\/([^:]+):([^@]+)@(.+)$/)
  if (match) {
    return NextResponse.json({
      scheme: match[1],
      user: match[2],
      password_len: match[3].length,
      host_port: match[4],
      full_len: url.length,
    })
  }
  return NextResponse.json({ raw_prefix: url.slice(0, 15), len: url.length })
}
