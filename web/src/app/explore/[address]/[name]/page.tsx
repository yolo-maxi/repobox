'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import ReactMarkdown from 'react-markdown';
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
  const router = useRouter();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [currentPath, setCurrentPath] = useState<string>('');
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [currentFiles, setCurrentFiles] = useState<RepoDetails['file_tree']>([]);
  const [activeTab, setActiveTab] = useState<'files' | 'commits' | 'readme'>('files');
  const [loading, setLoading] = useState(true);
  const [ensName, setEnsName] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;

  if (!address || !name) {
    return (
      <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)] p-8">
        <div className="max-w-6xl mx-auto">
          <div className="text-center py-20">
            <div className="text-[var(--bp-dim)]">Invalid repository</div>
            <Link href="/explore" className="text-[var(--bp-accent)] hover:opacity-80 text-sm mt-4 inline-block">
              ← Back to explorer
            </Link>
          </div>
        </div>
      </div>
    );
  }

  useEffect(() => {

    const fetchRepo = async () => {
      try {
        const [repoRes, commitsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${address}/${name}`),
          fetch(`/api/explorer/repos/${address}/${name}/commits?limit=20`)
        ]);

        if (repoRes.ok) {
          const repoData = await repoRes.json();
          setRepo(repoData);
          setCurrentFiles(repoData.file_tree || []);
        }

        if (commitsRes.ok) {
          const commitsData = await commitsRes.json();
          setCommits(commitsData.commits || []);
        }

        // Try to resolve ENS name (simplified - would need better ENS resolution in real app)
        if (address.length === 42 && address.startsWith('0x')) {
          // This is a placeholder - real ENS reverse resolution would be more complex
          // For now, we'll just show the address
        }
      } catch (error) {
        console.error('Error fetching repo:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchRepo();
  }, [address, name]);

  const navigateToPath = async (path: string) => {
    try {
      const res = await fetch(`/api/explorer/repos/${address}/${name}/tree/${path}`);
      if (res.ok) {
        const data = await res.json();
        setCurrentPath(path);
        setCurrentFiles(data.files || []);
        setFileContent(null);
      }
    } catch (error) {
      console.error('Error navigating to path:', error);
    }
  };

  const viewFile = async (filePath: string) => {
    try {
      const res = await fetch(`/api/explorer/repos/${address}/${name}/blob/${filePath}`);
      if (res.ok) {
        const data = await res.json();
        setFileContent(data);
        setActiveTab('files');
      }
    } catch (error) {
      console.error('Error viewing file:', error);
    }
  };

  const goBack = () => {
    if (fileContent) {
      setFileContent(null);
      return;
    }
    
    if (currentPath) {
      const parentPath = currentPath.split('/').slice(0, -1).join('/');
      if (parentPath) {
        navigateToPath(parentPath);
      } else {
        setCurrentPath('');
        setCurrentFiles(repo?.file_tree || []);
      }
    }
  };

  const handleCopyAddress = async () => {
    await copyToClipboard(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)] p-8">
        <div className="max-w-6xl mx-auto">
          <div className="text-center py-20">
            <div className="animate-pulse text-[var(--bp-accent)]">Loading...</div>
          </div>
        </div>
      </div>
    );
  }

  if (!repo) {
    return (
      <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)] p-8">
        <div className="max-w-6xl mx-auto">
          <div className="text-center py-20">
            <div className="text-[var(--bp-dim)]">Repository not found</div>
            <Link href="/explore" className="text-[var(--bp-accent)] hover:opacity-80 text-sm mt-4 inline-block">
              ← Back to explorer
            </Link>
          </div>
        </div>
      </div>
    );
  }

  const pathBreadcrumbs = currentPath ? currentPath.split('/') : [];

  return (
    <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)]">
      {/* Header */}
      <header className="border-b border-[var(--bp-border)] bg-[var(--bp-surface)]">
        <div className="max-w-6xl mx-auto px-8 py-6">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center space-x-4">
              <Link href="/explore" className="text-[var(--bp-accent)] hover:opacity-80">
                ← Explorer
              </Link>
              <div>
                <h1 className="text-2xl font-bold text-[var(--bp-heading)]">
                  {repo.name}
                </h1>
                <div className="flex items-center space-x-2 mt-1">
                  <span className="text-[var(--bp-dim)] text-sm font-mono">
                    {formatAddress(repo.owner_address)}
                  </span>
                  <button
                    onClick={handleCopyAddress}
                    className="text-xs text-[var(--bp-dim)] hover:text-[var(--bp-accent)] border border-[var(--bp-border)] px-2 py-1 rounded"
                  >
                    {copied ? 'Copied!' : 'Copy'}
                  </button>
                  {ensName && (
                    <span className="text-[var(--bp-accent)] text-sm">
                      {ensName}
                    </span>
                  )}
                </div>
              </div>
            </div>
            
            <div className="text-right text-sm text-[var(--bp-dim)]">
              <div>{repo.commit_count} commits</div>
              <div>{repo.default_branch} branch</div>
            </div>
          </div>

          {/* Tabs */}
          <div className="flex space-x-6">
            {['files', 'commits', 'readme'].map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab as any)}
                className={`pb-2 border-b-2 text-sm font-medium ${
                  activeTab === tab
                    ? 'border-[var(--bp-accent)] text-[var(--bp-accent)]'
                    : 'border-transparent text-[var(--bp-dim)] hover:text-[var(--bp-text)]'
                }`}
              >
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
              </button>
            ))}
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-8 py-8">
        {activeTab === 'files' && (
          <div>
            {/* Breadcrumb */}
            {(currentPath || fileContent) && (
              <div className="flex items-center space-x-2 mb-4 text-sm">
                <button
                  onClick={goBack}
                  className="text-[var(--bp-accent)] hover:opacity-80"
                >
                  ←
                </button>
                <Link
                  href={`/explore/${address}/${name}`}
                  onClick={(e) => {
                    e.preventDefault();
                    setCurrentPath('');
                    setCurrentFiles(repo.file_tree || []);
                    setFileContent(null);
                  }}
                  className="text-[var(--bp-accent)] hover:opacity-80"
                >
                  {repo.name}
                </Link>
                {pathBreadcrumbs.map((segment, index) => (
                  <div key={index} className="flex items-center space-x-2">
                    <span className="text-[var(--bp-dim)]">/</span>
                    <button
                      onClick={() => {
                        const path = pathBreadcrumbs.slice(0, index + 1).join('/');
                        navigateToPath(path);
                        setFileContent(null);
                      }}
                      className="text-[var(--bp-accent)] hover:opacity-80"
                    >
                      {segment}
                    </button>
                  </div>
                ))}
                {fileContent && (
                  <>
                    <span className="text-[var(--bp-dim)]">/</span>
                    <span className="text-[var(--bp-text)]">
                      {fileContent.path.split('/').pop()}
                    </span>
                  </>
                )}
              </div>
            )}

            {fileContent ? (
              /* File Content */
              <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg overflow-hidden">
                <div className="border-b border-[var(--bp-border)] px-4 py-2 bg-[var(--bp-bg)]">
                  <div className="flex items-center justify-between">
                    <span className="font-mono text-sm">
                      {getFileIcon(fileContent.path, false)} {fileContent.path.split('/').pop()}
                    </span>
                  </div>
                </div>
                <pre className="p-4 overflow-auto max-h-[600px] text-sm font-mono">
                  <code>{fileContent.content}</code>
                </pre>
              </div>
            ) : (
              /* File Tree */
              <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg overflow-hidden">
                {currentFiles.length === 0 ? (
                  <div className="p-8 text-center text-[var(--bp-dim)]">
                    Empty repository or directory
                  </div>
                ) : (
                  <div>
                    {currentFiles.map((file, index) => (
                      <div
                        key={index}
                        className={`flex items-center justify-between px-4 py-3 hover:bg-[var(--bp-bg)] cursor-pointer ${
                          index > 0 ? 'border-t border-[var(--bp-border)]' : ''
                        }`}
                        onClick={() => {
                          if (file.type === 'tree') {
                            navigateToPath(file.path);
                          } else {
                            viewFile(file.path);
                          }
                        }}
                      >
                        <div className="flex items-center space-x-3">
                          <span className="text-lg">
                            {getFileIcon(file.name, file.type === 'tree')}
                          </span>
                          <span className="text-[var(--bp-text)]">{file.name}</span>
                        </div>
                        
                        <div className="text-xs text-[var(--bp-dim)]">
                          {file.size !== undefined && formatBytes(file.size)}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {activeTab === 'commits' && (
          <div className="space-y-4">
            {commits.length === 0 ? (
              <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-8 text-center">
                <div className="text-[var(--bp-dim)]">No commits yet</div>
              </div>
            ) : (
              commits.map((commit) => (
                <div
                  key={commit.hash}
                  className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-4"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="text-[var(--bp-text)] font-medium mb-2">
                        {commit.message}
                      </div>
                      <div className="flex items-center space-x-4 text-xs text-[var(--bp-dim)]">
                        <span>{commit.author}</span>
                        <span>{formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}</span>
                        <span className="font-mono">{commit.hash.slice(0, 7)}</span>
                      </div>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'readme' && (
          <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg">
            {repo.readme_content ? (
              <div className="p-6">
                <div className="markdown-content">
                  <ReactMarkdown>
                    {repo.readme_content}
                  </ReactMarkdown>
                </div>
              </div>
            ) : (
              <div className="p-8 text-center text-[var(--bp-dim)]">
                No README found
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}