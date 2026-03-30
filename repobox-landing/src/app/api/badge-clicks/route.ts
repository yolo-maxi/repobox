import { NextRequest, NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

interface BadgeClick {
  timestamp: string;
  variant: string;
  size: string;
  referrer: string;
  userAgent: string;
  ip: string;
}

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const variant = searchParams.get('variant') || 'unknown';
  const size = searchParams.get('size') || 'unknown';
  const referrer = request.headers.get('referer') || 'direct';
  const userAgent = request.headers.get('user-agent') || 'unknown';
  
  // Get client IP (handle various proxy headers)
  const forwarded = request.headers.get('x-forwarded-for');
  const ip = forwarded ? forwarded.split(',')[0] : 
             request.headers.get('x-real-ip') || 
             'unknown';

  // Log the click
  const clickData: BadgeClick = {
    timestamp: new Date().toISOString(),
    variant,
    size,
    referrer,
    userAgent,
    ip
  };

  try {
    const logPath = path.join(process.cwd(), '.state', 'badge-clicks.jsonl');
    const logDir = path.dirname(logPath);
    
    // Ensure directory exists
    if (!fs.existsSync(logDir)) {
      fs.mkdirSync(logDir, { recursive: true });
    }
    
    // Append click data
    fs.appendFileSync(logPath, JSON.stringify(clickData) + '\n');
    
    // Redirect to repo.box homepage
    return NextResponse.redirect('https://repo.box', 302);
  } catch (error) {
    console.error('Badge click tracking error:', error);
    // Still redirect even if logging fails
    return NextResponse.redirect('https://repo.box', 302);
  }
}