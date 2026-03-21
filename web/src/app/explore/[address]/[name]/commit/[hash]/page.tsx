'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import { repoUrls } from '@/lib/repoUrls';
import ExploreHeader from '@/components/explore/ExploreHeader';
import ExploreSidebar from '@/components/explore/ExploreSidebar';

interface RepoDetails {
  address: string;
  name: string;
  owner_address: string;
  default_branch: string;
}

interface CommitDetails {
  hash: string;
  shortHash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
  parentHash: string | null;
  childHash: string | null;
  // Note: In a full implementation, you'd also include file changes here
}

export default function CommitPage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commit, setCommit] = useState<CommitDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const hash = Array.isArray(params.hash) ? params.hash[0] : params.hash;

  useEffect(() => {
    if (!address || !name || !hash) return;

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

        // Try to get commit details
        // Note: This would need a new API endpoint in a full implementation
        // For now, we'll create a minimal commit object
        const commitData: CommitDetails = {
          hash,
          shortHash: hash.slice(0, 7),
          author: 'Unknown',
          email: '',
          timestamp: Date.now() / 1000,
          message: 'Commit details not yet implemented',
          parentHash: null,
          childHash: null
        };
        
        setCommit(commitData);
      } catch (err) {
        console.error('Commit page error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load commit');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [address, name, hash]);

  if (!address || !name || !hash) {
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
              <p>Loading commit...</p>
            </div>
          </main>
        </div>
      </div>
    );
  }

  if (error || !repo || !commit) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-empty">
              <h3>Commit not found</h3>
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

  return (
    <div className="explore-layout">
      <ExploreHeader />
      
      <div className="explore-breadcrumb-nav">
        <div className="explore-main-header-content">
          <Link href="/explore" className="explore-breadcrumb-link">Explore</Link>
          <span className="explore-breadcrumb-separator">/</span>
          <Link href={`/explore/${repo.owner_address}`} className="explore-breadcrumb-link">
            {formatAddress(repo.owner_address)}
          </Link>
          <span className="explore-breadcrumb-separator">/</span>
          <Link href={repoUrls.home(address, name)} className="explore-breadcrumb-link">
            {repo.name}
          </Link>
          <span className="explore-breadcrumb-separator">/</span>
          <span className="explore-breadcrumb-current">commit</span>
          <span className="explore-breadcrumb-separator">/</span>
          <span className="explore-breadcrumb-current">{commit.shortHash}</span>
        </div>
      </div>

      <div className="explore-container">
        <ExploreSidebar />
        
        <main className="explore-main">
          <div className="explore-repo-detail-header">
            <div className="explore-repo-detail-info">
              <h1 className="explore-repo-detail-title">
                {commit.message}
              </h1>
              <div className="explore-commit-details">
                <code className="explore-commit-author">
                  {formatAddress(repo.owner_address)}
                </code>
                <span className="explore-commit-time">
                  committed {formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}
                </span>
              </div>
            </div>
            <div className="explore-repo-detail-actions">
              <Link
                href={repoUrls.commits(address, name, repo.default_branch)}
                className="explore-action-btn"
              >
                Browse commits
              </Link>
              <Link
                href={repoUrls.tree(address, name, repo.default_branch)}
                className="explore-action-btn"
              >
                Browse files
              </Link>
            </div>
          </div>

          <div className="explore-repo-tab-content">
            <div className="explore-commit-info">
              <div className="explore-commit-hash-section">
                <h3>Commit Hash</h3>
                <code className="explore-commit-full-hash">{commit.hash}</code>
              </div>
              
              <div className="explore-commit-placeholder">
                <h3>📋 Feature Coming Soon</h3>
                <p>Detailed commit diff view is not yet implemented.</p>
                <p>This page will show:</p>
                <ul>
                  <li>File changes and diffs</li>
                  <li>Added and removed lines</li>
                  <li>Parent/child commit navigation</li>
                  <li>Commit statistics</li>
                </ul>
                <p>
                  For now, you can browse the repository at this commit state by 
                  navigating to the files tab.
                </p>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}