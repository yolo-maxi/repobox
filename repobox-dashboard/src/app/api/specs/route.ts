import { NextResponse } from 'next/server'
import fs from 'fs'
import path from 'path'
import { SPEC_DIR } from '@/lib/paths'

export async function GET() {
  const files = fs.readdirSync(SPEC_DIR).filter(f => f.endsWith('.md')).sort()
  const specs = files.map(f => ({
    name: f.replace('.md', ''),
    filename: f,
    content: fs.readFileSync(path.join(SPEC_DIR, f), 'utf-8')
  }))
  return NextResponse.json(specs)
}
