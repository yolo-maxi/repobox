import React from 'react';

interface LinkRendererProps {
  href?: string;
  children: React.ReactNode;
  title?: string;
}

export default function LinkRenderer({ href, children, title }: LinkRendererProps) {
  if (!href) {
    return <span className="md-link-disabled">{children}</span>;
  }

  // Check if it's an external link
  const isExternal = href.startsWith('http://') || href.startsWith('https://');
  const isAnchor = href.startsWith('#');
  
  // Handle anchor links (internal navigation)
  if (isAnchor) {
    const handleAnchorClick = (e: React.MouseEvent) => {
      e.preventDefault();
      const target = document.getElementById(href.slice(1));
      if (target) {
        target.scrollIntoView({ behavior: 'smooth' });
        window.location.hash = href;
      }
    };

    return (
      <a
        href={href}
        onClick={handleAnchorClick}
        className="md-link md-link-anchor"
        title={title}
      >
        {children}
      </a>
    );
  }

  // External links
  if (isExternal) {
    return (
      <a
        href={href}
        target="_blank"
        rel="noopener noreferrer"
        className="md-link md-link-external"
        title={title}
      >
        {children}
        <span className="md-link-external-icon">↗</span>
      </a>
    );
  }

  // Internal/relative links
  return (
    <a
      href={href}
      className="md-link md-link-internal"
      title={title}
    >
      {children}
    </a>
  );
}