'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, formatBytes, getFileIcon, copyToClipboard } from '@/lib/utils';
import MarkdownRenderer from '@/components/markdown/MarkdownRenderer';
import BranchSelector from '@/components/BranchSelector';
import RepoStatsCards from '@/components/RepoStatsCards';

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

interface Branch {
  name: string;
  is_default: boolean;
  last_commit: {
    hash: string;
    timestamp: number;
    message: string;
  };
}

interface Contributor {
  address: string;
  pushCount: number;
  lastPush: string;
  isOwner: boolean;
}

export default function RepoPage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [currentFiles, setCurrentFiles] = useState<RepoDetails['file_tree']>([]);
  const [repoConfig, setRepoConfig] = useState<RepoConfig>({ exists: false, content: '' });
  const [contributors, setContributors] = useState<Contributor[]>([]);
  const [activeTab, setActiveTab] = useState<'readme' | 'files' | 'commits' | 'contributors' | 'config'>('readme');
  const [loading, setLoading] = useState(true);
  const [cloneCopied, setCloneCopied] = useState(false);
  const [addrCopied, setAddrCopied] = useState(false);
  const [selectedBranch, setSelectedBranch] = useState<string>('HEAD');
  const [branches, setBranches] = useState<Branch[]>([]);
  const [branchLoading, setBranchLoading] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;

  const cloneUrl = repo ? `https://git.repo.box/${repo.owner_address}/${repo.name}.git` : '';

  useEffect(() => {
    if (!address || !name) return;
    const fetchRepo = async () => {
      try {
        const branchParam = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
        const configBranchParam = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
        
        const [repoRes, commitsRes, branchesRes, contributorsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${address}/${name}${branchParam}`),
          fetch(`/api/explorer/repos/${address}/${name}/commits?limit=30${selectedBranch !== 'HEAD' ? `&branch=${selectedBranch}` : ''}`),
          fetch(`/api/explorer/repos/${address}/${name}/branches`),
          fetch(`/api/explorer/repos/${address}/${name}/contributors`)
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
        
        if (branchesRes.ok) {
          const branchData = await branchesRes.json();
          setBranches(branchData.branches || []);
          
          // Set initial branch if this is the first load
          if (selectedBranch === 'HEAD' && branchData.default_branch) {
            setSelectedBranch(branchData.default_branch);
          }
        }
        
        if (contributorsRes.ok) {
          const contributorsData = await contributorsRes.json();
          setContributors(contributorsData.contributors || []);
        }
        
        // Try to fetch .repobox/config.yml
        const configRes = await fetch(`/api/explorer/repos/${address}/${name}/blob/.repobox/config.yml${configBranchParam}`);
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
        setBranchLoading(false);
      }
    };
    fetchRepo();
  }, [address, name, selectedBranch]);

  const navigateToPath = async (path: string) => {
    if (!address || !name) return;
    const branchParam = selectedBranch !== repo?.default_branch ? `?branch=${selectedBranch}` : '';
    const res = await fetch(`/api/explorer/repos/${address}/${name}/tree/${path}${branchParam}`);
    if (res.ok) {
      const data = await res.json();
      setCurrentPath(path);
      setCurrentFiles(data.files || []);
      setFileContent(null);
    }
  };

  const viewFile = async (filePath: string) => {
    if (!address || !name) return;
    const branchParam = selectedBranch !== repo?.default_branch ? `?branch=${selectedBranch}` : '';
    const res = await fetch(`/api/explorer/repos/${address}/${name}/blob/${filePath}${branchParam}`);
    if (res.ok) {
      const data = await res.json();
      setFileContent(data);
      setActiveTab('files');
    }
  };

  const handleBranchChange = async (newBranch: string) => {
    setBranchLoading(true);
    setSelectedBranch(newBranch);
    
    // Reset current path and file content when switching branches
    setCurrentPath('');
    setFileContent(null);
    
    // Data will be refetched via useEffect dependency on selectedBranch
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
      {/* Header */}
      <header className="explore-main-header">
        <div className="explore-main-header-content">
          <div className="explore-nav">
            <Link href="/" className="explore-logo">
              repo<span className="explore-logo-dot">.</span>box
            </Link>
            <nav className="explore-nav-links">
              <Link href="/" className="explore-nav-link">Home</Link>
              <Link href="/explore" className="explore-nav-link">Explore</Link>
              <Link href="/docs" className="explore-nav-link">Docs</Link>
            </nav>
          </div>
          <div className="explore-breadcrumb-nav">
            <Link href="/explore" className="explore-breadcrumb-link">Explore</Link>
            <span className="explore-breadcrumb-separator">/</span>
            <Link href={`/explore/${repo.owner_address}`} className="explore-breadcrumb-link">
              {formatAddress(repo.owner_address)}
            </Link>
            <span className="explore-breadcrumb-separator">/</span>
            <span className="explore-breadcrumb-current">{repo.name}</span>
          </div>
        </div>
      </header>

      <div className="explore-main-content">
        {/* Repository Header */}
        <div className="explore-repo-detail-header">
          <div className="explore-repo-detail-info">
            <div className="explore-repo-detail-title-row">
              <svg className="explore-repo-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
              </svg>
              <h1 className="explore-repo-detail-title">{repo.name}</h1>
            </div>
            
            <div className="explore-repo-detail-owner">
              <Link href={`/explore/${repo.owner_address}`} className="explore-repo-detail-owner-link">
                <code>{formatAddress(repo.owner_address)}</code>
              </Link>
              <button 
                onClick={handleCopyAddr} 
                className="explore-repo-detail-copy-btn"
                title="Copy address"
              >
                {addrCopied ? (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <polyline points="20,6 9,17 4,12"></polyline>
                  </svg>
                ) : (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                  </svg>
                )}
              </button>
            </div>

            <div className="explore-repo-detail-clone">
              <div className="explore-clone-label">HTTPS</div>
              <div className="explore-clone-input-group">
                <input 
                  type="text" 
                  value={cloneUrl}
                  readOnly
                  className="explore-clone-input"
                />
                <button 
                  onClick={handleCopyClone} 
                  className="explore-clone-copy-btn"
                  title="Copy clone command"
                >
                  {cloneCopied ? (
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <polyline points="20,6 9,17 4,12"></polyline>
                    </svg>
                  ) : (
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                    </svg>
                  )}
                </button>
              </div>
            </div>
          </div>

          <div className="explore-repo-detail-stats">
            <div className="explore-repo-detail-stat">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M20 6L9 17l-5-5"></path>
              </svg>
              <span className="explore-repo-detail-stat-value">{repo.commit_count.toLocaleString()}</span>
              <span className="explore-repo-detail-stat-label">commits</span>
            </div>
            <div className="explore-repo-detail-stat">
              {branches.length > 0 ? (
                <BranchSelector
                  branches={branches}
                  currentBranch={selectedBranch}
                  defaultBranch={repo.default_branch}
                  onChange={handleBranchChange}
                  disabled={branchLoading}
                />
              ) : (
                <>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <line x1="6" y1="3" x2="6" y2="15"></line>
                    <circle cx="18" cy="6" r="3"></circle>
                    <circle cx="6" cy="18" r="3"></circle>
                    <path d="M18 9a9 9 0 0 1-9 9"></path>
                  </svg>
                  <span className="explore-repo-detail-stat-value">{repo.default_branch}</span>
                  <span className="explore-repo-detail-stat-label">branch</span>
                </>
              )}
            </div>
          </div>
        </div>

        {/* Repository Stats */}
        <RepoStatsCards 
          address={address} 
          name={name} 
          branch={selectedBranch}
        />

        {/* Tabs */}
        <div className="explore-repo-tabs-container">
          <nav className="explore-repo-tabs">
            {['readme', 'files', 'commits', 'contributors', 'config'].map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab as typeof activeTab)}
                className={`explore-repo-tab ${activeTab === tab ? 'active' : ''}`}
              >
                <span>{tab === 'readme' ? 'README' : 
                       tab === 'files' ? 'Files' : 
                       tab === 'commits' ? 'Commits' :
                       tab === 'contributors' ? 'Contributors' :
                       'Config'}</span>
              </button>
            ))}
          </nav>
        </div>

        {/* Tab Content */}
        <div className="explore-repo-tab-content">

        {/* README Tab (default) */}
        {activeTab === 'readme' && (
          <div className="explore-readme">
            {repo.readme_content ? (
              <div className="explore-readme-content">
                <MarkdownRenderer 
                  content={repo.readme_content}
                  baseUrl={`/api/explorer/repos/${address}/${name}/blob/`}
                  className="explore-readme-markdown"
                />
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

        {/* Contributors Tab */}
        {activeTab === 'contributors' && (
          <div className="explore-contributors">
            {contributors.length === 0 ? (
              <div className="explore-empty"><p>No contributors found</p></div>
            ) : (
              <div className="explore-contributors-grid">
                {contributors.map((contributor) => (
                  <Link
                    key={contributor.address}
                    href={`/explore/${contributor.address}`}
                    className="explore-contributor-card"
                  >
                    <div className="explore-contributor-header">
                      <code className="explore-contributor-address">
                        {formatAddress(contributor.address)}
                      </code>
                      {contributor.isOwner && (
                        <span className="explore-contributor-owner-badge">owner</span>
                      )}
                    </div>
                    <div className="explore-contributor-stats">
                      <div className="explore-contributor-stat">
                        <span className="explore-contributor-stat-value">{contributor.pushCount}</span>
                        <span className="explore-contributor-stat-label">pushes</span>
                      </div>
                      <div className="explore-contributor-stat">
                        <span className="explore-contributor-stat-value">
                          {formatTimeAgo(contributor.lastPush)}
                        </span>
                        <span className="explore-contributor-stat-label">last active</span>
                      </div>
                    </div>
                  </Link>
                ))}
              </div>
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
        </div>
      </div>
    </div>
  );
}
