'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, formatBytes, getFileIcon, copyToClipboard } from '@/lib/utils';
import { resolveNameToAddress } from '@/lib/addressResolver';
import { repoUrls, generateBreadcrumbs } from '@/lib/repoUrls';
import MarkdownRenderer from '@/components/markdown/MarkdownRenderer';
import BranchSelector from '@/components/BranchSelector';
import RepoStatsCards from '@/components/RepoStatsCards';
import CloneUrlWidget from '@/components/CloneUrlWidget';
import ExploreHeader from '@/components/explore/ExploreHeader';
import ContributionChart from '@/components/explore/ContributionChart';

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

type TabType = 'readme' | 'files' | 'commits' | 'contributors' | 'config';

export default function RepoPage() {
  const params = useParams();
  const router = useRouter();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [currentFiles, setCurrentFiles] = useState<RepoDetails['file_tree']>([]);
  const [repoConfig, setRepoConfig] = useState<RepoConfig>({ exists: false, content: '' });
  const [contributors, setContributors] = useState<Contributor[]>([]);
  const [activeTab, setActiveTab] = useState<TabType>('readme');
  const [loading, setLoading] = useState(true);
  const [addrCopied, setAddrCopied] = useState(false);
  const [selectedBranch, setSelectedBranch] = useState<string>('HEAD');
  const [branches, setBranches] = useState<Branch[]>([]);
  const [branchLoading, setBranchLoading] = useState(false);
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolving, setResolving] = useState(true);
  const [notFound, setNotFound] = useState(false);

  const addressOrName = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;

  useEffect(() => {
    const resolveAddress = async () => {
      if (!addressOrName) return;
      if (/^0x[a-fA-F0-9]{40}$/i.test(addressOrName)) {
        setResolvedAddress(addressOrName);
        setResolving(false);
        return;
      }
      try {
        const resolved = await resolveNameToAddress(addressOrName);
        if (resolved) {
          setResolvedAddress(resolved);
          router.replace(`/explore/${resolved}/${name}`);
        } else {
          setNotFound(true);
        }
      } catch (error) {
        console.error('Resolution failed:', error);
        setNotFound(true);
      } finally {
        setResolving(false);
      }
    };
    resolveAddress();
  }, [addressOrName, name, router]);

  useEffect(() => {
    if (!resolvedAddress || !name) return;
    const fetchRepo = async () => {
      try {
        const branchParam = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
        const configBranchParam = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';

        const [repoRes, commitsRes, branchesRes, contributorsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}${branchParam}`),
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}/commits?limit=30${selectedBranch !== 'HEAD' ? `&branch=${selectedBranch}` : ''}`),
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}/branches`),
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}/contributors`)
        ]);

        if (repoRes.ok) {
          const data = await repoRes.json();
          setRepo(data);
          setCurrentFiles(data.file_tree || []);
          if (!data.readme_content) setActiveTab('files');
        }

        if (commitsRes.ok) {
          const data = await commitsRes.json();
          setCommits(data.commits || []);
        }

        if (branchesRes.ok) {
          const branchData = await branchesRes.json();
          setBranches(branchData.branches || []);
          if (selectedBranch === 'HEAD' && branchData.default_branch) {
            setSelectedBranch(branchData.default_branch);
          }
        }

        if (contributorsRes.ok) {
          const contributorsData = await contributorsRes.json();
          setContributors(contributorsData.contributors || []);
        }

        const configRes = await fetch(`/api/explorer/repos/${resolvedAddress}/${name}/blob/.repobox/config.yml${configBranchParam}`);
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
  }, [resolvedAddress, name, selectedBranch]);

  const navigateToPath = (path: string) => {
    if (!resolvedAddress || !name) return;
    window.location.href = repoUrls.tree(resolvedAddress, name, selectedBranch || 'HEAD', path);
  };

  const viewFile = (filePath: string) => {
    if (!resolvedAddress || !name) return;
    window.location.href = repoUrls.blob(resolvedAddress, name, selectedBranch || 'HEAD', filePath);
  };

  const handleBranchChange = async (newBranch: string) => {
    setBranchLoading(true);
    setSelectedBranch(newBranch);
    setCurrentPath('');
    setFileContent(null);
  };

  const goBack = () => {
    if (fileContent) { setFileContent(null); return; }
    if (currentPath) {
      const parent = currentPath.split('/').slice(0, -1).join('/');
      if (parent) navigateToPath(parent);
      else { setCurrentPath(''); setCurrentFiles(repo?.file_tree || []); }
    }
  };

  const handleCopyAddr = async () => {
    if (!repo) return;
    await copyToClipboard(repo.owner_address);
    setAddrCopied(true);
    setTimeout(() => setAddrCopied(false), 2000);
  };

  if (!addressOrName || !name) return null;

  // Loading / error states
  if (resolving || loading) {
    return (
      <div className="rd-explore-page">
        <ExploreHeader />
        <div className="rd-detail-loading">
          <div className="rd-spinner" />
          <p>{resolving ? 'Resolving address...' : 'Loading repository...'}</p>
        </div>
      </div>
    );
  }

  if (notFound || !repo) {
    return (
      <div className="rd-explore-page">
        <ExploreHeader />
        <div className="rd-detail-empty">
          <h3>{notFound ? 'Address not found' : 'Repository not found'}</h3>
          {notFound && <p>Could not resolve &quot;{addressOrName}&quot; to an address.</p>}
          <Link href="/explore" className="rd-back-link">Back to Explorer</Link>
        </div>
      </div>
    );
  }

  const breadcrumbs = generateBreadcrumbs(
    resolvedAddress || '', name, selectedBranch, currentPath, !!fileContent
  );

  const tabConfigs = [
    { id: 'readme' as TabType, label: 'README', disabled: !repo.readme_content },
    { id: 'files' as TabType, label: 'Files', count: currentFiles.length },
    { id: 'commits' as TabType, label: 'Commits', count: repo.commit_count },
    { id: 'contributors' as TabType, label: 'Contributors', count: contributors.length },
    { id: 'config' as TabType, label: 'Config' },
  ];

  return (
    <div className="rd-explore-page">
      <ExploreHeader />

      {/* Breadcrumb */}
      <div className="rd-breadcrumb-bar">
        <div className="rd-breadcrumb-inner">
          <Link href="/explore" className="rd-breadcrumb-link">Explore</Link>
          <span className="rd-breadcrumb-sep">/</span>
          <Link href={`/explore/${repo.owner_address}`} className="rd-breadcrumb-link">
            {formatAddress(repo.owner_address)}
          </Link>
          <span className="rd-breadcrumb-sep">/</span>
          <span className="rd-breadcrumb-current">{repo.name}</span>
        </div>
      </div>

      <div className="rd-detail-layout">
        <main className="rd-detail-main">
          {/* Repo header */}
          <div className="rd-repo-header">
            <div className="rd-repo-header-top">
              <div className="rd-repo-identity">
                <svg className="rd-repo-icon" width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8Z"/>
                </svg>
                <h1 className="rd-repo-name">{repo.name}</h1>
              </div>
              <div className="rd-repo-badges">
                <span className="rd-badge rd-badge--commits">
                  {repo.commit_count.toLocaleString()} commits
                </span>
                {branches.length > 0 ? (
                  <BranchSelector
                    branches={branches}
                    currentBranch={selectedBranch}
                    defaultBranch={repo.default_branch}
                    onChange={handleBranchChange}
                    disabled={branchLoading}
                  />
                ) : (
                  <span className="rd-badge">
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor" style={{ marginRight: 4 }}>
                      <path d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.493 2.493 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z"/>
                    </svg>
                    {repo.default_branch}
                  </span>
                )}
              </div>
            </div>
            <div className="rd-repo-owner-row">
              <Link href={`/explore/${repo.owner_address}`} className="rd-repo-owner-link">
                <code>{formatAddress(repo.owner_address)}</code>
              </Link>
              <button onClick={handleCopyAddr} className="rd-copy-btn" title="Copy address">
                {addrCopied ? (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <polyline points="20,6 9,17 4,12" />
                  </svg>
                ) : (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                )}
              </button>
              <span className="rd-repo-created">
                Created {formatTimeAgo(repo.created_at)}
              </span>
            </div>
          </div>

          {/* Clone URL Widget */}
          <CloneUrlWidget
            ownerAddress={repo.owner_address}
            repoName={repo.name}
          />

          {/* Repository Stats */}
          <RepoStatsCards
            address={resolvedAddress || ''}
            name={name}
            branch={selectedBranch}
          />

          {/* Tabs */}
          <div className="rd-tabs-bar">
            <nav className="rd-tabs">
              {tabConfigs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  disabled={tab.disabled}
                  className={`rd-tab ${activeTab === tab.id ? 'rd-tab--active' : ''} ${tab.disabled ? 'rd-tab--disabled' : ''}`}
                >
                  {tab.label}
                  {tab.count !== undefined && (
                    <span className="rd-tab-count">{tab.count}</span>
                  )}
                </button>
              ))}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="rd-tab-content">
            {activeTab === 'readme' && (
              <div className="rd-readme">
                {repo.readme_content ? (
                  <div className="rd-readme-inner">
                    <MarkdownRenderer
                      content={repo.readme_content}
                      baseUrl={`/api/explorer/repos/${resolvedAddress}/${name}/blob/`}
                      className="rd-readme-md"
                    />
                  </div>
                ) : (
                  <div className="rd-detail-empty">
                    <p>No README found</p>
                    <p>Add a README.md file to help others understand your project.</p>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'files' && (
              <div>
                {(currentPath || fileContent) && (
                  <div className="rd-file-breadcrumb">
                    <button onClick={goBack} className="rd-file-back">
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M19 12H5m0 0l7 7m-7-7l7-7"/>
                      </svg>
                    </button>
                    <button
                      onClick={() => { setCurrentPath(''); setCurrentFiles(repo.file_tree || []); setFileContent(null); }}
                      className="rd-file-crumb"
                    >
                      {repo.name}
                    </button>
                    {breadcrumbs.slice(1).map((crumb, i) => (
                      <span key={i}>
                        <span className="rd-file-crumb-sep">/</span>
                        <button
                          onClick={() => {
                            const pathParts = currentPath.split('/');
                            const targetPath = pathParts.slice(0, i + 1).join('/');
                            navigateToPath(targetPath);
                            setFileContent(null);
                          }}
                          className="rd-file-crumb"
                        >
                          {crumb.label}
                        </button>
                      </span>
                    ))}
                    {fileContent && (
                      <span>
                        <span className="rd-file-crumb-sep">/</span>
                        <span className="rd-file-crumb-current">{fileContent.path.split('/').pop()}</span>
                      </span>
                    )}
                  </div>
                )}

                {fileContent ? (
                  <div className="rd-file-viewer">
                    <div className="rd-file-viewer-header">
                      <span>{getFileIcon(fileContent.path, false)}</span>
                      <span className="rd-file-viewer-name">{fileContent.path.split('/').pop()}</span>
                    </div>
                    <pre className="rd-file-viewer-code"><code>{fileContent.content}</code></pre>
                  </div>
                ) : (
                  <div className="rd-file-tree">
                    {currentFiles.length === 0 ? (
                      <div className="rd-detail-empty"><p>Empty directory</p></div>
                    ) : (
                      currentFiles.map((file, i) => (
                        <button
                          key={i}
                          onClick={() => file.type === 'tree' ? navigateToPath(file.path) : viewFile(file.path)}
                          className="rd-file-item"
                        >
                          <span className="rd-file-item-icon">{getFileIcon(file.name, file.type === 'tree')}</span>
                          <span className="rd-file-item-name">{file.name}</span>
                          {file.size !== undefined && (
                            <span className="rd-file-item-size">{formatBytes(file.size)}</span>
                          )}
                        </button>
                      ))
                    )}
                  </div>
                )}
              </div>
            )}

            {activeTab === 'commits' && (
              <div className="rd-commit-list">
                <div className="rd-commit-list-header">
                  <h3>Recent commits on {selectedBranch}</h3>
                  <Link
                    href={repoUrls.commits(resolvedAddress || '', name, selectedBranch)}
                    className="rd-view-all-link"
                  >
                    View all commits
                  </Link>
                </div>
                {commits.length === 0 ? (
                  <div className="rd-detail-empty"><p>No commits found</p></div>
                ) : (
                  commits.slice(0, 10).map((commit) => (
                    <div key={commit.hash} className="rd-commit-item">
                      <div className="rd-commit-msg">
                        <Link
                          href={repoUrls.commit(resolvedAddress || '', name, commit.hash)}
                          className="rd-commit-msg-link"
                        >
                          {commit.message}
                        </Link>
                      </div>
                      <div className="rd-commit-meta">
                        <code className="rd-commit-author">{formatAddress(repo.owner_address)}</code>
                        <span className="rd-commit-time">
                          {formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}
                        </span>
                        <Link
                          href={repoUrls.commit(resolvedAddress || '', name, commit.hash)}
                          className="rd-commit-hash"
                        >
                          <code>{commit.hash.slice(0, 7)}</code>
                        </Link>
                      </div>
                    </div>
                  ))
                )}
              </div>
            )}

            {activeTab === 'contributors' && (
              <div className="rd-contributors">
                {contributors.length === 0 ? (
                  <div className="rd-detail-empty"><p>No contributors found</p></div>
                ) : (
                  <>
                    <div className="rd-contributors-grid">
                      {contributors.map((contributor) => (
                        <Link
                          key={contributor.address}
                          href={`/explore/${contributor.address}`}
                          className="rd-contributor-card"
                        >
                          <div className="rd-contributor-header">
                            <code className="rd-contributor-addr">
                              {formatAddress(contributor.address)}
                            </code>
                            {contributor.isOwner && (
                              <span className="rd-badge rd-badge--owner">owner</span>
                            )}
                          </div>
                          <div className="rd-contributor-stats">
                            <div>
                              <span className="rd-contributor-val">{contributor.pushCount}</span>
                              <span className="rd-contributor-label">pushes</span>
                            </div>
                            <div>
                              <span className="rd-contributor-val">{formatTimeAgo(contributor.lastPush)}</span>
                              <span className="rd-contributor-label">last active</span>
                            </div>
                          </div>
                        </Link>
                      ))}
                    </div>
                    <ContributionChart
                      contributors={contributors}
                      address={resolvedAddress || ''}
                      name={name}
                      branch={selectedBranch}
                    />
                  </>
                )}
              </div>
            )}

            {activeTab === 'config' && (
              <div className="rd-config">
                <div className="rd-config-section">
                  <h3 className="rd-config-heading">Owner Identity</h3>
                  <div className="rd-config-identity">
                    <code>{repo.owner_address}</code>
                    <span className="rd-badge rd-badge--owner">owner</span>
                  </div>
                </div>
                <div className="rd-config-section">
                  <h3 className="rd-config-heading">Permission Ruleset</h3>
                  {repoConfig.exists ? (
                    <pre className="rd-code-block"><code>{repoConfig.content}</code></pre>
                  ) : (
                    <div className="rd-config-empty">
                      <p>No <code>.repobox/config.yml</code> found in this repository.</p>
                      <p className="rd-config-hint">
                        Run <code>git repobox init</code> to create a permission config.
                      </p>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}
