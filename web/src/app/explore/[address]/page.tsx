'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, copyToClipboard } from '@/lib/utils';

interface Repo {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
  commit_count: number;
  last_commit_date: string | null;
  description: string | null;
}

export default function AddressPage() {
  const params = useParams();
  const [repos, setRepos] = useState<Repo[]>([]);
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;

  useEffect(() => {
    if (!address) return;
    const fetchData = async () => {
      try {
        const res = await fetch(`/api/explorer/repos?owner=${address}`);
        if (res.ok) {
          const data = await res.json();
          setRepos(data.repos || []);
        }
      } catch (error) {
        console.error('Error:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [address]);

  const handleCopyAddress = async () => {
    if (!address) return;
    await copyToClipboard(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (!address) return null;

  return (
    <div className="explore-page">
      {/* Back Link */}
      <div className="explore-back">
        <Link href="/explore" className="explore-back-link">
          ← Back to Explorer
        </Link>
      </div>

      {/* Address Header */}
      <header className="explore-address-header">
        <div className="explore-address-info">
          <div className="explore-address-avatar">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C13.1 2 14 2.9 14 4C14 5.1 13.1 6 12 6C10.9 6 10 5.1 10 4C10 2.9 10.9 2 12 2ZM21 9V7L15 1H5C3.89 1 3 1.89 3 3V21A2 2 0 0 0 5 23H19A2 2 0 0 0 21 21V9H21ZM15 3H19L15 7V3Z"/>
            </svg>
          </div>
          <div className="explore-address-details">
            <div className="explore-address-main">
              <span className="explore-address-text">{address}</span>
              <button onClick={handleCopyAddress} className="explore-copy-btn">
                {copied ? '✓' : 'Copy'}
              </button>
            </div>
          </div>
        </div>

        <div className="explore-address-stats">
          <div className="explore-stat-item">
            <span className="explore-stat-value">{repos.length}</span>
            <span className="explore-stat-label">repositories</span>
          </div>
          <div className="explore-stat-item">
            <span className="explore-stat-value">
              {repos.reduce((sum, r) => sum + r.commit_count, 0)}
            </span>
            <span className="explore-stat-label">commits</span>
          </div>
        </div>
      </header>

      {/* Repositories */}
      <section className="explore-repos-section">
        <h2 className="explore-section-title">Repositories</h2>

        {loading ? (
          <div className="explore-loading">
            <div className="explore-loading-spinner"></div>
            <p>Loading repositories...</p>
          </div>
        ) : repos.length === 0 ? (
          <div className="explore-empty">
            <p>No repositories found for this address</p>
          </div>
        ) : (
          <div className="explore-repo-list">
            {repos.map((repo) => (
              <Link
                key={`${repo.address}/${repo.name}`}
                href={`/explore/${repo.address}/${repo.name}`}
                className="explore-repo-card"
              >
                <div className="explore-repo-header">
                  <h3 className="explore-repo-name">{repo.name}</h3>
                </div>
                
                {repo.description && (
                  <p className="explore-repo-description">
                    {repo.description.replace(/\n/g, ' ').trim()}
                  </p>
                )}
                
                <div className="explore-repo-meta">
                  <span className="explore-repo-commits">{repo.commit_count} commits</span>
                  {repo.last_commit_date && (
                    <span className="explore-repo-updated">
                      Updated {formatTimeAgo(repo.last_commit_date)}
                    </span>
                  )}
                </div>
              </Link>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}