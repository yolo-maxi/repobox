import { NextResponse } from 'next/server'
import fs from 'fs'

const SKILL_FILE = '/home/xiko/repobox/docs/SKILL.md'

export async function GET() {
  const content = fs.readFileSync(SKILL_FILE, 'utf-8')
  return NextResponse.json({ content })
}

export async function PUT(req: Request) {
  const { content } = await req.json()
  fs.writeFileSync(SKILL_FILE, content)
  return NextResponse.json({ ok: true })
}
