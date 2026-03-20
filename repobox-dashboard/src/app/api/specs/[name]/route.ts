import { NextResponse } from 'next/server'
import fs from 'fs'
import path from 'path'
import { SPEC_DIR } from '@/lib/paths'

export async function GET(_req: Request, { params }: { params: Promise<{ name: string }> }) {
  const { name } = await params
  const file = path.join(SPEC_DIR, `${name}.md`)
  if (!fs.existsSync(file)) return NextResponse.json({ error: 'not found' }, { status: 404 })
  return NextResponse.json({ name, content: fs.readFileSync(file, 'utf-8') })
}

export async function PUT(req: Request, { params }: { params: Promise<{ name: string }> }) {
  const { name } = await params
  const { content } = await req.json()
  const file = path.join(SPEC_DIR, `${name}.md`)
  fs.writeFileSync(file, content)
  return NextResponse.json({ ok: true })
}
