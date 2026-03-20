import { NextResponse } from 'next/server'
import fs from 'fs'
import { TESTS_FILE } from '@/lib/paths'

export async function GET() {
  const data = JSON.parse(fs.readFileSync(TESTS_FILE, 'utf-8'))
  return NextResponse.json(data)
}

export async function PUT(req: Request) {
  const data = await req.json()
  fs.writeFileSync(TESTS_FILE, JSON.stringify(data, null, 2))
  return NextResponse.json({ ok: true })
}
