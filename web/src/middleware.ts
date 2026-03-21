import { NextRequest, NextResponse } from 'next/server';

export function middleware(request: NextRequest) {
  const url = request.nextUrl.clone();
  
  // Handle old tab-based URLs for backward compatibility
  if (url.pathname.match(/^\/explore\/[^\/]+\/[^\/]+$/) && url.searchParams.has('tab')) {
    const pathSegments = url.pathname.split('/');
    const address = pathSegments[2];
    const name = pathSegments[3];
    const tab = url.searchParams.get('tab');
    const path = url.searchParams.get('path') || '';
    const branch = url.searchParams.get('branch') || 'HEAD';
    
    // Redirect to new URL structure
    switch (tab) {
      case 'files':
        if (path) {
          // Determine if it's a file or directory
          // This is a simple heuristic - in reality you'd check the file system
          const isFile = path.includes('.') && !path.endsWith('/');
          if (isFile) {
            url.pathname = `/explore/${address}/${name}/blob/${branch}/${path}`;
          } else {
            url.pathname = `/explore/${address}/${name}/tree/${branch}/${path}`;
          }
        } else {
          url.pathname = `/explore/${address}/${name}/tree/${branch}`;
        }
        break;
      case 'commits':
        url.pathname = `/explore/${address}/${name}/commits/${branch}`;
        break;
      default:
        // For other tabs or no tab, redirect to repo home
        url.pathname = `/explore/${address}/${name}`;
        break;
    }
    
    // Clear search params
    url.search = '';
    return NextResponse.redirect(url);
  }
  
  return NextResponse.next();
}

export const config = {
  matcher: [
    '/explore/:address/:name'
  ]
};