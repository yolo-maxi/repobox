'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

export function SiteNav() {
  const pathname = usePathname();
  
  const links = [
    { href: '/', label: 'home' },
    { href: '/explore', label: 'explorer' },
    { href: '/docs', label: 'docs' },
    { href: '/playground', label: 'playground' },
  ];

  const isActive = (href: string) => {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  };

  return (
    <nav className="site-nav">
      <Link href="/" className="site-nav-logo">
        repo<span style={{ color: 'var(--bp-accent)' }}>.</span>box
      </Link>
      <div className="site-nav-links">
        {links.map(l => (
          <Link
            key={l.href}
            href={l.href}
            className={`site-nav-link ${isActive(l.href) ? 'active' : ''}`}
          >
            {l.label}
          </Link>
        ))}
      </div>
    </nav>
  );
}
