'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { formatBytes, getFileIcon } from '@/lib/utils';
import { repoUrls } from '@/lib/repoUrls';
import ExploreHeader from '@/components/explore/ExploreHeader';
import ExploreSidebar from '@/components/explore/ExploreSidebar';

interface RepoDetails {
  address: string;
  name: string;
  owner_address: string;
  default_branch: string;
}

interface FileEntry {
  type: 'blob' | 'tree';
  name: string;
  size?: number;
  path: string;
}

export default function TreePage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const branch = Array.isArray(params.branch) ? params.branch[0] : params.branch;
  const pathSegments = params.path ? (Array.isArray(params.path) ? params.path : [params.path]) : [];
  const currentPath = pathSegments.join('/');

  useEffect(() => {
    if (!address || !name || !branch) return;

    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);

        // Get repository details
        const repoRes = await fetch(`/api/explorer/repos/${address}/${name}`);
        if (!repoRes.ok) {
          throw new Error('Repository not found');
        }
        const repoData = await repoRes.json();
        setRepo(repoData);

        // Get file tree for current path
        const treePath = currentPath ? `/${currentPath}` : '';
        const treeRes = await fetch(`/api/explorer/repos/${address}/${name}/tree${treePath}?branch=${branch}`);
        if (!treeRes.ok) {
          if (treeRes.status === 404) {
            throw new Error('Path not found');
          }
          throw new Error('Failed to load directory');
        }
        
        const treeData = await treeRes.json();
        setFiles(treeData.files || []);
      } catch (err) {
        console.error('Tree page error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load directory');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [address, name, branch, currentPath]);

  if (!address || !name || !branch) {
    return notFound();
  }

  if (loading) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-loading">
              <div className="explore-loading-spinner"></div>
              <p>Loading directory...</p>
            </div>
          </main>
        </div>
      </div>
    );
  }

  if (error || !repo) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-empty">
              <h3>Directory not found</h3>
              <p>{error}</p>
              <Link href={repoUrls.home(address, name)} className="explore-back-link">
                ← Back to repository
              </Link>
            </div>
          </main>
        </div>
      </div>
    );
  }

  // Generate breadcrumb navigation
  const breadcrumbs = [
    { label: repo.name, href: repoUrls.home(address, name) }
  ];

  if (currentPath) {
    const pathParts = currentPath.split('/').filter(Boolean);
    for (let i = 0; i < pathParts.length; i++) {
      const partialPath = pathParts.slice(0, i + 1).join('/');
      breadcrumbs.push({
        label: pathParts[i],
        href: repoUrls.tree(address, name, branch, partialPath)
      });
    }
  }

  return (
    <div className="explore-layout">
      <ExploreHeader />
      
      <div className="explore-breadcrumb-nav">
        <div className="explore-main-header-content">
          <Link href="/explore" className="explore-breadcrumb-link">Explore</Link>
          <span className="explore-breadcrumb-separator">/</span>
          <Link href={`/explore/${repo.owner_address}`} className="explore-breadcrumb-link">
            {repo.owner_address.slice(0, 6)}...{repo.owner_address.slice(-4)}
          </Link>
          <span className="explore-breadcrumb-separator">/</span>
          {breadcrumbs.map((crumb, index) => (
            <span key={index}>
              {index > 0 && <span className="explore-breadcrumb-separator">/</span>}
              <Link href={crumb.href} className="explore-breadcrumb-link">
                {crumb.label}
              </Link>
            </span>
          ))}
          <span className="explore-breadcrumb-separator">@</span>
          <span className="explore-breadcrumb-current">{branch}</span>
        </div>
      </div>

      <div className="explore-container">
        <ExploreSidebar />
        
        <main className="explore-main">
          <div className="explore-repo-detail-header">
            <div className="explore-repo-detail-info">
              <h1 className="explore-repo-detail-title">
                {currentPath ? currentPath : '/'}
              </h1>
              <p className="explore-repo-detail-subtitle">
                Branch: <code>{branch}</code> • 
                {files.length} {files.length === 1 ? 'item' : 'items'}
              </p>
            </div>
          </div>

          <div className="explore-repo-tab-content">
            <div className="explore-file-list">
              {/* Parent directory link */}
              {currentPath && (
                <Link
                  href={
                    pathSegments.length > 1 
                      ? repoUrls.tree(address, name, branch, pathSegments.slice(0, -1).join('/'))
                      : repoUrls.tree(address, name, branch)
                  }
                  className="explore-file-item"
                >
                  <div className="explore-file-info">
                    <span className="explore-file-icon">📁</span>
                    <span className="explore-file-name">..</span>
                  </div>
                </Link>
              )}

              {files.length === 0 ? (
                <div className="explore-empty">
                  <p>Empty directory</p>
                </div>
              ) : (
                files.map((file, i) => (
                  <Link
                    key={i}
                    href={
                      file.type === 'tree'
                        ? repoUrls.tree(address, name, branch, file.path)
                        : repoUrls.blob(address, name, branch, file.path)
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
                  </Link>
                ))
              )}
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}