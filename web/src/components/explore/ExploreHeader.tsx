'use client';

import Link from 'next/link';

export default function ExploreHeader() {
  return (
    <header className="explore-main-header">
      <div className="explore-main-header-content">
        <div className="explore-nav">
          <Link href="/" className="explore-logo">
            repo<span className="explore-logo-dot">.</span>box
          </Link>
          <nav className="explore-nav-links">
            <Link href="/" className="explore-nav-link">Home</Link>
            <Link href="/explore" className="explore-nav-link">Explore</Link>
            <Link href="/docs" className="explore-nav-link">Docs</Link>
            <Link href="/playground" className="explore-nav-link">Playground</Link>
          </nav>
        </div>
      </div>
    </header>
  );
}