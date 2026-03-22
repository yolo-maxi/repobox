'use client';

import { useState, useEffect, useCallback } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, formatBytes, getFileIcon, copyToClipboard } from '@/lib/utils';
import { resolveNameToAddress } from '@/lib/addressResolver';
import { repoUrls, generateBreadcrumbs } from '@/lib/repoUrls';
import MarkdownRenderer from '@/components/markdown/MarkdownRenderer';
import BranchSelector from '@/components/BranchSelector';
import CloneUrlWidget from '@/components/CloneUrlWidget';
import ContributionChart from '@/components/explore/ContributionChart';
import FileTree from '@/components/explore/FileTree';
import { SiteNav } from '@/components/SiteNav';

interface RepoDetails {
  address: string; name: string; owner_address: string; created_at: string;
  commit_count: number; default_branch: string;
  file_tree: Array<{ type: 'blob' | 'tree'; name: string; size?: number; path: string }>;
  readme_content: string | null;
}
interface Commit { hash: string; author: string; email: string; timestamp: number; message: string }
interface FileContent { path: string; content: string }
interface RepoConfig { exists: boolean; content: string }
interface Branch { name: string; is_default: boolean; last_commit: { hash: string; timestamp: number; message: string } }
interface Contributor { address: string; pushCount: number; lastPush: string; isOwner: boolean }

type TabType = 'readme' | 'files' | 'commits' | 'contributors' | 'config';

// ENS name cache for contributor display
const ensNameCache = new Map<string, string | null>();

function useENSName(address: string): string | null {
  const [name, setName] = useState<string | null>(ensNameCache.get(address) ?? null);
  useEffect(() => {
    if (ensNameCache.has(address)) { setName(ensNameCache.get(address)!); return; }
    fetch(`/api/explorer/reverse/${address}`)
      .then(r => r.ok ? r.json() : null)
      .then(d => {
        const n = d?.displayName || null;
        ensNameCache.set(address, n);
        setName(n);
      })
      .catch(() => { ensNameCache.set(address, null); });
  }, [address]);
  return name;
}

// Simple YAML syntax highlighting
function highlightYaml(code: string): string {
  return code
    .split('\n')
    .map(line => {
      // Comments
      if (/^\s*#/.test(line)) return `<span class="hl-comment">${escHtml(line)}</span>`;
      // Key: value
      const m = line.match(/^(\s*)([\w.-]+)(\s*:\s*)(.*)/);
      if (m) {
        const [, indent, key, colon, val] = m;
        let valHtml = escHtml(val);
        // String values
        if (/^["']/.test(val)) valHtml = `<span class="hl-string">${escHtml(val)}</span>`;
        // Booleans/numbers
        else if (/^(true|false|null|\d+)$/i.test(val.trim())) valHtml = `<span class="hl-bool">${escHtml(val)}</span>`;
        // Wildcards
        else if (val.trim() === '*') valHtml = `<span class="hl-wildcard">${escHtml(val)}</span>`;
        return `${escHtml(indent)}<span class="hl-key">${escHtml(key)}</span><span class="hl-colon">${escHtml(colon)}</span>${valHtml}`;
      }
      // List items
      const listMatch = line.match(/^(\s*)(- )(.*)/);
      if (listMatch) {
        return `${escHtml(listMatch[1])}<span class="hl-dash">${escHtml(listMatch[2])}</span>${escHtml(listMatch[3])}`;
      }
      return escHtml(line);
    })
    .join('\n');
}
function escHtml(s: string): string {
  return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

// Contributor card with ENS
function ContributorCard({ contributor }: { contributor: Contributor }) {
  const ensName = useENSName(contributor.address);
  return (
    <Link href={`/explore/${contributor.address}`} className="rd-contributor-card">
      <div className="rd-contributor-header">
        <div className="rd-contributor-avatar">
          {ensName ? ensName.charAt(0).toUpperCase() : '0x'}
        </div>
        <div>
          <div className="rd-contributor-name">
            {ensName || formatAddress(contributor.address)}
          </div>
          {ensName && <div className="rd-contributor-addr">{formatAddress(contributor.address)}</div>}
        </div>
        {contributor.isOwner && <span className="rd-badge">owner</span>}
      </div>
      <div className="rd-contributor-stats">
        <span><strong>{contributor.pushCount}</strong> pushes</span>
        <span className="rd-dim">{formatTimeAgo(contributor.lastPush)}</span>
      </div>
    </Link>
  );
}

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
        } else { setNotFound(true); }
      } catch { setNotFound(true); }
      finally { setResolving(false); }
    };
    resolveAddress();
  }, [addressOrName, name, router]);

  useEffect(() => {
    if (!resolvedAddress || !name) return;
    const fetchRepo = async () => {
      try {
        const bp = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
        const cbp = selectedBranch !== 'HEAD' ? `?branch=${selectedBranch}` : '';
        const [repoRes, commitsRes, branchesRes, contributorsRes] = await Promise.all([
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}${bp}`),
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}/commits?limit=30${selectedBranch !== 'HEAD' ? `&branch=${selectedBranch}` : ''}`),
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}/branches`),
          fetch(`/api/explorer/repos/${resolvedAddress}/${name}/contributors`)
        ]);
        if (repoRes.ok) {
          const data = await repoRes.json();
          setRepo(data); setCurrentFiles(data.file_tree || []);
          if (!data.readme_content) setActiveTab('files');
        }
        if (commitsRes.ok) { const d = await commitsRes.json(); setCommits(d.commits || []); }
        if (branchesRes.ok) {
          const bd = await branchesRes.json(); setBranches(bd.branches || []);
          if (selectedBranch === 'HEAD' && bd.default_branch) setSelectedBranch(bd.default_branch);
        }
        if (contributorsRes.ok) { const cd = await contributorsRes.json(); setContributors(cd.contributors || []); }
        const configRes = await fetch(`/api/explorer/repos/${resolvedAddress}/${name}/blob/.repobox/config.yml${cbp}`);
        if (configRes.ok) {
          const d = await configRes.json();
          if (d.content) setRepoConfig({ exists: true, content: d.content });
        }
      } catch (e) { console.error(e); }
      finally { setLoading(false); setBranchLoading(false); }
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
  const handleBranchChange = (newBranch: string) => {
    setBranchLoading(true); setSelectedBranch(newBranch);
    setCurrentPath(''); setFileContent(null);
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

  // Loading / not found states
  if (resolving || loading) {
    return (
      <div className="rd-root">
        <SiteNav />
        <div className="rd-center"><div className="rd-spinner" /><p>Loading…</p></div>
      </div>
    );
  }
  if (notFound || !repo) {
    return (
      <div className="rd-root">
        <SiteNav />
        <div className="rd-center">
          <h3>{notFound ? 'Address not found' : 'Repository not found'}</h3>
          <Link href="/explore" style={{ color: 'var(--bp-accent)' }}>← Back to Explorer</Link>
        </div>
      </div>
    );
  }

  const breadcrumbs = generateBreadcrumbs(resolvedAddress || '', name, selectedBranch, currentPath, !!fileContent);
  const tabs: { id: TabType; label: string; count?: number; disabled?: boolean }[] = [
    { id: 'readme', label: 'README', disabled: !repo.readme_content },
    { id: 'files', label: 'Files', count: currentFiles.length },
    { id: 'commits', label: 'Commits', count: repo.commit_count },
    { id: 'contributors', label: 'Users', count: contributors.length },
    { id: 'config', label: 'Config' },
  ];

  // Sort files: directories first, then alphabetical
  const sortedFiles = [...currentFiles].sort((a, b) => {
    if (a.type !== b.type) return a.type === 'tree' ? -1 : 1;
    return a.name.localeCompare(b.name);
  });

  return (
    <div className="rd-root">
      <SiteNav />

      {/* Repo header */}
      <div className="rd-header">
        <div className="rd-header-inner">
          <div className="rd-breadcrumb">
            <Link href="/explore">explore</Link>
            <span>/</span>
            <Link href={`/explore/${repo.owner_address}`}>{formatAddress(repo.owner_address)}</Link>
            <span>/</span>
            <span className="rd-breadcrumb-current">{repo.name}</span>
          </div>
          <div className="rd-header-row">
            <h1 className="rd-repo-title">{repo.name}</h1>
            <div className="rd-header-actions">
              {branches.length > 0 && (
                <BranchSelector
                  branches={branches}
                  currentBranch={selectedBranch}
                  defaultBranch={repo.default_branch}
                  onChange={handleBranchChange}
                  disabled={branchLoading}
                />
              )}
            </div>
          </div>
          <div className="rd-owner-row">
            <code className="rd-owner-addr">{repo.owner_address}</code>
            <button onClick={handleCopyAddr} className="rd-copy-btn" title="Copy address">
              {addrCopied ? '✓' : '⎘'}
            </button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="rd-content">
        {/* Tabs */}
        <div className="rd-tabs">
          {tabs.map(t => (
            <button
              key={t.id}
              onClick={() => !t.disabled && setActiveTab(t.id)}
              className={`rd-tab ${activeTab === t.id ? 'active' : ''} ${t.disabled ? 'disabled' : ''}`}
            >
              {t.label}
              {t.count !== undefined && <span className="rd-tab-count">{t.count}</span>}
            </button>
          ))}
        </div>

        {/* Tab content */}
        <div className="rd-panel">
          {/* README */}
          {activeTab === 'readme' && (
            repo.readme_content ? (
              <div className="rd-readme">
                <MarkdownRenderer
                  content={repo.readme_content}
                  baseUrl={`/api/explorer/repos/${resolvedAddress}/${name}/blob/`}
                  className="rd-readme-md"
                />
              </div>
            ) : (
              <div className="rd-empty">No README found</div>
            )
          )}

          {/* Files */}
          {activeTab === 'files' && (
            <div className="rd-files">
              <FileTree
                address={resolvedAddress || ''}
                repoName={name}
                branch={selectedBranch}
                initialFiles={repo.file_tree || []}
                onFileClick={(path) => viewFile(path)}
              />
            </div>
          )}

          {/* Commits */}
          {activeTab === 'commits' && (
            <div className="rd-commits">
              {commits.length === 0 ? (
                <div className="rd-empty">No commits found</div>
              ) : commits.slice(0, 20).map((c) => (
                <div key={c.hash} className="rd-commit">
                  <div className="rd-commit-left">
                    <Link href={repoUrls.commit(resolvedAddress || '', name, c.hash)} className="rd-commit-msg">
                      {c.message.split('\n')[0]}
                    </Link>
                    <div className="rd-commit-meta">
                      <code className="rd-commit-author">{c.author}</code>
                      <span className="rd-dim">committed {formatTimeAgo(new Date(c.timestamp * 1000).toISOString())}</span>
                    </div>
                  </div>
                  <Link href={repoUrls.commit(resolvedAddress || '', name, c.hash)} className="rd-commit-hash">
                    {c.hash.slice(0, 7)}
                  </Link>
                </div>
              ))}
            </div>
          )}

          {/* Contributors */}
          {activeTab === 'contributors' && (
            <div className="rd-contributors">
              {contributors.length === 0 ? (
                <div className="rd-empty">No contributors found</div>
              ) : (
                <>
                  <div className="rd-contributor-grid">
                    {contributors.map(c => <ContributorCard key={c.address} contributor={c} />)}
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

          {/* Config */}
          {activeTab === 'config' && (
            <div className="rd-config">
              <div className="rd-config-section">
                <h3 className="rd-config-heading">Owner Identity</h3>
                <div className="rd-config-identity">
                  <code>{repo.owner_address}</code>
                  <span className="rd-badge">owner</span>
                </div>
              </div>
              <div className="rd-config-section">
                <h3 className="rd-config-heading">Permission Ruleset</h3>
                {repoConfig.exists ? (
                  <div className="rd-config-code-wrap">
                    <div className="rd-config-code-header">.repobox/config.yml</div>
                    <pre className="rd-config-code" dangerouslySetInnerHTML={{ __html: highlightYaml(repoConfig.content) }} />
                  </div>
                ) : (
                  <div className="rd-config-empty">
                    <p>No <code>.repobox/config.yml</code> found.</p>
                    <p className="rd-dim">Run <code>repobox init</code> to create a permission config.</p>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Sidebar — stats, clone, recent commits (below tabs on mobile) */}
        <aside className="rd-sidebar">
          {/* Stats */}
          <div className="rd-sidebar-card">
            <div className="rd-sidebar-label">About</div>
            <div className="rd-stat-row">
              <span>Commits</span><strong>{repo.commit_count}</strong>
            </div>
            <div className="rd-stat-row">
              <span>Contributors</span><strong>{contributors.length}</strong>
            </div>
            <div className="rd-stat-row">
              <span>Branch</span><strong>{selectedBranch}</strong>
            </div>
            <div className="rd-stat-row">
              <span>Created</span><strong>{formatTimeAgo(repo.created_at)}</strong>
            </div>
          </div>

          {/* Clone */}
          <CloneUrlWidget ownerAddress={repo.owner_address} repoName={repo.name} />

          {/* Recent commits */}
          <div className="rd-sidebar-card">
            <div className="rd-sidebar-label">Latest Commits</div>
            {commits.slice(0, 5).map(c => (
              <div key={c.hash} className="rd-sidebar-commit">
                <Link href={repoUrls.commit(resolvedAddress || '', name, c.hash)} className="rd-sidebar-commit-msg">
                  {c.message.split('\n')[0].substring(0, 50)}{c.message.length > 50 ? '…' : ''}
                </Link>
                <div className="rd-sidebar-commit-meta">
                  <code>{c.hash.slice(0, 7)}</code>
                  <span>{formatTimeAgo(new Date(c.timestamp * 1000).toISOString())}</span>
                </div>
              </div>
            ))}
          </div>
        </aside>
      </div>

      <style>{`
        .rd-root {
          min-height: 100vh;
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono, 'JetBrains Mono', monospace);
          font-size: 13px;
        }
        .rd-center {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: 60vh;
          gap: 12px;
          color: var(--bp-dim);
        }
        .rd-spinner {
          width: 24px; height: 24px;
          border: 2px solid var(--bp-border);
          border-top-color: var(--bp-accent);
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }
        @keyframes spin { to { transform: rotate(360deg); } }

        /* Header */
        .rd-header {
          border-bottom: 1px solid var(--bp-border);
          padding: 20px 32px;
        }
        .rd-header-inner { max-width: 1280px; margin: 0 auto; }
        .rd-breadcrumb {
          display: flex; align-items: center; gap: 6px;
          font-size: 12px; color: var(--bp-dim); margin-bottom: 12px;
        }
        .rd-breadcrumb a { color: var(--bp-accent); text-decoration: none; }
        .rd-breadcrumb a:hover { opacity: 0.8; }
        .rd-breadcrumb-current { color: var(--bp-heading); font-weight: 600; }

        .rd-header-row {
          display: flex; align-items: center; justify-content: space-between;
          gap: 16px; margin-bottom: 8px;
        }
        .rd-repo-title {
          font-size: 24px; font-weight: 700; color: var(--bp-heading);
          letter-spacing: -0.5px;
        }
        .rd-header-actions { display: flex; gap: 8px; align-items: center; }
        .rd-owner-row {
          display: flex; align-items: center; gap: 8px;
          font-size: 11px; color: var(--bp-dim);
        }
        .rd-owner-addr { font-size: 11px; opacity: 0.7; }
        .rd-copy-btn {
          background: none; border: none; color: var(--bp-dim);
          cursor: pointer; font-size: 14px; padding: 2px 4px;
          border-radius: 3px; transition: color 0.15s;
          font-family: inherit;
        }
        .rd-copy-btn:hover { color: var(--bp-accent); }

        /* Content grid */
        .rd-content {
          max-width: 1280px; margin: 0 auto;
          padding: 24px 32px;
          display: grid;
          grid-template-columns: 1fr 280px;
          grid-template-rows: auto 1fr;
          gap: 24px;
        }

        /* Tabs */
        .rd-tabs {
          grid-column: 1;
          display: flex; gap: 2px;
          border-bottom: 1px solid var(--bp-border);
          padding-bottom: 0;
        }
        .rd-tab {
          padding: 8px 16px;
          border: none; cursor: pointer;
          font-size: 12px; font-weight: 500;
          font-family: inherit;
          background: transparent;
          color: var(--bp-dim);
          border-bottom: 2px solid transparent;
          margin-bottom: -1px;
          transition: all 0.12s;
        }
        .rd-tab:hover:not(.disabled) { color: var(--bp-text); }
        .rd-tab.active {
          color: var(--bp-accent);
          border-bottom-color: var(--bp-accent);
        }
        .rd-tab.disabled { opacity: 0.3; cursor: not-allowed; }
        .rd-tab-count {
          margin-left: 6px; font-size: 10px;
          background: rgba(79, 195, 247, 0.1);
          color: var(--bp-accent);
          padding: 1px 6px; border-radius: 10px;
        }

        /* Panel */
        .rd-panel {
          grid-column: 1;
          min-width: 0;
        }

        /* Sidebar */
        .rd-sidebar {
          grid-column: 2;
          grid-row: 1 / 3;
          display: flex;
          flex-direction: column;
          gap: 16px;
        }
        .rd-sidebar-card {
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          padding: 16px;
        }
        .rd-sidebar-label {
          font-size: 10px; font-weight: 600;
          text-transform: uppercase; letter-spacing: 0.1em;
          color: var(--bp-dim); margin-bottom: 12px;
        }
        .rd-stat-row {
          display: flex; justify-content: space-between;
          padding: 4px 0; font-size: 12px; color: var(--bp-dim);
        }
        .rd-stat-row strong {
          color: var(--bp-heading); font-variant-numeric: tabular-nums;
        }
        .rd-sidebar-commit { margin-bottom: 10px; }
        .rd-sidebar-commit:last-child { margin-bottom: 0; }
        .rd-sidebar-commit-msg {
          color: var(--bp-text); text-decoration: none;
          font-size: 12px; line-height: 1.4;
          display: block;
        }
        .rd-sidebar-commit-msg:hover { color: var(--bp-accent); }
        .rd-sidebar-commit-meta {
          display: flex; gap: 8px; font-size: 10px;
          color: var(--bp-dim); opacity: 0.6; margin-top: 2px;
        }

        /* Empty & dim */
        .rd-empty { padding: 40px; text-align: center; color: var(--bp-dim); }
        .rd-dim { color: var(--bp-dim); opacity: 0.6; }
        .rd-badge {
          font-size: 10px; padding: 2px 8px;
          background: rgba(79, 195, 247, 0.12);
          color: var(--bp-accent); border-radius: 10px;
          font-weight: 500;
        }

        /* README */
        .rd-readme { padding: 24px; }

        /* Files */
        .rd-files { }
        .rd-file-breadcrumb {
          display: flex; align-items: center; gap: 6px;
          padding: 12px 16px; border-bottom: 1px solid var(--bp-border);
          font-size: 12px;
        }
        .rd-back-btn, .rd-crumb-link {
          background: none; border: none; color: var(--bp-accent);
          cursor: pointer; font-family: inherit; font-size: 12px;
          padding: 2px 4px;
        }
        .rd-crumb-sep { color: var(--bp-dim); opacity: 0.4; }
        .rd-crumb-file { color: var(--bp-heading); font-weight: 500; }

        .rd-file-table {
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          overflow: hidden;
        }
        .rd-file-row {
          display: flex; align-items: center;
          width: 100%; padding: 10px 16px;
          background: none; border: none; border-bottom: 1px solid var(--bp-border);
          cursor: pointer; font-family: inherit; font-size: 13px;
          color: var(--bp-text); text-align: left;
          transition: background 0.1s;
          gap: 10px;
        }
        .rd-file-row:last-child { border-bottom: none; }
        .rd-file-row:hover { background: rgba(79, 195, 247, 0.03); }
        .rd-file-icon { width: 20px; text-align: center; flex-shrink: 0; font-size: 14px; }
        .rd-file-name { flex: 1; }
        .rd-file-name.dir { color: var(--bp-accent); font-weight: 500; }
        .rd-file-size { color: var(--bp-dim); font-size: 11px; opacity: 0.6; }

        .rd-file-viewer { border: 1px solid var(--bp-border); border-radius: 8px; overflow: hidden; }
        .rd-file-viewer-header {
          padding: 10px 16px; background: var(--bp-surface);
          border-bottom: 1px solid var(--bp-border);
          font-size: 12px; color: var(--bp-heading);
        }
        .rd-file-code {
          margin: 0; padding: 16px;
          background: var(--bp-bg);
          font-size: 12px; line-height: 1.5;
          overflow-x: auto; color: var(--bp-text);
        }

        /* Commits */
        .rd-commits { }
        .rd-commit {
          display: flex; align-items: flex-start;
          justify-content: space-between; gap: 16px;
          padding: 14px 0;
          border-bottom: 1px solid var(--bp-border);
        }
        .rd-commit:last-child { border-bottom: none; }
        .rd-commit-left { flex: 1; min-width: 0; }
        .rd-commit-msg {
          color: var(--bp-heading); text-decoration: none;
          font-size: 13px; font-weight: 500; line-height: 1.4;
          display: block;
        }
        .rd-commit-msg:hover { color: var(--bp-accent); }
        .rd-commit-meta {
          display: flex; gap: 8px; align-items: center;
          margin-top: 4px; font-size: 11px;
        }
        .rd-commit-author { color: var(--bp-dim); font-size: 11px; }
        .rd-commit-hash {
          font-size: 12px; color: var(--bp-accent);
          text-decoration: none; font-family: inherit;
          padding: 4px 10px;
          background: rgba(79, 195, 247, 0.08);
          border-radius: 4px;
          flex-shrink: 0;
        }
        .rd-commit-hash:hover { background: rgba(79, 195, 247, 0.15); }

        /* Contributors */
        .rd-contributor-grid {
          display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
          gap: 12px; margin-bottom: 24px;
        }
        .rd-contributor-card {
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 8px; padding: 16px;
          text-decoration: none; color: inherit;
          transition: border-color 0.15s;
        }
        .rd-contributor-card:hover { border-color: rgba(79, 195, 247, 0.3); }
        .rd-contributor-header {
          display: flex; align-items: center; gap: 10px; margin-bottom: 10px;
        }
        .rd-contributor-avatar {
          width: 32px; height: 32px; border-radius: 50%;
          background: rgba(79, 195, 247, 0.12);
          display: flex; align-items: center; justify-content: center;
          font-size: 11px; font-weight: 700; color: var(--bp-accent);
          flex-shrink: 0;
        }
        .rd-contributor-name { font-size: 13px; font-weight: 600; color: var(--bp-heading); }
        .rd-contributor-addr { font-size: 10px; color: var(--bp-dim); opacity: 0.6; }
        .rd-contributor-stats {
          display: flex; justify-content: space-between;
          font-size: 11px; color: var(--bp-dim);
        }

        /* Config */
        .rd-config { padding: 20px 0; }
        .rd-config-section { margin-bottom: 24px; }
        .rd-config-heading {
          font-size: 12px; font-weight: 600; color: var(--bp-heading);
          text-transform: uppercase; letter-spacing: 0.05em;
          margin-bottom: 12px;
        }
        .rd-config-identity {
          display: flex; align-items: center; gap: 10px;
          padding: 12px 16px;
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 6px; font-size: 12px;
        }
        .rd-config-code-wrap {
          border: 1px solid var(--bp-border);
          border-radius: 8px; overflow: hidden;
        }
        .rd-config-code-header {
          padding: 8px 16px;
          background: var(--bp-surface);
          border-bottom: 1px solid var(--bp-border);
          font-size: 11px; color: var(--bp-dim);
        }
        .rd-config-code {
          margin: 0; padding: 16px 20px;
          background: var(--bp-bg);
          font-size: 12px; line-height: 1.6;
          overflow-x: auto;
        }
        .rd-config-empty { color: var(--bp-dim); font-size: 12px; padding: 16px; }
        .rd-config-empty code {
          color: var(--bp-accent);
          background: rgba(79, 195, 247, 0.08);
          padding: 1px 6px; border-radius: 3px;
        }

        /* YAML highlighting */
        .hl-key { color: #4fc3f7; }
        .hl-colon { color: var(--bp-dim); }
        .hl-string { color: #a5d6a7; }
        .hl-bool { color: #ffb74d; }
        .hl-wildcard { color: #ff8a65; font-weight: 700; }
        .hl-comment { color: #546e7a; font-style: italic; }
        .hl-dash { color: var(--bp-dim); }

        /* Mobile */
        @media (max-width: 900px) {
          .rd-content {
            grid-template-columns: 1fr;
            padding: 16px;
          }
          .rd-sidebar {
            grid-column: 1;
            grid-row: auto;
            order: 3;
          }
          .rd-tabs { order: 1; overflow-x: auto; }
          .rd-panel { order: 2; }
          .rd-header { padding: 16px; }
          .rd-repo-title { font-size: 20px; }
          .rd-contributor-grid { grid-template-columns: 1fr; }
        }
      `}</style>
    </div>
  );
}
