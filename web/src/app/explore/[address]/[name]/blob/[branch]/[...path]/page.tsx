'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, formatBytes, getFileIcon } from '@/lib/utils';
import { repoUrls, generateBreadcrumbs } from '@/lib/repoUrls';
import { SiteNav } from '@/components/SiteNav';
import MarkdownRenderer from '@/components/markdown/MarkdownRenderer';

interface Commit {
  hash: string; author: string; timestamp: number; message: string;
}

interface FileContent {
  path: string; content: string; size: number;
}

function getFileLanguage(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  const map: Record<string, string> = {
    js: 'javascript', jsx: 'jsx', ts: 'typescript', tsx: 'tsx',
    rs: 'rust', go: 'go', py: 'python', java: 'java',
    c: 'c', cpp: 'cpp', html: 'html', css: 'css',
    sh: 'bash', json: 'json', yml: 'yaml', yaml: 'yaml',
    toml: 'toml', sol: 'solidity', md: 'markdown', sql: 'sql',
  };
  return map[ext] || 'text';
}

function isMarkdown(p: string): boolean {
  return /\.(md|markdown)$/i.test(p);
}

function isBinary(p: string): boolean {
  const ext = p.split('.').pop()?.toLowerCase() || '';
  return ['png','jpg','jpeg','gif','webp','ico','svg','mp3','mp4','zip','tar','gz','exe','pdf','woff','woff2','ttf','eot'].includes(ext);
}

export default function BlobPage() {
  const params = useParams();
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [recentCommits, setRecentCommits] = useState<Commit[]>([]);
  const [repoName, setRepoName] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const branch = Array.isArray(params.branch) ? params.branch[0] : params.branch;
  const pathSegments = params.path ? (Array.isArray(params.path) ? params.path : [params.path]) : [];
  const filePath = pathSegments.join('/');

  useEffect(() => {
    if (!address || !name || !branch || !filePath) return;
    const fetchData = async () => {
      try {
        setLoading(true);
        const [fileRes, commitsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${address}/${name}/blob/${filePath}?branch=${branch}`),
          fetch(`/api/explorer/repos/${address}/${name}/commits?limit=5&branch=${branch}`),
        ]);
        if (!fileRes.ok) throw new Error(fileRes.status === 404 ? 'File not found' : 'Failed to load file');
        const fileData = await fileRes.json();
        setFileContent(fileData);
        setRepoName(name);
        if (commitsRes.ok) {
          const cd = await commitsRes.json();
          setRecentCommits(cd.commits || []);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load file');
      } finally { setLoading(false); }
    };
    fetchData();
  }, [address, name, branch, filePath]);

  if (!address || !name || !branch || !filePath) return notFound();

  const fileName = filePath.split('/').pop() || '';
  const parentPath = pathSegments.slice(0, -1).join('/');

  if (loading) {
    return (
      <div className="blob-root">
        <SiteNav />
        <div className="blob-center"><div className="blob-spinner" /><p>Loading…</p></div>
      </div>
    );
  }

  if (error || !fileContent) {
    return (
      <div className="blob-root">
        <SiteNav />
        <div className="blob-center">
          <h3>File not found</h3>
          <p>{error}</p>
          <Link href={repoUrls.home(address, name)} style={{ color: 'var(--bp-accent)' }}>← Back to repository</Link>
        </div>
      </div>
    );
  }

  return (
    <div className="blob-root">
      <SiteNav />

      {/* Breadcrumb header */}
      <div className="blob-header">
        <div className="blob-header-inner">
          <div className="blob-breadcrumb">
            <Link href="/explore">explore</Link>
            <span>/</span>
            <Link href={`/explore/${address}`}>{formatAddress(address)}</Link>
            <span>/</span>
            <Link href={repoUrls.home(address, name)}>{name}</Link>
            <span>/</span>
            {pathSegments.slice(0, -1).map((seg, i) => (
              <span key={i}>
                <Link href={repoUrls.tree(address, name, branch, pathSegments.slice(0, i + 1).join('/'))}>{seg}</Link>
                <span>/</span>
              </span>
            ))}
            <span className="blob-breadcrumb-file">{fileName}</span>
            <span className="blob-breadcrumb-branch">@ {branch}</span>
          </div>
        </div>
      </div>

      {/* Content grid: file left, sidebar right */}
      <div className="blob-content">
        <main className="blob-main">
          {/* File viewer */}
          <div className="blob-file-card">
            <div className="blob-file-header">
              <div className="blob-file-info">
                <span className="blob-file-icon">{getFileIcon(fileName, false)}</span>
                <span className="blob-file-name">{fileName}</span>
                {fileContent.size > 0 && <span className="blob-file-size">{formatBytes(fileContent.size)}</span>}
              </div>
              <div className="blob-file-actions">
                {parentPath && (
                  <Link href={repoUrls.tree(address, name, branch, parentPath)} className="blob-action-btn">↑ Parent</Link>
                )}
                <Link href={repoUrls.home(address, name)} className="blob-action-btn">Repository</Link>
              </div>
            </div>

            <div className="blob-file-body">
              {isBinary(filePath) ? (
                <div className="blob-binary">
                  <p>Binary file — {formatBytes(fileContent.size)}</p>
                </div>
              ) : isMarkdown(filePath) ? (
                <div className="blob-markdown">
                  <MarkdownRenderer
                    content={fileContent.content}
                    baseUrl={`/api/explorer/repos/${address}/${name}/blob/`}
                    className="blob-md-content"
                  />
                </div>
              ) : (
                <pre className="blob-code"><code>{fileContent.content}</code></pre>
              )}
            </div>
          </div>
        </main>

        {/* Sidebar */}
        <aside className="blob-sidebar">
          <div className="blob-sidebar-card">
            <div className="blob-sidebar-label">Recent Commits</div>
            {recentCommits.length === 0 ? (
              <p className="blob-dim">No commits</p>
            ) : (
              recentCommits.map(c => (
                <div key={c.hash} className="blob-sidebar-commit">
                  <Link href={repoUrls.commit(address, name, c.hash)} className="blob-sidebar-commit-msg">
                    {c.message.split('\n')[0].substring(0, 60)}{c.message.length > 60 ? '…' : ''}
                  </Link>
                  <div className="blob-sidebar-commit-meta">
                    <code>{c.hash.slice(0, 7)}</code>
                    <span>{formatTimeAgo(new Date(c.timestamp * 1000).toISOString())}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </aside>
      </div>

      <style>{`
        .blob-root {
          min-height: 100vh;
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono, 'JetBrains Mono', monospace);
          font-size: 13px;
        }
        .blob-center {
          display: flex; flex-direction: column;
          align-items: center; justify-content: center;
          min-height: 60vh; gap: 12px; color: var(--bp-dim);
        }
        .blob-spinner {
          width: 24px; height: 24px;
          border: 2px solid var(--bp-border);
          border-top-color: var(--bp-accent);
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }
        @keyframes spin { to { transform: rotate(360deg); } }

        /* Header */
        .blob-header {
          border-bottom: 1px solid var(--bp-border);
          padding: 16px 32px;
        }
        .blob-header-inner { max-width: 1280px; margin: 0 auto; }
        .blob-breadcrumb {
          display: flex; align-items: center; gap: 6px;
          font-size: 12px; color: var(--bp-dim);
          flex-wrap: wrap;
        }
        .blob-breadcrumb a { color: var(--bp-accent); text-decoration: none; }
        .blob-breadcrumb a:hover { opacity: 0.8; }
        .blob-breadcrumb-file { color: var(--bp-heading); font-weight: 600; }
        .blob-breadcrumb-branch {
          color: var(--bp-dim); opacity: 0.5;
          margin-left: 4px; font-size: 11px;
        }

        /* Content grid */
        .blob-content {
          max-width: 1280px; margin: 0 auto;
          padding: 24px 32px;
          display: grid;
          grid-template-columns: 1fr 280px;
          gap: 24px;
          align-items: start;
        }

        /* File card */
        .blob-file-card {
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          overflow: hidden;
        }
        .blob-file-header {
          display: flex; align-items: center; justify-content: space-between;
          padding: 10px 16px;
          background: var(--bp-surface);
          border-bottom: 1px solid var(--bp-border);
          gap: 12px; flex-wrap: wrap;
        }
        .blob-file-info {
          display: flex; align-items: center; gap: 8px;
        }
        .blob-file-icon { font-size: 14px; }
        .blob-file-name { color: var(--bp-heading); font-weight: 500; font-size: 13px; }
        .blob-file-size { color: var(--bp-dim); font-size: 11px; opacity: 0.6; }
        .blob-file-actions { display: flex; gap: 8px; }
        .blob-action-btn {
          padding: 4px 10px;
          border: 1px solid var(--bp-border);
          border-radius: 4px;
          color: var(--bp-dim); font-size: 11px;
          text-decoration: none;
          transition: all 0.12s;
        }
        .blob-action-btn:hover {
          border-color: rgba(79, 195, 247, 0.3);
          color: var(--bp-accent);
        }

        .blob-file-body { background: var(--bp-bg); }
        .blob-code {
          margin: 0; padding: 16px 20px;
          font-size: 12px; line-height: 1.6;
          overflow-x: auto; color: var(--bp-text);
        }
        .blob-markdown { padding: 24px; }
        .blob-binary {
          padding: 48px; text-align: center;
          color: var(--bp-dim);
        }

        /* Sidebar */
        .blob-sidebar { }
        .blob-sidebar-card {
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          padding: 16px;
        }
        .blob-sidebar-label {
          font-size: 10px; font-weight: 600;
          text-transform: uppercase; letter-spacing: 0.1em;
          color: var(--bp-dim); margin-bottom: 12px;
        }
        .blob-sidebar-commit { margin-bottom: 10px; }
        .blob-sidebar-commit:last-child { margin-bottom: 0; }
        .blob-sidebar-commit-msg {
          color: var(--bp-text); text-decoration: none;
          font-size: 12px; line-height: 1.4; display: block;
        }
        .blob-sidebar-commit-msg:hover { color: var(--bp-accent); }
        .blob-sidebar-commit-meta {
          display: flex; gap: 8px; font-size: 10px;
          color: var(--bp-dim); opacity: 0.6; margin-top: 2px;
        }
        .blob-dim { color: var(--bp-dim); font-size: 12px; }

        @media (max-width: 900px) {
          .blob-content {
            grid-template-columns: 1fr;
            padding: 16px;
          }
          .blob-main { order: 1; }
          .blob-sidebar { order: 2; }
          .blob-header { padding: 12px 16px; }
        }
      `}</style>
    </div>
  );
}
