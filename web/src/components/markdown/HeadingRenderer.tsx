import React from 'react';

interface HeadingRendererProps {
  level: 1 | 2 | 3 | 4 | 5 | 6;
  children: React.ReactNode;
  id?: string;
}

function generateHeadingId(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .trim();
}

function extractTextFromChildren(children: React.ReactNode): string {
  if (typeof children === 'string') {
    return children;
  }
  
  if (React.isValidElement(children)) {
    const props = children.props as { children?: React.ReactNode };
    return extractTextFromChildren(props.children);
  }
  
  if (Array.isArray(children)) {
    return children.map(extractTextFromChildren).join('');
  }
  
  return '';
}

export default function HeadingRenderer({ level, children, id }: HeadingRendererProps) {
  const textContent = extractTextFromChildren(children);
  const headingId = id || generateHeadingId(textContent);
  
  const HeadingTag = `h${level}` as 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
  
  const handleAnchorClick = (e: React.MouseEvent) => {
    e.preventDefault();
    window.location.hash = `#${headingId}`;
    document.getElementById(headingId)?.scrollIntoView({ 
      behavior: 'smooth' 
    });
  };
  
  return (
    <HeadingTag
      id={headingId}
      className={`md-heading md-heading-${level}`}
    >
      <a 
        href={`#${headingId}`}
        onClick={handleAnchorClick}
        className="md-heading-anchor"
        aria-label={`Link to ${textContent}`}
      >
        {children}
        <span className="md-heading-anchor-icon">🔗</span>
      </a>
    </HeadingTag>
  );
}