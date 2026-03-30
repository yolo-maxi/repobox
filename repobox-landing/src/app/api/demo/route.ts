// Demo asset API endpoint for project previews
// Serves screenshots, GIFs, and other demo media

import { NextRequest, NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

export async function GET(request: NextRequest) {
  const { searchParams, pathname } = new URL(request.url);
  
  // Extract filename from pathname: /api/demo/filename.ext
  const filename = pathname.split('/api/demo/')[1];
  
  if (!filename) {
    return NextResponse.json({ error: 'Filename required' }, { status: 400 });
  }

  // Security: only allow specific file extensions and prevent path traversal
  const allowedExtensions = ['.png', '.jpg', '.jpeg', '.gif', '.webp'];
  const ext = path.extname(filename).toLowerCase();
  
  if (!allowedExtensions.includes(ext) || filename.includes('..') || filename.includes('/')) {
    return NextResponse.json({ error: 'Invalid filename' }, { status: 400 });
  }

  // Demo assets are stored in public/demo/
  const demoPath = path.join(process.cwd(), 'public', 'demo', filename);
  
  try {
    // Check if file exists
    if (!fs.existsSync(demoPath)) {
      // For missing demo assets, return placeholder image
      return generatePlaceholder(filename);
    }

    // Read and serve the file
    const fileBuffer = fs.readFileSync(demoPath);
    const mimeType = getMimeType(ext);

    return new NextResponse(fileBuffer, {
      headers: {
        'Content-Type': mimeType,
        'Cache-Control': 'public, max-age=3600',
        'Content-Length': fileBuffer.length.toString(),
      },
    });
  } catch (error) {
    console.error('Demo API error:', error);
    return NextResponse.json({ error: 'Failed to serve demo asset' }, { status: 500 });
  }
}

function getMimeType(extension: string): string {
  switch (extension) {
    case '.png': return 'image/png';
    case '.jpg':
    case '.jpeg': return 'image/jpeg';
    case '.gif': return 'image/gif';
    case '.webp': return 'image/webp';
    default: return 'application/octet-stream';
  }
}

function generatePlaceholder(filename: string): NextResponse {
  // Generate SVG placeholder for missing demo assets
  const svg = `
    <svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
      <rect width="400" height="300" fill="#1e1e1e"/>
      <rect x="1" y="1" width="398" height="298" fill="none" stroke="#4fc3f7" stroke-width="2" stroke-dasharray="5,5"/>
      <text x="200" y="130" fill="#4fc3f7" font-family="monospace" font-size="14" text-anchor="middle">DEMO PREVIEW</text>
      <text x="200" y="150" fill="#5a7a94" font-family="monospace" font-size="12" text-anchor="middle">${filename}</text>
      <text x="200" y="180" fill="#5a7a94" font-family="monospace" font-size="10" text-anchor="middle">Live demo available</text>
    </svg>
  `;

  return new NextResponse(svg, {
    headers: {
      'Content-Type': 'image/svg+xml',
      'Cache-Control': 'public, max-age=300', // Shorter cache for placeholders
    },
  });
}