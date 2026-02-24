import { NextResponse } from 'next/server'
import { makeToken } from '../../../lib/token'

export function GET() {
  const ts = Date.now()
  return NextResponse.json({ token: makeToken(ts), ts })
}
