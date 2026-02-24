import { NextResponse } from 'next/server'
import { addSubscriber } from '../../../lib/store'

export async function POST(request) {
  try {
    const { email } = await request.json()
    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return NextResponse.json({ error: 'Invalid email' }, { status: 400 })
    }
    const result = await addSubscriber(email.toLowerCase())
    return NextResponse.json(result)
  } catch (e) {
    return NextResponse.json({ error: 'Server error' }, { status: 500 })
  }
}
