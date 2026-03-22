'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState, useEffect } from 'react';
import { Menu, X, Play, BookOpen, Compass, Home, Palette } from 'lucide-react';

const NAV_LINKS = [
  { href: '/', label: 'home', icon: Home },
  { href: '/explore', label: 'explorer', icon: Compass },
  { href: '/docs', label: 'docs', icon: BookOpen },
  { href: '/playground', label: 'playground', icon: Play },
];

const TRAY_EXTRA_LINKS = [
  { href: '/brand', label: 'brand', icon: Palette },
];

export function SiteNav() {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 40);
    window.addEventListener('scroll', handleScroll, { passive: true });
    handleScroll();
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  // Close menu on route change
  useEffect(() => { setMenuOpen(false); }, [pathname]);

  const isActive = (href: string) => {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  };

  return (
    <>
      <nav className={`site-nav ${scrolled ? 'scrolled' : ''}`}>
        <Link href="/" className="site-nav-logo">
          repo<span style={{ color: 'var(--bp-accent)' }}>.</span>box
        </Link>

        {/* Desktop links */}
        <div className="site-nav-links">
          {NAV_LINKS.map(l => (
            <Link
              key={l.href}
              href={l.href}
              className={`site-nav-link ${isActive(l.href) ? 'active' : ''}`}
            >
              {l.label}
            </Link>
          ))}
        </div>

        {/* Mobile hamburger */}
        <button
          className="site-nav-hamburger"
          onClick={() => setMenuOpen(!menuOpen)}
          aria-label="Toggle menu"
        >
          {menuOpen ? <X size={20} /> : <Menu size={20} />}
        </button>
      </nav>

      {/* Mobile slide-out tray */}
      {menuOpen && <div className="site-nav-backdrop" onClick={() => setMenuOpen(false)} />}
      <div className={`site-nav-tray ${menuOpen ? 'open' : ''}`}>
        <div className="site-nav-tray-header">
          <span className="site-nav-tray-logo">
            repo<span style={{ color: 'var(--bp-accent)' }}>.</span>box
          </span>
          <button className="site-nav-tray-close" onClick={() => setMenuOpen(false)}>
            <X size={18} />
          </button>
        </div>

        <div className="site-nav-tray-links">
          {[...NAV_LINKS, ...TRAY_EXTRA_LINKS].map(l => {
            const Icon = l.icon;
            return (
              <Link
                key={l.href}
                href={l.href}
                className={`site-nav-tray-link ${isActive(l.href) ? 'active' : ''}`}
                onClick={() => setMenuOpen(false)}
              >
                <Icon size={16} />
                {l.label}
              </Link>
            );
          })}
        </div>

        <div className="site-nav-tray-footer">
          <div className="site-nav-tray-footer-label">Resources</div>
          <a href="/llms.txt" className="site-nav-tray-footer-link">llms.txt</a>
          <a href="/SKILL.md" className="site-nav-tray-footer-link">SKILL.md</a>
          <a href="https://github.com/yolo-maxi/repobox" className="site-nav-tray-footer-link" target="_blank" rel="noopener">GitHub</a>
        </div>

        <div className="site-nav-tray-footer">
          <div className="site-nav-tray-footer-label">Say hi</div>
          <a href="https://warpcast.com/0xfran" className="site-nav-tray-footer-link" target="_blank" rel="noopener">Fran</a>
          <a href="https://warpcast.com/oceanvael" className="site-nav-tray-footer-link" target="_blank" rel="noopener">Ocean</a>
        </div>
      </div>
    </>
  );
}
