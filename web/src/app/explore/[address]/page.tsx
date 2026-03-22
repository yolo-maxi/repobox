'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import { resolveNameToAddress } from '@/lib/addressResolver';
import EmptyState from '@/components/EmptyState';
import { EmptyRepository, AddressNotFound } from '@/components/illustrations';

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
  const router = useRouter();
  const [repos, setRepos] = useState<Repo[]>([]);
  const [loading, setLoading] = useState(true);
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolving, setResolving] = useState(true);
  const [notFound, setNotFound] = useState(false);

  const addressOrName = Array.isArray(params.address) ? params.address[0] : params.address;

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
  }, [addressOrName, router]);

  useEffect(() => {
    if (!resolvedAddress) return;
    const fetchData = async () => {
      try {
        const res = await fetch(`/api/explorer/repos?owner=${resolvedAddress}`);
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
  }, [resolvedAddress]);

  if (!addressOrName) return null;

  if (resolving) {
    return (
      <div className="rd-explore-page">
        <header className="rd-header">
          <div className="rd-header-inner">
            <Link href="/" className="rd-logo">repo<span className="rd-logo-dot">.</span>box</Link>
            <nav className="rd-nav">
              <Link href="/" className="rd-nav-link">Home</Link>
              <Link href="/explore" className="rd-nav-link rd-nav-link--active">Explore</Link>
              <Link href="/docs" className="rd-nav-link">Docs</Link>
            </nav>
          </div>
        </header>
        <div className="rd-detail-loading">
          <div className="rd-spinner" />
          <p>Resolving address...</p>
        </div>
      </div>
    );
  }

  if (notFound) {
    return (
      <div className="rd-explore-page">
        <header className="rd-header">
          <div className="rd-header-inner">
            <Link href="/" className="rd-logo">repo<span className="rd-logo-dot">.</span>box</Link>
            <nav className="rd-nav">
              <Link href="/" className="rd-nav-link">Home</Link>
              <Link href="/explore" className="rd-nav-link rd-nav-link--active">Explore</Link>
              <Link href="/docs" className="rd-nav-link">Docs</Link>
            </nav>
          </div>
        </header>
        <div className="rd-addr-content">
          <EmptyState
            illustration={AddressNotFound}
            title="Address not found"
            description={`Could not resolve "${addressOrName}" to an address.`}
            action={{ label: "Back to Explorer", href: "/explore" }}
            size="lg"
          />
        </div>
      </div>
    );
  }

  return (
    <div className="rd-explore-page">
      <header className="rd-header">
        <div className="rd-header-inner">
          <Link href="/" className="rd-logo">repo<span className="rd-logo-dot">.</span>box</Link>
          <nav className="rd-nav">
            <Link href="/" className="rd-nav-link">Home</Link>
            <Link href="/explore" className="rd-nav-link rd-nav-link--active">Explore</Link>
            <Link href="/docs" className="rd-nav-link">Docs</Link>
            <Link href="/playground" className="rd-nav-link">Playground</Link>
          </nav>
        </div>
      </header>

      {/* Breadcrumb */}
      <div className="rd-breadcrumb-bar">
        <div className="rd-breadcrumb-inner">
          <Link href="/explore" className="rd-breadcrumb-link">Explore</Link>
          <span className="rd-breadcrumb-sep">/</span>
          <span className="rd-breadcrumb-current">Developer</span>
        </div>
      </div>

      <div className="rd-addr-content">
        {/* Profile header */}
        <div className="rd-profile-header">
          <div className="rd-profile-left">
            <div className="rd-profile-avatar">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.5">
                <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
              </svg>
            </div>
            <div>
              <h1 className="rd-profile-title">Developer</h1>
              <AddressDisplay
                address={resolvedAddress || ''}
                displayName={addressOrName !== resolvedAddress ? addressOrName : undefined}
                size="lg"
                linkable={false}
                showCopy={true}
                showTooltip={true}
              />
            </div>
          </div>
          <div className="rd-profile-stats">
            <div className="rd-profile-stat">
              <span className="rd-profile-stat-num">{repos.length}</span>
              <span className="rd-profile-stat-label">repositories</span>
            </div>
            <div className="rd-profile-stat">
              <span className="rd-profile-stat-num">
                {repos.reduce((sum, r) => sum + r.commit_count, 0).toLocaleString()}
              </span>
              <span className="rd-profile-stat-label">commits</span>
            </div>
          </div>
        </div>

        {/* Repo grid */}
        <div className="rd-addr-section">
          <h2 className="rd-addr-section-title">Repositories</h2>

          {loading ? (
            <div className="rd-addr-grid">
              {[1,2,3,4].map(i => (
                <div key={i} className="rd-addr-card rd-addr-card--skeleton" />
              ))}
            </div>
          ) : repos.length === 0 ? (
            <EmptyState
              illustration={EmptyRepository}
              title="No repositories found"
              description="This developer hasn't published any repositories yet"
              size="lg"
            />
          ) : (
            <div className="rd-addr-grid">
              {repos.map((repo) => {
                const isRecent = repo.last_commit_date &&
                  new Date(repo.last_commit_date).getTime() > Date.now() - (7 * 24 * 60 * 60 * 1000);
                return (
                  <Link
                    key={`${repo.address}/${repo.name}`}
                    href={`/explore/${repo.address}/${repo.name}`}
                    className={`rd-addr-card ${isRecent ? 'rd-addr-card--active' : ''}`}
                  >
                    <div className="rd-addr-card-top">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" className="rd-addr-card-icon">
                        <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8Z"/>
                      </svg>
                      <h3 className="rd-addr-card-name">{repo.name}</h3>
                    </div>
                    {repo.description && (
                      <p className="rd-addr-card-desc">
                        {repo.description.replace(/\n/g, ' ').trim()}
                      </p>
                    )}
                    <div className="rd-addr-card-meta">
                      <span>
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                          <path d="M11.93 8.5a4.002 4.002 0 0 1-7.86 0H.75a.75.75 0 0 1 0-1.5h3.32a4.002 4.002 0 0 1 7.86 0h3.32a.75.75 0 0 1 0 1.5Zm-1.43-.25a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z"/>
                        </svg>
                        {repo.commit_count} {repo.commit_count === 1 ? 'commit' : 'commits'}
                      </span>
                      {repo.last_commit_date && (
                        <span>Updated {formatTimeAgo(repo.last_commit_date)}</span>
                      )}
                    </div>
                  </Link>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
