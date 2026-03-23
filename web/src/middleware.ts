import { NextRequest, NextResponse } from 'next/server';

const RESERVED_TOP_LEVEL = new Set([
  '',
  'api',
  '_next',
  'explore',
  'dashboard',
  'docs',
  'blog',
  'brand',
  'projects',
  'playground',
]);

const STATIC_FILES = new Set([
  'favicon.ico',
  'robots.txt',
  'sitemap.xml',
  'manifest.json',
]);

function normalizeLegacyTab(pathname: string, searchParams: URLSearchParams): string | null {
  // Supports old style: /explore/:owner/:repo?tab=files|commits&path=&branch=
  // Also supports same query shape on canonical path /:owner/:repo?tab=...
  const pathSegments = pathname.split('/').filter(Boolean);

  let owner: string | undefined;
  let repo: string | undefined;

  if (pathSegments[0] === 'explore') {
    owner = pathSegments[1];
    repo = pathSegments[2];
  } else {
    owner = pathSegments[0];
    repo = pathSegments[1];
  }

  if (!owner || !repo || !searchParams.has('tab')) return null;

  const tab = searchParams.get('tab');
  const path = searchParams.get('path') || '';
  const branch = searchParams.get('branch') || 'HEAD';

  switch (tab) {
    case 'files': {
      if (!path) return `/${owner}/${repo}/tree/${branch}`;
      const isFile = path.includes('.') && !path.endsWith('/');
      return isFile
        ? `/${owner}/${repo}/blob/${branch}/${path}`
        : `/${owner}/${repo}/tree/${branch}/${path}`;
    }
    case 'commits':
      return `/${owner}/${repo}/commits/${branch}`;
    default:
      return `/${owner}/${repo}`;
  }
}

export function middleware(request: NextRequest) {
  const url = request.nextUrl.clone();
  const pathname = url.pathname;

  // 1) Legacy tab query normalization
  const tabCanonical = normalizeLegacyTab(pathname, url.searchParams);
  if (tabCanonical) {
    url.pathname = tabCanonical;
    url.search = '';
    return NextResponse.redirect(url, 308);
  }

  const segments = pathname.split('/').filter(Boolean);
  const first = segments[0] || '';

  // 2) Canonicalize /explore/* -> /*
  if (first === 'explore' && segments.length >= 2) {
    url.pathname = `/${segments.slice(1).join('/')}`;
    return NextResponse.redirect(url, 308);
  }

  // Keep /explore landing page as-is
  if (pathname === '/explore') {
    return NextResponse.next();
  }

  // 3) Skip reserved and static paths
  if (RESERVED_TOP_LEVEL.has(first) || STATIC_FILES.has(first)) {
    return NextResponse.next();
  }

  // Ignore obvious static file path (except .eth names)
  if (first.includes('.') && !first.endsWith('.eth')) {
    return NextResponse.next();
  }

  // 4) Rewrite pretty canonical paths -> actual /explore routes
  // /:identity
  // /:identity/:repo
  // /:identity/:repo/(tree|blob|commits|commit)/...
  if (segments.length >= 1) {
    url.pathname = `/explore${pathname}`;
    return NextResponse.rewrite(url);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ['/((?!_next/static|_next/image).*)'],
};
