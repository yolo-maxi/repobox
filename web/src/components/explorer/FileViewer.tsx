'use client';

import React, { useState, useMemo } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { formatBytes, getFileIcon, copyToClipboard } from '@/lib/utils';

interface FileViewerProps {
  filePath: string;
  fileContent: string;
  onClose?: () => void;
}

interface FileStats {
  size: number;
  lineCount: number;
  isBinary: boolean;
}

function getLanguageFromPath(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase();
  
  const languageMap: Record<string, string> = {
    'rs': 'rust',
    'js': 'javascript',
    'jsx': 'jsx', 
    'ts': 'typescript',
    'tsx': 'tsx',
    'py': 'python',
    'go': 'go',
    'java': 'java',
    'cpp': 'cpp',
    'cc': 'cpp',
    'cxx': 'cpp',
    'c': 'c',
    'h': 'c',
    'hpp': 'cpp',
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
    'jsonc': 'json',
    'md': 'markdown',
    'mdx': 'markdown',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'scss',
    'less': 'less',
    'sql': 'sql',
    'xml': 'xml',
    'toml': 'toml',
    'ini': 'ini',
    'cfg': 'ini',
    'conf': 'ini',
    'dockerfile': 'dockerfile',
    'makefile': 'makefile',
    'vim': 'vim',
    'lua': 'lua',
    'perl': 'perl',
    'r': 'r'
  };
  
  // Special cases for files without extensions
  const fileName = path.split('/').pop()?.toLowerCase();
  if (fileName === 'dockerfile') return 'dockerfile';
  if (fileName === 'makefile') return 'makefile';
  
  return languageMap[ext || ''] || 'text';
}

function getFileType(language: string): string {
  const typeMap: Record<string, string> = {
    'rust': 'Rust source file',
    'javascript': 'JavaScript source file',
    'jsx': 'React JSX component',
    'typescript': 'TypeScript source file', 
    'tsx': 'React TSX component',
    'python': 'Python script',
    'go': 'Go source file',
    'java': 'Java source file',
    'cpp': 'C++ source file',
    'c': 'C source file',
    'csharp': 'C# source file',
    'php': 'PHP script',
    'ruby': 'Ruby script',
    'bash': 'Shell script',
    'yaml': 'YAML configuration',
    'json': 'JSON data',
    'markdown': 'Markdown document',
    'html': 'HTML document',
    'css': 'CSS stylesheet',
    'scss': 'SCSS stylesheet',
    'sql': 'SQL script',
    'xml': 'XML document',
    'toml': 'TOML configuration',
    'dockerfile': 'Docker configuration',
    'makefile': 'Makefile'
  };
  
  return typeMap[language] || 'Text file';
}

function analyzeFile(content: string): FileStats {
  const size = new Blob([content]).size;
  const lineCount = content.split('\n').length;
  
  // Simple binary detection - check for null bytes or high ratio of non-printable chars
  const nullBytes = content.includes('\0');
  const printableRatio = content.replace(/[\x20-\x7E\r\n\t]/g, '').length / content.length;
  const isBinary = nullBytes || printableRatio > 0.3;
  
  return { size, lineCount, isBinary };
}

export function FileViewer({ filePath, fileContent, onClose }: FileViewerProps) {
  const [showRaw, setShowRaw] = useState(false);
  const [copied, setCopied] = useState(false);
  
  const fileName = filePath.split('/').pop() || '';
  const language = getLanguageFromPath(filePath);
  const fileType = getFileType(language);
  const stats = useMemo(() => analyzeFile(fileContent), [fileContent]);
  
  const handleCopy = async () => {
    await copyToClipboard(fileContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  
  const handleDownload = () => {
    const blob = new Blob([fileContent], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };
  
  // Large file fallback (>100KB)
  const shouldUseRaw = stats.size > 100 * 1024;
  
  // Binary file handling
  if (stats.isBinary) {
    return (
      <div className="file-viewer">
        <div className="file-viewer-header">
          <div className="file-viewer-title">
            <span className="file-viewer-icon">{getFileIcon(fileName, false)}</span>
            <span className="file-viewer-name">{fileName}</span>
          </div>
          <div className="file-viewer-actions">
            <button onClick={handleDownload} className="file-viewer-btn">
              📥 Download
            </button>
            {onClose && (
              <button onClick={onClose} className="file-viewer-btn">
                ✕ Close
              </button>
            )}
          </div>
        </div>
        
        <div className="file-viewer-metadata">
          <span>{formatBytes(stats.size)}</span>
          <span>•</span>
          <span>Binary file</span>
        </div>
        
        <div className="file-viewer-binary">
          <div className="file-viewer-binary-icon">📄</div>
          <p>Binary file not shown</p>
          <p className="file-viewer-binary-hint">
            Download the file to view its contents
          </p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="file-viewer">
      <div className="file-viewer-header">
        <div className="file-viewer-title">
          <span className="file-viewer-icon">{getFileIcon(fileName, false)}</span>
          <span className="file-viewer-name">{fileName}</span>
        </div>
        <div className="file-viewer-actions">
          <button onClick={handleDownload} className="file-viewer-btn">
            📥 Download
          </button>
          {onClose && (
            <button onClick={onClose} className="file-viewer-btn">
              ✕ Close
            </button>
          )}
        </div>
      </div>
      
      <div className="file-viewer-metadata">
        <span>{stats.lineCount.toLocaleString()} lines</span>
        <span>•</span>
        <span>{formatBytes(stats.size)}</span>
        <span>•</span>
        <span>UTF-8</span>
        <span>•</span>
        <span>{fileType}</span>
      </div>
      
      <div className="file-viewer-toolbar">
        <button 
          onClick={handleCopy}
          className="file-viewer-btn"
          title="Copy file content"
        >
          {copied ? '✓ Copied' : '📋 Copy'}
        </button>
        <button
          onClick={() => setShowRaw(!showRaw)}
          className={`file-viewer-btn ${showRaw ? 'active' : ''}`}
          title="Toggle raw text view"
        >
          Raw
        </button>
        {shouldUseRaw && (
          <span className="file-viewer-warning">
            ⚠️ Large file - showing plain text
          </span>
        )}
      </div>
      
      <div className="file-viewer-content">
        {showRaw || shouldUseRaw || language === 'text' ? (
          <pre className="file-viewer-pre">
            <code className="file-viewer-code">{fileContent}</code>
          </pre>
        ) : (
          <SyntaxHighlighter
            language={language}
            style={vscDarkPlus}
            showLineNumbers={true}
            customStyle={{
              margin: 0,
              borderRadius: 0,
              fontSize: '12px',
              lineHeight: '20px',
              fontFamily: 'JetBrains Mono, monospace',
              background: 'transparent'
            }}
            lineNumberStyle={{
              color: 'var(--bp-dim)',
              backgroundColor: 'rgba(50, 100, 160, 0.05)',
              borderRight: '1px solid var(--bp-border)',
              paddingRight: '12px',
              marginRight: '12px',
              userSelect: 'none',
              minWidth: '3em',
              textAlign: 'right'
            }}
            codeTagProps={{
              style: {
                fontFamily: 'inherit',
                fontSize: 'inherit'
              }
            }}
          >
            {fileContent}
          </SyntaxHighlighter>
        )}
      </div>
    </div>
  );
}