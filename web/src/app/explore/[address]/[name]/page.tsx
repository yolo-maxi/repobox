'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, formatBytes, getFileIcon, copyToClipboard } from '@/lib/utils';

interface RepoDetails {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
  commit_count: number;
  default_branch: string;
  file_tree: Array<{
    type: 'blob' | 'tree';
    name: string;
    size?: number;
    path: string;
  }>;
  readme_content: string | null;
}

interface Commit {
  hash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
}

interface FileContent {
  path: string;
  content: string;
}

export default function RepoPage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [currentFiles, setCurrentFiles] = useState<RepoDetails['file_tree']>([]);
  const [activeTab, setActiveTab] = useState<'files' | 'commits' | 'readme'>('files');
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;

  useEffect(() => {
    if (!address || !name) return;
    const fetchRepo = async () => {
      try {
        const [repoRes, commitsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${address}/${name}`),
          fetch(`/api/explorer/repos/${address}/${name}/commits?limit=30`)
        ]);
        if (repoRes.ok) {
          const data = await repoRes.json();
          setRepo(data);
          setCurrentFiles(data.file_tree || []);
        }
        if (commitsRes.ok) {
          const data = await commitsRes.json();
          setCommits(data.commits || []);
        }
      } catch (error) {
        console.error('Error:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchRepo();
  }, [address, name]);

  const navigateToPath = async (path: string) => {
    const res = await fetch(`/api/explorer/repos/${address}/${name}/tree/${path}`);
    if (res.ok) {
      const data = await res.json();
      setCurrentPath(path);
      setCurrentFiles(data.files || []);
      setFileContent(null);
    }
  };

  const viewFile = async (filePath: string) => {
    const res = await fetch(`/api/explorer/repos/${address}/${name}/blob/${filePath}`);
    if (res.ok) {
      const data = await res.json();
      setFileContent(data);
      setActiveTab('files');
    }
  };

  const goBack = () => {
    if (fileContent) { setFileContent(null); return; }
    if (currentPath) {
      const parent = currentPath.split('/').slice(0, -1).join('/');
      if (parent) navigateToPath(parent);
      else { setCurrentPath(''); setCurrentFiles(repo?.file_tree || []); }
    }
  };

  if (!address || !name) return null;

  if (loading) {
    return (
      <div className="p-10 text-center">
        <div className="animate-pulse text-[var(--bp-accent)] text-sm">Loading...</div>
      </div>
    );
  }

  if (!repo) {
    return (
      <div className="p-10 text-center">
        <div className="text-[var(--bp-dim)] text-lg mb-3">Repository not found</div>
        <Link href="/explore" className="text-[var(--bp-accent)] text-sm hover:opacity-80">
          ← Back to explorer
        </Link>
      </div>
    );
  }

  const pathParts = currentPath ? currentPath.split('/') : [];

  return (
    <div className="p-6 md:p-10">
      {/* Back + Header */}
      <Link href="/explore" className="text-[var(--bp-accent)] text-sm hover:opacity-80 mb-6 inline-block">
        ← Explorer
      </Link>

      <div className="flex items-start justify-between mb-8 flex-wrap gap-4">
        <div>
          <h1 className="text-2xl font-bold text-[var(--bp-heading)] mb-2">
            {repo.name}
          </h1>
          <div className="flex items-center gap-3">
            <span className="text-[var(--bp-dim)] text-sm font-mono">
              {repo.owner_address}
            </span>
            <button
              onClick={async () => {
                await copyToClipboard(repo.owner_address);
                setCopied(true);
                setTimeout(() => setCopied(false), 2000);
              }}
              className="glass-tab text-xs px-2 py-1 rounded"
            >
              {copied ? '✓ Copied' : 'Copy'}
            </button>
          </div>
          <div className="mt-2 text-xs text-[var(--bp-dim)] font-mono">
            git clone https://git.repo.box/{formatAddress(repo.owner_address)}/{repo.name}.git
          </div>
        </div>
        
        <div className="flex gap-4 text-sm text-[var(--bp-dim)]">
          <div className="glass-stat rounded-lg px-4 py-2 text-center">
            <div className="text-lg font-bold text-[var(--bp-accent)] font-mono">{repo.commit_count}</div>
            <div className="text-xs">commits</div>
          </div>
          <div className="glass-stat rounded-lg px-4 py-2 text-center">
            <div className="text-lg font-bold text-[var(--bp-heading)] font-mono">{repo.default_branch}</div>
            <div className="text-xs">branch</div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-6">
        {(['files', 'commits', 'readme'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`glass-tab px-4 py-2 text-sm rounded-lg capitalize ${
              activeTab === tab ? 'glass-tab-active' : ''
            }`}
          >
            {tab === 'files' ? '📁 Files' : tab === 'commits' ? '📝 Commits' : '📖 README'}
          </button>
        ))}
      </div>

      {/* Files Tab */}
      {activeTab === 'files' && (
        <div>
          {/* Breadcrumb */}
          {(currentPath || fileContent) && (
            <div className="flex items-center gap-2 mb-4 text-sm flex-wrap">
              <button onClick={goBack} className="text-[var(--bp-accent)] hover:opacity-80">←</button>
              <button
                onClick={() => { setCurrentPath(''); setCurrentFiles(repo.file_tree || []); setFileContent(null); }}
                className="text-[var(--bp-accent)] hover:opacity-80 font-mono"
              >
                {repo.name}
              </button>
              {pathParts.map((seg, i) => (
                <span key={i} className="flex items-center gap-2">
                  <span className="text-[var(--bp-dim)]">/</span>
                  <button
                    onClick={() => { navigateToPath(pathParts.slice(0, i + 1).join('/')); setFileContent(null); }}
                    className="text-[var(--bp-accent)] hover:opacity-80 font-mono"
                  >
                    {seg}
                  </button>
                </span>
              ))}
              {fileContent && (
                <span className="flex items-center gap-2">
                  <span className="text-[var(--bp-dim)]">/</span>
                  <span className="text-[var(--bp-text)] font-mono">{fileContent.path.split('/').pop()}</span>
                </span>
              )}
            </div>
          )}

          {fileContent ? (
            <div className="glass-panel-inner rounded-xl overflow-hidden">
              <div className="px-4 py-2.5 border-b border-[rgba(50,100,160,0.15)] flex items-center gap-2">
                <span className="font-mono text-sm text-[var(--bp-dim)]">
                  {getFileIcon(fileContent.path, false)} {fileContent.path.split('/').pop()}
                </span>
              </div>
              <pre className="p-4 overflow-auto max-h-[600px] text-sm font-mono text-[var(--bp-text)] leading-relaxed">
                <code>{fileContent.content}</code>
              </pre>
            </div>
          ) : (
            <div className="glass-panel-inner rounded-xl overflow-hidden">
              {currentFiles.length === 0 ? (
                <div className="p-8 text-center text-[var(--bp-dim)]">Empty</div>
              ) : (
                currentFiles.map((file, i) => (
                  <div
                    key={i}
                    onClick={() => file.type === 'tree' ? navigateToPath(file.path) : viewFile(file.path)}
                    className={`flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-[rgba(79,195,247,0.04)] transition-colors ${
                      i > 0 ? 'border-t border-[rgba(50,100,160,0.1)]' : ''
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-base">{getFileIcon(file.name, file.type === 'tree')}</span>
                      <span className="text-[var(--bp-text)] text-sm">{file.name}</span>
                    </div>
                    {file.size !== undefined && (
                      <span className="text-xs text-[var(--bp-dim)] font-mono">{formatBytes(file.size)}</span>
                    )}
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      )}

      {/* Commits Tab */}
      {activeTab === 'commits' && (
        <div className="space-y-2">
          {commits.length === 0 ? (
            <div className="glass-panel-inner rounded-xl p-8 text-center text-[var(--bp-dim)]">No commits</div>
          ) : (
            commits.map((c) => (
              <div key={c.hash} className="glass-card rounded-lg p-4">
                <div className="text-sm text-[var(--bp-text)] mb-2">{c.message}</div>
                <div className="flex items-center gap-4 text-xs text-[var(--bp-dim)] font-mono">
                  <span>{c.author}</span>
                  <span>{formatTimeAgo(new Date(c.timestamp * 1000).toISOString())}</span>
                  <span className="text-[var(--bp-accent)]">{c.hash.slice(0, 7)}</span>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* README Tab */}
      {activeTab === 'readme' && (
        <div className="glass-panel-inner rounded-xl p-6">
          {repo.readme_content ? (
            <div className="prose prose-invert prose-sm max-w-none text-[var(--bp-text)]"
              dangerouslySetInnerHTML={{ __html: repo.readme_content }}
            />
          ) : (
            <div className="text-center text-[var(--bp-dim)] py-8">No README found</div>
          )}
        </div>
      )}
    </div>
  );
}
