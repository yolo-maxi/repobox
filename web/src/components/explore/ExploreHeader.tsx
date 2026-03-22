'use client';

import Link from 'next/link';

export default function ExploreHeader() {
  return (
    <header className="rd-header">
      <div className="rd-header-inner">
        <Link href="/" className="rd-logo">
          repo<span className="rd-logo-dot">.</span>box
        </Link>
        <nav className="rd-nav">
          <Link href="/" className="rd-nav-link">Home</Link>
          <Link href="/explore" className="rd-nav-link rd-nav-link--active">Explore</Link>
          <Link href="/docs" className="rd-nav-link">Docs</Link>
          <Link href="/playground" className="rd-nav-link">Playground</Link>
        </nav>
      </div>
    </header>
  );
}
