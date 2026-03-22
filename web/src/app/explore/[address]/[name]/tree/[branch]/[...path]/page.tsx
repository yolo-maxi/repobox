'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, formatBytes } from '@/lib/utils';
import { repoUrls } from '@/lib/repoUrls';
import { SiteNav } from '@/components/SiteNav';
import FileTree from '@/components/explore/FileTree';

interface FileEntry {
  type: 'blob' | 'tree';
  name: string;
  size?: number;
  path: string;
}

interface Commit {
  hash: string; author: string; timestamp: number; message: string;
}

export default function TreePage() {
  const params = useParams();
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [recentCommits, setRecentCommits] = useState<Commit[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const branch = Array.isArray(params.branch) ? params.branch[0] : params.branch;
  const pathSegments = params.path ? (Array.isArray(params.path) ? params.path : [params.path]) : [];
  const treePath = pathSegments.join('/');

  useEffect(() => {
    if (!address || !name || !branch) return;
    const fetchData = async () => {
      try {
        setLoading(true);
        const [treeRes, commitsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${address}/${name}/tree?path=${encodeURIComponent(treePath)}&branch=${branch}`),
          fetch(`/api/explorer/repos/${address}/${name}/commits?limit=5&branch=${branch}`),
        ]);
        if (!treeRes.ok) throw new Error('Directory not found');
        const td = await treeRes.json();
        setFiles(td.files || []);
        if (commitsRes.ok) {
          const cd = await commitsRes.json();
          setRecentCommits(cd.commits || []);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load');
      } finally { setLoading(false); }
    };
    fetchData();
  }, [address, name, branch, treePath]);

  if (!address || !name || !branch) return notFound();

  if (loading) {
    return (
      <div className="tree-root">
        <SiteNav />
        <div className="tree-center"><div className="tree-spinner" /><p>Loading…</p></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="tree-root">
        <SiteNav />
        <div className="tree-center">
          <h3>Directory not found</h3>
          <p>{error}</p>
          <Link href={repoUrls.home(address, name)} style={{ color: 'var(--bp-accent)' }}>← Back to repository</Link>
        </div>
      </div>
    );
  }

  const handleFileClick = (filePath: string) => {
    window.location.href = repoUrls.blob(address, name, branch, filePath);
  };

  return (
    <div className="tree-root">
      <SiteNav />

      {/* Breadcrumb header */}
      <div className="tree-header">
        <div className="tree-header-inner">
          <div className="tree-breadcrumb">
            <Link href="/explore">explore</Link>
            <span>/</span>
            <Link href={`/explore/${address}`}>{formatAddress(address)}</Link>
            <span>/</span>
            <Link href={repoUrls.home(address, name)}>{name}</Link>
            <span>/</span>
            {pathSegments.map((seg, i) => (
              <span key={i}>
                {i > 0 && <span>/</span>}
                <Link href={repoUrls.tree(address, name, branch, pathSegments.slice(0, i + 1).join('/'))}>{seg}</Link>
              </span>
            ))}
            <span className="tree-breadcrumb-branch">@ {branch}</span>
          </div>
        </div>
      </div>

      {/* Content grid */}
      <div className="tree-content">
        <main className="tree-main">
          {/* Folder header — matches blob file header */}
          <div className="tree-folder-card">
            <div className="tree-folder-header">
              <div className="tree-folder-info">
                <span className="tree-folder-icon">📁</span>
                <span className="tree-folder-name">{pathSegments[pathSegments.length - 1] || name}</span>
              </div>
              <div className="tree-folder-actions">
                {pathSegments.length > 1 ? (
                  <Link href={repoUrls.tree(address, name, branch, pathSegments.slice(0, -1).join('/'))} className="tree-action-btn">↑ Parent</Link>
                ) : pathSegments.length === 1 ? (
                  <Link href={repoUrls.home(address, name)} className="tree-action-btn">↑ Parent</Link>
                ) : null}
                <Link href={repoUrls.home(address, name)} className="tree-action-btn">Repository</Link>
              </div>
            </div>
          </div>

          <FileTree
            address={address}
            repoName={name}
            branch={branch}
            initialFiles={files}
            onFileClick={handleFileClick}
          />
        </main>

        <aside className="tree-sidebar">
          <div className="tree-sidebar-card">
            <div className="tree-sidebar-label">Recent Commits</div>
            {recentCommits.length === 0 ? (
              <p className="tree-dim">No commits</p>
            ) : (
              recentCommits.map(c => (
                <div key={c.hash} className="tree-sidebar-commit">
                  <Link href={repoUrls.commit(address, name, c.hash)} className="tree-sidebar-commit-msg">
                    {c.message.split('\n')[0].substring(0, 60)}{c.message.length > 60 ? '…' : ''}
                  </Link>
                  <div className="tree-sidebar-commit-meta">
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
        .tree-root {
          min-height: 100vh;
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono, 'JetBrains Mono', monospace);
          font-size: 13px;
        }
        .tree-center {
          display: flex; flex-direction: column;
          align-items: center; justify-content: center;
          min-height: 60vh; gap: 12px; color: var(--bp-dim);
        }
        .tree-spinner {
          width: 24px; height: 24px;
          border: 2px solid var(--bp-border);
          border-top-color: var(--bp-accent);
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }
        @keyframes spin { to { transform: rotate(360deg); } }

        .tree-header {
          border-bottom: 1px solid var(--bp-border);
          padding: 16px 32px;
        }
        .tree-header-inner { max-width: 1280px; margin: 0 auto; }
        .tree-breadcrumb {
          display: flex; align-items: center; gap: 6px;
          font-size: 12px; color: var(--bp-dim); flex-wrap: wrap;
        }
        .tree-breadcrumb a { color: var(--bp-accent); text-decoration: none; }
        .tree-breadcrumb a:hover { opacity: 0.8; }
        .tree-breadcrumb-branch {
          color: var(--bp-dim); opacity: 0.5;
          margin-left: 4px; font-size: 11px;
        }

        .tree-content {
          max-width: 1280px; margin: 0 auto;
          padding: 24px 32px;
          display: grid;
          grid-template-columns: 1fr 280px;
          gap: 24px;
          align-items: start;
        }

        .tree-folder-card {
          border: 1px solid var(--bp-border);
          border-radius: 8px 8px 0 0;
          overflow: hidden;
          border-bottom: none;
        }
        .tree-folder-header {
          display: flex; align-items: center; justify-content: space-between;
          padding: 10px 16px;
          background: var(--bp-surface);
          border-bottom: 1px solid var(--bp-border);
          gap: 12px; flex-wrap: wrap;
        }
        .tree-folder-info {
          display: flex; align-items: center; gap: 8px;
        }
        .tree-folder-icon { font-size: 14px; }
        .tree-folder-name { color: var(--bp-heading); font-weight: 500; font-size: 13px; }
        .tree-folder-actions { display: flex; gap: 8px; }
        .tree-action-btn {
          padding: 4px 10px;
          border: 1px solid var(--bp-border);
          border-radius: 4px;
          color: var(--bp-dim); font-size: 11px;
          text-decoration: none;
          transition: all 0.12s;
        }
        .tree-action-btn:hover {
          border-color: rgba(79, 195, 247, 0.3);
          color: var(--bp-accent);
        }

        /* Connect folder header to file tree below */
        .tree-main .ft-root {
          border-top-left-radius: 0;
          border-top-right-radius: 0;
        }

        .tree-sidebar-card {
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          padding: 16px;
        }
        .tree-sidebar-label {
          font-size: 10px; font-weight: 600;
          text-transform: uppercase; letter-spacing: 0.1em;
          color: var(--bp-dim); margin-bottom: 12px;
        }
        .tree-sidebar-commit { margin-bottom: 10px; }
        .tree-sidebar-commit:last-child { margin-bottom: 0; }
        .tree-sidebar-commit-msg {
          color: var(--bp-text); text-decoration: none;
          font-size: 12px; line-height: 1.4; display: block;
        }
        .tree-sidebar-commit-msg:hover { color: var(--bp-accent); }
        .tree-sidebar-commit-meta {
          display: flex; gap: 8px; font-size: 10px;
          color: var(--bp-dim); opacity: 0.6; margin-top: 2px;
        }
        .tree-dim { color: var(--bp-dim); font-size: 12px; }

        @media (max-width: 900px) {
          .tree-content {
            grid-template-columns: 1fr;
            padding: 16px;
          }
          .tree-main { order: 1; }
          .tree-sidebar { order: 2; }
          .tree-header { padding: 12px 16px; }
        }
      `}</style>
    </div>
  );
}
