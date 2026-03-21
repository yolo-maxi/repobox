import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

const TOKEN = process.env.DASHBOARD_TOKEN || ''
const COOKIE_NAME = 'rb-auth'

export function middleware(request: NextRequest) {
  // Only protect dashboard routes
  if (!request.nextUrl.pathname.startsWith('/dashboard')) {
    return NextResponse.next()
  }

  // No token configured — block everything
  if (!TOKEN) {
    return new NextResponse('🔒 Dashboard not configured.', { status: 503 })
  }

  // Check for token in query param (magic link)
  const url = request.nextUrl
  const queryToken = url.searchParams.get('token')
  
  if (queryToken === TOKEN) {
    // Valid token — set cookie and redirect to clean URL
    url.searchParams.delete('token')
    const response = NextResponse.redirect(url)
    response.cookies.set(COOKIE_NAME, TOKEN, {
      httpOnly: true,
      secure: true,
      sameSite: 'lax',
      maxAge: 60 * 60 * 24 * 30, // 30 days
    })
    return response
  }

  // Check cookie
  const cookie = request.cookies.get(COOKIE_NAME)
  if (cookie?.value === TOKEN) {
    return NextResponse.next()
  }

  // No auth — block
  return new NextResponse('🔒 Access denied. Use your magic link.', { status: 403 })
}

export const config = {
  matcher: '/dashboard/:path*',
}
