'use client';

import React, { useState } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface CodeBlockProps {
  children: string;
  className?: string;
  language?: string;
  showLineNumbers?: boolean;
  showLanguage?: boolean;
  enableCopy?: boolean;
}

function detectLanguageFromClassName(className: string = ''): string {
  const match = className.match(/language-(\w+)/);
  if (!match) return 'text';
  
  const lang = match[1].toLowerCase();
  
  // Map common aliases to supported languages
  const languageMap: Record<string, string> = {
    'js': 'javascript',
    'jsx': 'jsx',
    'ts': 'typescript',
    'tsx': 'tsx',
    'py': 'python',
    'rs': 'rust',
    'go': 'go',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c',
    'cs': 'csharp',
    'php': 'php',
    'rb': 'ruby',
    'sh': 'bash',
    'bash': 'bash',
    'zsh': 'bash',
    'fish': 'bash',
    'yml': 'yaml',
    'yaml': 'yaml',
    'json': 'json',
    'md': 'markdown',
    'markdown': 'markdown',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'scss',
    'sql': 'sql',
    'xml': 'xml',
    'toml': 'toml',
    'ini': 'ini',
    'dockerfile': 'dockerfile',
    'makefile': 'makefile',
    'vim': 'vim'
  };
  
  return languageMap[lang] || lang;
}

export default function CodeBlock({ 
  children, 
  className, 
  language,
  showLineNumbers = false,
  showLanguage = true,
  enableCopy = true
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const detectedLanguage = language || detectLanguageFromClassName(className);
  const code = children.trim();

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy code:', error);
    }
  };

  // For very short code or text, don't highlight
  if (detectedLanguage === 'text' || code.length < 10) {
    return (
      <div className="md-code-block">
        <div className="md-code-header">
          {showLanguage && (
            <span className="md-code-language">text</span>
          )}
          {enableCopy && (
            <button 
              onClick={handleCopy}
              className="md-code-copy-btn"
              title="Copy code"
            >
              {copied ? '✓ Copied' : '📋 Copy'}
            </button>
          )}
        </div>
        <pre className="md-code-pre">
          <code className="md-code-content">{code}</code>
        </pre>
      </div>
    );
  }

  try {
    return (
      <div className="md-code-block">
        <div className="md-code-header">
          {showLanguage && (
            <span className="md-code-language">{detectedLanguage}</span>
          )}
          {enableCopy && (
            <button 
              onClick={handleCopy}
              className="md-code-copy-btn"
              title="Copy code"
            >
              {copied ? '✓ Copied' : '📋 Copy'}
            </button>
          )}
        </div>
        <SyntaxHighlighter
          language={detectedLanguage}
          style={vscDarkPlus}
          showLineNumbers={showLineNumbers}
          customStyle={{
            margin: 0,
            borderRadius: '0 0 8px 8px',
            fontSize: '12px',
            lineHeight: '20px',
            fontFamily: 'JetBrains Mono, monospace',
            background: '#0d1f35'
          }}
          codeTagProps={{
            style: {
              fontFamily: 'inherit',
              fontSize: 'inherit'
            }
          }}
        >
          {code}
        </SyntaxHighlighter>
      </div>
    );
  } catch (error) {
    // Fallback if highlighting fails
    return (
      <div className="md-code-block">
        <div className="md-code-header">
          {showLanguage && (
            <span className="md-code-language">{detectedLanguage}</span>
          )}
          {enableCopy && (
            <button 
              onClick={handleCopy}
              className="md-code-copy-btn"
              title="Copy code"
            >
              {copied ? '✓ Copied' : '📋 Copy'}
            </button>
          )}
        </div>
        <pre className="md-code-pre">
          <code className="md-code-content">{code}</code>
        </pre>
      </div>
    );
  }
}