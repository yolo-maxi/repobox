import { NextResponse } from 'next/server'
import fs from 'fs'
import path from 'path'
import { DATA_DIR } from '@/lib/paths'

const STORIES_FILE = path.join(DATA_DIR, 'stories.json')

export async function GET() {
  try {
    const data = JSON.parse(fs.readFileSync(STORIES_FILE, 'utf-8'))
    return NextResponse.json(data)
  } catch {
    return NextResponse.json({})
  }
}

export async function PUT(req: Request) {
  const data = await req.json()
  fs.writeFileSync(STORIES_FILE, JSON.stringify(data, null, 2))
  return NextResponse.json({ ok: true })
}