'use client';

import React from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { DiffHunk, DiffLine } from '@/lib/git';

interface DiffViewerProps {
  hunks: DiffHunk[];
  filePath: string;
}

interface SyntaxHighlightedDiffLineProps {
  content: string;
  filePath: string;
  type: 'addition' | 'deletion' | 'context';
}

function detectLanguageFromPath(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase();
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
    'yml': 'yaml',
    'yaml': 'yaml',
    'json': 'json',
    'md': 'markdown',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sql': 'sql',
    'xml': 'xml',
    'toml': 'toml',
    'ini': 'ini',
    'dockerfile': 'dockerfile',
    'makefile': 'makefile'
  };
  
  // Special case for files without extensions
  const fileName = filePath.split('/').pop()?.toLowerCase() || '';
  if (fileName === 'dockerfile' || fileName === 'containerfile') return 'dockerfile';
  if (fileName === 'makefile' || fileName === 'makefile.in') return 'makefile';
  if (fileName.startsWith('.env')) return 'bash';
  if (fileName === 'cargo.toml' || fileName === 'pyproject.toml') return 'toml';
  
  return languageMap[ext || ''] || 'text';
}

function SyntaxHighlightedDiffLine({ content, filePath, type }: SyntaxHighlightedDiffLineProps) {
  const language = detectLanguageFromPath(filePath);
  
  // For empty lines or text files, don't highlight
  if (language === 'text' || content.trim() === '') {
    return <span className="diff-line-text-content">{content}</span>;
  }
  
  try {
    return (
      <SyntaxHighlighter
        language={language}
        style={vscDarkPlus}
        customStyle={{
          margin: 0,
          padding: 0,
          background: 'transparent',
          fontSize: '11px',
          lineHeight: '18px',
          fontFamily: 'JetBrains Mono, monospace'
        }}
        codeTagProps={{
          style: {
            background: 'transparent',
            fontFamily: 'inherit',
            fontSize: 'inherit'
          }
        }}
        PreTag={'span'}
      >
        {content}
      </SyntaxHighlighter>
    );
  } catch (error) {
    // Fallback to plain text if highlighting fails
    return <span className="diff-line-text-content">{content}</span>;
  }
}

export default function DiffViewer({ hunks, filePath }: DiffViewerProps) {
  if (hunks.length === 0) {
    // Check if this might be a large file that was skipped
    const fileName = filePath.split('/').pop() || filePath;
    return (
      <div className="diff-empty">
        <p>
          {fileName.includes('.') ? 
            'Binary file, large file, or no diff available' : 
            'No diff available for this file'
          }
        </p>
        <p className="diff-empty-hint">
          File may be binary, too large (&gt;2000 lines), or newly created empty file.
        </p>
      </div>
    );
  }

  // Calculate total lines for performance warning
  const totalLines = hunks.reduce((sum, hunk) => sum + hunk.lines.length, 0);
  
  return (
    <div className="diff-content">
      {totalLines > 200 && (
        <div className="diff-performance-warning">
          <p>⚠ Large diff ({totalLines} lines) - may affect performance</p>
        </div>
      )}
      
      {hunks.map((hunk, hunkIndex) => (
        <div key={hunkIndex} className="diff-hunk">
          <div className="diff-hunk-header">
            @@ -{hunk.oldStart},{hunk.oldCount} +{hunk.newStart},{hunk.newCount} @@
          </div>
          <div className="diff-lines">
            {hunk.lines.map((line, lineIndex) => (
              <div key={lineIndex} className={`diff-line diff-line-${line.type}`}>
                <div className="diff-line-numbers">
                  <span className="diff-line-old">
                    {line.oldLineNumber || ''}
                  </span>
                  <span className="diff-line-new">
                    {line.newLineNumber || ''}
                  </span>
                </div>
                <div className="diff-line-content">
                  <span className="diff-line-indicator">
                    {line.type === 'addition' ? '+' : 
                     line.type === 'deletion' ? '-' : ' '}
                  </span>
                  <SyntaxHighlightedDiffLine 
                    content={line.content}
                    filePath={filePath}
                    type={line.type}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}