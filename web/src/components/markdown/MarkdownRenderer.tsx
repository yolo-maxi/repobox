import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Components } from 'react-markdown';
import CodeBlock from './CodeBlock';
import TableRenderer from './TableRenderer';
import HeadingRenderer from './HeadingRenderer';
import ImageRenderer from './ImageRenderer';
import LinkRenderer from './LinkRenderer';

interface MarkdownRendererProps {
  content: string;
  baseUrl?: string;
  className?: string;
}

export default function MarkdownRenderer({ 
  content, 
  baseUrl, 
  className = '' 
}: MarkdownRendererProps) {
  const components: Components = {
    // Code blocks with syntax highlighting
    code: ({ className, children, ...props }) => {
      // Check if this is inline code (no className typically means inline)
      const isInline = !className || !className.includes('language-');
      
      if (isInline) {
        return (
          <code className="md-inline-code" {...props}>
            {children}
          </code>
        );
      }
      
      return (
        <CodeBlock 
          className={className} 
          enableCopy={true}
          showLanguage={true}
        >
          {String(children)}
        </CodeBlock>
      );
    },
    
    // Remove the default pre wrapper
    pre: ({ children }) => <>{children}</>,
    
    // Enhanced table rendering
    table: ({ children }) => <TableRenderer>{children}</TableRenderer>,
    
    // Headings with anchor links
    h1: ({ children }) => <HeadingRenderer level={1}>{children}</HeadingRenderer>,
    h2: ({ children }) => <HeadingRenderer level={2}>{children}</HeadingRenderer>,
    h3: ({ children }) => <HeadingRenderer level={3}>{children}</HeadingRenderer>,
    h4: ({ children }) => <HeadingRenderer level={4}>{children}</HeadingRenderer>,
    h5: ({ children }) => <HeadingRenderer level={5}>{children}</HeadingRenderer>,
    h6: ({ children }) => <HeadingRenderer level={6}>{children}</HeadingRenderer>,
    
    // Enhanced image handling
    img: ({ src, alt, title }) => (
      <ImageRenderer 
        src={src} 
        alt={alt} 
        title={title} 
      />
    ),
    
    // Enhanced link handling
    a: ({ href, children, title }) => (
      <LinkRenderer href={href} title={title}>
        {children}
      </LinkRenderer>
    ),
    
    // Styled blockquotes
    blockquote: ({ children }) => (
      <blockquote className="md-blockquote">
        {children}
      </blockquote>
    ),
    
    // Styled lists
    ul: ({ children }) => <ul className="md-list md-list-unordered">{children}</ul>,
    ol: ({ children }) => <ol className="md-list md-list-ordered">{children}</ol>,
    li: ({ children }) => <li className="md-list-item">{children}</li>,
    
    // Enhanced table components
    th: ({ children }) => <th className="md-table-header">{children}</th>,
    td: ({ children }) => <td className="md-table-cell">{children}</td>,
    
    // Paragraph styling
    p: ({ children }) => <p className="md-paragraph">{children}</p>,
    
    // Strong and emphasis
    strong: ({ children }) => <strong className="md-strong">{children}</strong>,
    em: ({ children }) => <em className="md-emphasis">{children}</em>,
    
    // Horizontal rule
    hr: () => <hr className="md-hr" />,
  };

  return (
    <div className={`md-container ${className}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={components}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}