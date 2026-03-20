'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
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

interface RepoConfig {
  exists: boolean;
  content: string;
}

export default function RepoPage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [currentFiles, setCurrentFiles] = useState<RepoDetails['file_tree']>([]);
  const [repoConfig, setRepoConfig] = useState<RepoConfig>({ exists: false, content: '' });
  const [activeTab, setActiveTab] = useState<'readme' | 'files' | 'commits' | 'config'>('readme');
  const [loading, setLoading] = useState(true);
  const [cloneCopied, setCloneCopied] = useState(false);
  const [addrCopied, setAddrCopied] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;

  const cloneUrl = repo ? `https://git.repo.box/${repo.owner_address}/${repo.name}.git` : '';

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
          // If no README, default to files tab
          if (!data.readme_content) setActiveTab('files');
        }
        if (commitsRes.ok) {
          const data = await commitsRes.json();
          setCommits(data.commits || []);
        }
        // Try to fetch .repobox/config.yml
        const configRes = await fetch(`/api/explorer/repos/${address}/${name}/blob/.repobox/config.yml`);
        if (configRes.ok) {
          const data = await configRes.json();
          if (data.content) {
            setRepoConfig({ exists: true, content: data.content });
          }
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
    if (!address || !name) return;
    const res = await fetch(`/api/explorer/repos/${address}/${name}/tree/${path}`);
    if (res.ok) {
      const data = await res.json();
      setCurrentPath(path);
      setCurrentFiles(data.files || []);
      setFileContent(null);
    }
  };

  const viewFile = async (filePath: string) => {
    if (!address || !name) return;
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

  const handleCopyClone = async () => {
    await copyToClipboard(`git clone ${cloneUrl}`);
    setCloneCopied(true);
    setTimeout(() => setCloneCopied(false), 2000);
  };

  const handleCopyAddr = async () => {
    if (!repo) return;
    await copyToClipboard(repo.owner_address);
    setAddrCopied(true);
    setTimeout(() => setAddrCopied(false), 2000);
  };

  if (!address || !name) return null;

  if (loading) {
    return (
      <div className="explore-page">
        <div className="explore-loading">
          <div className="explore-loading-spinner"></div>
          <p>Loading repository...</p>
        </div>
      </div>
    );
  }

  if (!repo) {
    return (
      <div className="explore-page">
        <div className="explore-empty">
          <h3>Repository not found</h3>
          <Link href="/explore" className="explore-back-link">← Back to Explorer</Link>
        </div>
      </div>
    );
  }

  const pathParts = currentPath ? currentPath.split('/') : [];

  return (
    <div className="explore-page">
      {/* Back Link */}
      <div className="explore-back">
        <Link href="/explore" className="explore-back-link">← Back to Explorer</Link>
      </div>

      {/* Repository Header */}
      <header className="explore-repo-header">
        <div className="explore-repo-info">
          <h1 className="explore-repo-title">{repo.name}</h1>
          <div className="explore-repo-owner-info">
            <button 
              onClick={handleCopyAddr} 
              className="explore-repo-owner"
              title="Click to copy address"
            >
              <code>{repo.owner_address}</code>
              <span className="explore-copy-hint">{addrCopied ? '✓ copied' : 'copy'}</span>
            </button>
          </div>
          {/* Clone URL — full address, clickable to copy */}
          <button onClick={handleCopyClone} className="explore-clone-url" title="Click to copy clone command">
            <code style={{ fontSize: '12px', wordBreak: 'break-all' }}>
              git clone {cloneUrl}
            </code>
            <span className="explore-copy-hint">{cloneCopied ? '✓ copied' : 'click to copy'}</span>
          </button>
        </div>

        <div className="explore-repo-stats">
          <div className="explore-stat-item">
            <span className="explore-stat-value">{repo.commit_count}</span>
            <span className="explore-stat-label">COMMITS</span>
          </div>
          <div className="explore-stat-item">
            <span className="explore-stat-value">{repo.default_branch}</span>
            <span className="explore-stat-label">BRANCH</span>
          </div>
        </div>
      </header>

      {/* Tabs — README first */}
      <nav className="explore-tabs">
        {['readme', 'files', 'commits', 'config'].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab as typeof activeTab)}
            className={`explore-tab ${activeTab === tab ? 'active' : ''}`}
          >
            {tab === 'readme' ? 'README' : 
             tab === 'files' ? 'Files' : 
             tab === 'commits' ? 'Commits' :
             'Config'}
          </button>
        ))}
      </nav>

      {/* Tab Content */}
      <section className="explore-tab-content">

        {/* README Tab (default) */}
        {activeTab === 'readme' && (
          <div className="explore-readme">
            {repo.readme_content ? (
              <div className="explore-readme-content prose">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={{
                    // Render code blocks with syntax highlighting class
                    code: ({ className, children, ...props }) => {
                      const isInline = !className;
                      if (isInline) {
                        return <code className="explore-inline-code" {...props}>{children}</code>;
                      }
                      return (
                        <pre className="explore-code-block">
                          <code className={className} {...props}>{children}</code>
                        </pre>
                      );
                    },
                    pre: ({ children }) => <>{children}</>,
                    // Tables
                    table: ({ children }) => (
                      <div className="explore-table-wrap">
                        <table className="explore-table">{children}</table>
                      </div>
                    ),
                    // Links
                    a: ({ children, href }) => (
                      <a href={href} target="_blank" rel="noopener noreferrer" className="explore-link">
                        {children}
                      </a>
                    ),
                  }}
                >
                  {repo.readme_content}
                </ReactMarkdown>
              </div>
            ) : (
              <div className="explore-empty">
                <p>No README found</p>
              </div>
            )}
          </div>
        )}

        {/* Files Tab */}
        {activeTab === 'files' && (
          <div>
            {(currentPath || fileContent) && (
              <div className="explore-breadcrumb">
                <button onClick={goBack} className="explore-breadcrumb-back">←</button>
                <button
                  onClick={() => { setCurrentPath(''); setCurrentFiles(repo.file_tree || []); setFileContent(null); }}
                  className="explore-breadcrumb-item"
                >
                  {repo.name}
                </button>
                {pathParts.map((seg, i) => (
                  <span key={i} className="explore-breadcrumb-separator">
                    <span>/</span>
                    <button
                      onClick={() => { navigateToPath(pathParts.slice(0, i + 1).join('/')); setFileContent(null); }}
                      className="explore-breadcrumb-item"
                    >
                      {seg}
                    </button>
                  </span>
                ))}
                {fileContent && (
                  <span className="explore-breadcrumb-separator">
                    <span>/</span>
                    <span className="explore-breadcrumb-file">{fileContent.path.split('/').pop()}</span>
                  </span>
                )}
              </div>
            )}

            {fileContent ? (
              <div className="explore-file-viewer">
                <div className="explore-file-header">
                  <span className="explore-file-icon">{getFileIcon(fileContent.path, false)}</span>
                  <span className="explore-file-name">{fileContent.path.split('/').pop()}</span>
                </div>
                <pre className="explore-file-content"><code>{fileContent.content}</code></pre>
              </div>
            ) : (
              <div className="explore-file-list">
                {currentFiles.length === 0 ? (
                  <div className="explore-empty"><p>Empty directory</p></div>
                ) : (
                  currentFiles.map((file, i) => (
                    <button
                      key={i}
                      onClick={() => file.type === 'tree' ? navigateToPath(file.path) : viewFile(file.path)}
                      className="explore-file-item"
                    >
                      <div className="explore-file-info">
                        <span className="explore-file-icon">{getFileIcon(file.name, file.type === 'tree')}</span>
                        <span className="explore-file-name">{file.name}</span>
                      </div>
                      {file.size !== undefined && (
                        <span className="explore-file-size">{formatBytes(file.size)}</span>
                      )}
                    </button>
                  ))
                )}
              </div>
            )}
          </div>
        )}

        {/* Commits Tab — show address, not author name */}
        {activeTab === 'commits' && (
          <div className="explore-commit-list">
            {commits.length === 0 ? (
              <div className="explore-empty"><p>No commits found</p></div>
            ) : (
              commits.map((commit) => (
                <div key={commit.hash} className="explore-commit-item">
                  <div className="explore-commit-message">{commit.message}</div>
                  <div className="explore-commit-meta">
                    <code className="explore-commit-author">{formatAddress(repo.owner_address)}</code>
                    <span className="explore-commit-time">
                      {formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}
                    </span>
                    <code className="explore-commit-hash">{commit.hash.slice(0, 7)}</code>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {/* Config Tab — show .repobox/config.yml + owner identity */}
        {activeTab === 'config' && (
          <div className="explore-config">
            <div className="explore-config-section">
              <h3 className="explore-config-heading">Owner Identity</h3>
              <div className="explore-config-identity">
                <code>{repo.owner_address}</code>
                <span className="explore-config-badge">owner</span>
              </div>
            </div>

            <div className="explore-config-section">
              <h3 className="explore-config-heading">Permission Ruleset</h3>
              {repoConfig.exists ? (
                <pre className="explore-code-block">
                  <code>{repoConfig.content}</code>
                </pre>
              ) : (
                <div className="explore-config-empty">
                  <p>No <code>.repobox/config.yml</code> found in this repository.</p>
                  <p className="explore-config-hint">
                    Run <code>git repobox init</code> to create a permission config.
                  </p>
                </div>
              )}
            </div>
          </div>
        )}
      </section>
    </div>
  );
}
