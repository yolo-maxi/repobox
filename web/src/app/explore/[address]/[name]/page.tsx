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
    if (fileContent) { 
      setFileContent(null); 
      return; 
    }
    if (currentPath) {
      const parent = currentPath.split('/').slice(0, -1).join('/');
      if (parent) {
        navigateToPath(parent);
      } else { 
        setCurrentPath(''); 
        setCurrentFiles(repo?.file_tree || []); 
      }
    }
  };

  const handleCopyAddress = async () => {
    if (!repo) return;
    await copyToClipboard(repo.owner_address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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
          <Link href="/explore" className="explore-back-link">
            ← Back to Explorer
          </Link>
        </div>
      </div>
    );
  }

  const pathParts = currentPath ? currentPath.split('/') : [];

  return (
    <div className="explore-page">
      {/* Back Link */}
      <div className="explore-back">
        <Link href="/explore" className="explore-back-link">
          ← Back to Explorer
        </Link>
      </div>

      {/* Repository Header */}
      <header className="explore-repo-header">
        <div className="explore-repo-info">
          <h1 className="explore-repo-title">{repo.name}</h1>
          <div className="explore-repo-owner-info">
            <span className="explore-repo-owner">{repo.owner_address}</span>
            <button onClick={handleCopyAddress} className="explore-copy-btn">
              {copied ? '✓' : 'Copy'}
            </button>
          </div>
          <div className="explore-clone-url">
            <code>git clone https://git.repo.box/{formatAddress(repo.owner_address)}/{repo.name}.git</code>
          </div>
        </div>
        
        <div className="explore-repo-stats">
          <div className="explore-stat-item">
            <span className="explore-stat-value">{repo.commit_count}</span>
            <span className="explore-stat-label">commits</span>
          </div>
          <div className="explore-stat-item">
            <span className="explore-stat-value">{repo.default_branch}</span>
            <span className="explore-stat-label">branch</span>
          </div>
        </div>
      </header>

      {/* Tabs */}
      <nav className="explore-tabs">
        <button
          onClick={() => setActiveTab('files')}
          className={`explore-tab ${activeTab === 'files' ? 'active' : ''}`}
        >
          Files
        </button>
        <button
          onClick={() => setActiveTab('commits')}
          className={`explore-tab ${activeTab === 'commits' ? 'active' : ''}`}
        >
          Commits
        </button>
        <button
          onClick={() => setActiveTab('readme')}
          className={`explore-tab ${activeTab === 'readme' ? 'active' : ''}`}
        >
          README
        </button>
      </nav>

      {/* Tab Content */}
      <section className="explore-tab-content">
        {/* Files Tab */}
        {activeTab === 'files' && (
          <div>
            {/* Breadcrumb */}
            {(currentPath || fileContent) && (
              <div className="explore-breadcrumb">
                <button onClick={goBack} className="explore-breadcrumb-back">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                  </svg>
                </button>
                <button
                  onClick={() => { 
                    setCurrentPath(''); 
                    setCurrentFiles(repo.file_tree || []); 
                    setFileContent(null); 
                  }}
                  className="explore-breadcrumb-item"
                >
                  {repo.name}
                </button>
                {pathParts.map((seg, i) => (
                  <div key={i} className="explore-breadcrumb-separator">
                    <span>/</span>
                    <button
                      onClick={() => { 
                        navigateToPath(pathParts.slice(0, i + 1).join('/')); 
                        setFileContent(null); 
                      }}
                      className="explore-breadcrumb-item"
                    >
                      {seg}
                    </button>
                  </div>
                ))}
                {fileContent && (
                  <div className="explore-breadcrumb-separator">
                    <span>/</span>
                    <span className="explore-breadcrumb-file">
                      {fileContent.path.split('/').pop()}
                    </span>
                  </div>
                )}
              </div>
            )}

            {fileContent ? (
              <div className="explore-file-viewer">
                <div className="explore-file-header">
                  <span className="explore-file-icon">
                    {getFileIcon(fileContent.path, false)}
                  </span>
                  <span className="explore-file-name">
                    {fileContent.path.split('/').pop()}
                  </span>
                </div>
                <pre className="explore-file-content">
                  <code>{fileContent.content}</code>
                </pre>
              </div>
            ) : (
              <div className="explore-file-list">
                {currentFiles.length === 0 ? (
                  <div className="explore-empty">
                    <p>Empty directory</p>
                  </div>
                ) : (
                  currentFiles.map((file, i) => (
                    <button
                      key={i}
                      onClick={() => 
                        file.type === 'tree' ? navigateToPath(file.path) : viewFile(file.path)
                      }
                      className="explore-file-item"
                    >
                      <div className="explore-file-info">
                        <span className="explore-file-icon">
                          {getFileIcon(file.name, file.type === 'tree')}
                        </span>
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

        {/* Commits Tab */}
        {activeTab === 'commits' && (
          <div className="explore-commit-list">
            {commits.length === 0 ? (
              <div className="explore-empty">
                <p>No commits found</p>
              </div>
            ) : (
              commits.map((commit) => (
                <div key={commit.hash} className="explore-commit-item">
                  <div className="explore-commit-message">{commit.message}</div>
                  <div className="explore-commit-meta">
                    <span className="explore-commit-author">{commit.author}</span>
                    <span className="explore-commit-time">
                      {formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}
                    </span>
                    <span className="explore-commit-hash">{commit.hash.slice(0, 7)}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {/* README Tab */}
        {activeTab === 'readme' && (
          <div className="explore-readme">
            {repo.readme_content ? (
              <div 
                className="explore-readme-content"
                dangerouslySetInnerHTML={{ __html: repo.readme_content }}
              />
            ) : (
              <div className="explore-empty">
                <p>No README found</p>
              </div>
            )}
          </div>
        )}
      </section>
    </div>
  );
}