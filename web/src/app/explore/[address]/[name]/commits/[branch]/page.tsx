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

interface Commit {
  hash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
}

export default function CommitsPage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const branch = Array.isArray(params.branch) ? params.branch[0] : params.branch;

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

        // Get commit history for branch
        const commitsRes = await fetch(`/api/explorer/repos/${address}/${name}/commits?branch=${branch}&limit=50`);
        if (!commitsRes.ok) {
          if (commitsRes.status === 404) {
            throw new Error('Branch not found');
          }
          throw new Error('Failed to load commits');
        }
        
        const commitsData = await commitsRes.json();
        const commitList = commitsData.commits || [];
        setCommits(commitList);
        setHasMore(commitList.length === 50); // Check if there might be more
      } catch (err) {
        console.error('Commits page error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load commits');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [address, name, branch]);

  const loadMoreCommits = async () => {
    if (!address || !name || !branch || loadingMore) return;

    try {
      setLoadingMore(true);
      const offset = commits.length;
      const response = await fetch(`/api/explorer/repos/${address}/${name}/commits?branch=${branch}&limit=50&offset=${offset}`);
      
      if (response.ok) {
        const data = await response.json();
        const newCommits = data.commits || [];
        setCommits(prev => [...prev, ...newCommits]);
        setHasMore(newCommits.length === 50);
      }
    } catch (error) {
      console.error('Error loading more commits:', error);
    } finally {
      setLoadingMore(false);
    }
  };

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
              <p>Loading commits...</p>
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
              <h3>Commits not found</h3>
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
          <span className="explore-breadcrumb-current">commits</span>
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
                Commits on <code>{branch}</code>
              </h1>
              <p className="explore-repo-detail-subtitle">
                {commits.length} commits
              </p>
            </div>
            <div className="explore-repo-detail-actions">
              <Link
                href={repoUrls.tree(address, name, branch)}
                className="explore-action-btn"
              >
                Browse files
              </Link>
            </div>
          </div>

          <div className="explore-repo-tab-content">
            <div className="explore-commit-list">
              {commits.length === 0 ? (
                <div className="explore-empty">
                  <h3>No commits found</h3>
                  <p>This branch doesn't have any commits yet.</p>
                </div>
              ) : (
                <>
                  {commits.map((commit) => (
                    <div key={commit.hash} className="explore-commit-item">
                      <div className="explore-commit-message">
                        <Link
                          href={repoUrls.commit(address, name, commit.hash)}
                          className="explore-commit-message-link"
                        >
                          {commit.message}
                        </Link>
                      </div>
                      <div className="explore-commit-meta">
                        <code className="explore-commit-author">
                          {formatAddress(repo.owner_address)}
                        </code>
                        <span className="explore-commit-time">
                          {formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}
                        </span>
                        <Link
                          href={repoUrls.commit(address, name, commit.hash)}
                          className="explore-commit-hash"
                        >
                          <code>{commit.hash.slice(0, 7)}</code>
                        </Link>
                      </div>
                    </div>
                  ))}

                  {/* Load More Button */}
                  {hasMore && (
                    <div className="explore-load-more">
                      <button
                        onClick={loadMoreCommits}
                        disabled={loadingMore}
                        className="explore-load-more-btn"
                      >
                        {loadingMore ? 'Loading...' : 'Load more commits'}
                      </button>
                    </div>
                  )}
                </>
              )}
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}