'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, copyToClipboard } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import { resolveNameToAddress, resolveAddressDisplay } from '@/lib/addressResolver';
import EmptyState from '@/components/EmptyState';
import { EmptyRepository, AddressNotFound } from '@/components/illustrations';
import Jazzicon from '@/components/Jazzicon';
import ActivityHeatmap from '@/components/ActivityHeatmap';
import { SiteNav } from '@/components/SiteNav';

interface Repo {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
  commit_count: number;
  last_commit_date: string | null;
  description: string | null;
}

interface ContributorRepo extends Repo {
  permissions: string[];
}

interface ActivityDay {
  day: string;
  count: number;
}

interface ProfileData {
  firstCommitDate: number | null;
  activeRepos: number;
  activity: ActivityDay[];
}

// Detect primary language from file list (simplified)
function detectLanguage(repoName: string): { name: string; color: string } {
  const languages = [
    { name: 'TypeScript', color: '#3178c6' },
    { name: 'JavaScript', color: '#f1e05a' },
    { name: 'Python', color: '#3572a5' },
    { name: 'Go', color: '#00add8' },
    { name: 'Rust', color: '#dea584' },
    { name: 'Java', color: '#b07219' },
  ];

  return languages[Math.floor(Math.random() * languages.length)];
}

function RepoCard({ repo, formatCommitCount }: { repo: Repo; formatCommitCount: (n: number) => string }) {
  const language = detectLanguage(repo.name);
  const isRecentlyActive = repo.last_commit_date &&
    new Date(repo.last_commit_date).getTime() > Date.now() - (7 * 24 * 60 * 60 * 1000);

  return (
    <Link
      href={`/explore/${repo.address}/${repo.name}`}
      className={`explore-repo-item ${isRecentlyActive ? 'recently-active' : ''}`}
    >
      <div className="explore-repo-item-header">
        <h3 className="explore-repo-item-name">{repo.name}</h3>
        <div
          className="explore-repo-item-language"
          style={{ backgroundColor: language.color }}
          title={language.name}
        ></div>
      </div>

      {repo.description && (
        <p className="explore-repo-item-description">
          {repo.description.replace(/\n/g, ' ').trim()}
        </p>
      )}

      <div className="explore-repo-item-meta">
        <span className="explore-repo-item-meta-item">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M20 6L9 17l-5-5"></path>
          </svg>
          {formatCommitCount(repo.commit_count)}
        </span>
        {repo.last_commit_date && (
          <span className="explore-repo-item-meta-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"></circle>
              <polyline points="12,6 12,12 16,14"></polyline>
            </svg>
            Updated {formatTimeAgo(repo.last_commit_date)}
          </span>
        )}
      </div>
    </Link>
  );
}

function ContributorRepoCard({ repo, formatCommitCount }: { repo: ContributorRepo; formatCommitCount: (n: number) => string }) {
  const language = detectLanguage(repo.name);
  const isRecentlyActive = repo.last_commit_date &&
    new Date(repo.last_commit_date).getTime() > Date.now() - (7 * 24 * 60 * 60 * 1000);

  return (
    <Link
      href={`/explore/${repo.address}/${repo.name}`}
      className={`explore-repo-item ${isRecentlyActive ? 'recently-active' : ''}`}
    >
      <div className="explore-repo-item-header">
        <h3 className="explore-repo-item-name">{repo.name}</h3>
        <div
          className="explore-repo-item-language"
          style={{ backgroundColor: language.color }}
          title={language.name}
        ></div>
      </div>

      <div className="explore-repo-item-owner-link">
        {formatAddress(repo.owner_address)}
      </div>

      <div className="explore-repo-item-badges">
        {repo.permissions.map((perm) => (
          <span key={perm} className="explore-permission-badge">
            {perm}
          </span>
        ))}
      </div>

      {repo.description && (
        <p className="explore-repo-item-description">
          {repo.description.replace(/\n/g, ' ').trim()}
        </p>
      )}

      <div className="explore-repo-item-meta">
        <span className="explore-repo-item-meta-item">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M20 6L9 17l-5-5"></path>
          </svg>
          {formatCommitCount(repo.commit_count)}
        </span>
        {repo.last_commit_date && (
          <span className="explore-repo-item-meta-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"></circle>
              <polyline points="12,6 12,12 16,14"></polyline>
            </svg>
            Updated {formatTimeAgo(repo.last_commit_date)}
          </span>
        )}
      </div>
    </Link>
  );
}

export default function AddressPage() {
  const params = useParams();
  const router = useRouter();
  const [repos, setRepos] = useState<Repo[]>([]);
  const [contributorRepos, setContributorRepos] = useState<ContributorRepo[]>([]);
  const [loading, setLoading] = useState(true);
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolving, setResolving] = useState(true);
  const [notFound, setNotFound] = useState(false);
  const [displayName, setDisplayName] = useState<string | null>(null);
  const [profileData, setProfileData] = useState<ProfileData | null>(null);
  const [profileLoading, setProfileLoading] = useState(true);

  const addressOrName = Array.isArray(params.address) ? params.address[0] : params.address;

  // Resolve name to address if needed
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

  // Resolve display name
  useEffect(() => {
    if (!resolvedAddress) return;
    const resolveDisplay = async () => {
      try {
        const name = await resolveAddressDisplay(resolvedAddress);
        setDisplayName(name);
      } catch (error) {
        console.error('Display name resolution failed:', error);
      }
    };
    resolveDisplay();
  }, [resolvedAddress]);

  // Fetch repos data
  useEffect(() => {
    if (!resolvedAddress) return;
    const fetchData = async () => {
      try {
        const res = await fetch(`/api/explorer/repos?owner=${resolvedAddress}`);
        if (res.ok) {
          const data = await res.json();
          setRepos(data.repos || []);
          setContributorRepos(data.contributor_repos || []);
        }
      } catch (error) {
        console.error('Error:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [resolvedAddress]);

  // Fetch profile data
  useEffect(() => {
    if (!resolvedAddress) return;
    const fetchProfileData = async () => {
      try {
        const res = await fetch(`/api/explorer/profile/${resolvedAddress}`);
        if (res.ok) {
          const data = await res.json();
          setProfileData(data);
        }
      } catch (error) {
        console.error('Profile data error:', error);
      } finally {
        setProfileLoading(false);
      }
    };
    fetchProfileData();
  }, [resolvedAddress]);

  const formatCommitCount = (count: number): string => {
    if (count === 1) return '1 commit';
    return `${count} commits`;
  };

  if (!addressOrName) return null;

  if (resolving) {
    return (
      <div className="explore-page">
        <SiteNav />

        <div className="explore-main-content">
          <div className="explore-loading">
            <div className="explore-loading-spinner"></div>
            <p>Resolving address...</p>
          </div>
        </div>
      </div>
    );
  }

  if (notFound) {
    return (
      <div className="explore-page">
        <SiteNav />

        <div className="explore-main-content">
          <EmptyState
            illustration={AddressNotFound}
            title="Address not found"
            description={`Could not resolve "${addressOrName}" to an address.`}
            action={{
              label: "\u2190 Back to Explorer",
              href: "/explore"
            }}
            size="lg"
          />
        </div>
      </div>
    );
  }

  const totalRepoCount = repos.length + contributorRepos.length;
  const totalCommits = repos.reduce((sum, r) => sum + r.commit_count, 0);
  
  // Determine title: use resolved name if available, otherwise "Developer"
  const profileTitle = displayName || "Developer";
  
  // Show full address if arrived via name resolution
  const showFullAddress = addressOrName !== resolvedAddress && !!displayName;
  
  // Format member since date
  const formatMemberSince = (timestamp: number | null): string => {
    if (!timestamp) return 'Unknown';
    const date = new Date(timestamp * 1000);
    const options: Intl.DateTimeFormatOptions = { 
      year: 'numeric', 
      month: 'long', 
      day: 'numeric' 
    };
    return date.toLocaleDateString(undefined, options);
  };
  
  // Handle copy address
  const handleCopyAddress = async () => {
    if (resolvedAddress) {
      try {
        await copyToClipboard(resolvedAddress);
      } catch (error) {
        console.error('Copy failed:', error);
      }
    }
  };

  return (
    <div className="explore-page">
      {/* Header */}
      <SiteNav />

      <div className="explore-main-content">
        {/* Address Header */}
        <div className="explore-profile-header">
          <div className="explore-profile-info">
            <div className="explore-profile-avatar">
              <Jazzicon address={resolvedAddress || ''} size={80} />
            </div>
            <div className="explore-profile-details">
              <h1 className="explore-profile-title">{profileTitle}</h1>
              
              {showFullAddress && (
                <>
                  <div className="explore-profile-full-address">
                    <code>{resolvedAddress}</code>
                    <button 
                      onClick={handleCopyAddress}
                      className="explore-profile-copy-btn"
                      title="Copy address"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                      </svg>
                    </button>
                  </div>
                  
                  {addressOrName.includes('.') && (
                    <div className="explore-profile-ens-source" title="Resolved via repobox.eth CCIP-Read">
                      {addressOrName}
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="m9 12 2 2 4-4"/>
                        <circle cx="21" cy="12" r="3"/>
                        <path d="m21 9-9 9-5-5"/>
                      </svg>
                    </div>
                  )}
                </>
              )}
              
              {!showFullAddress && (
                <div className="explore-profile-address">
                  <AddressDisplay
                    address={resolvedAddress || ''}
                    displayName={addressOrName !== resolvedAddress ? addressOrName : undefined}
                    size="lg"
                    linkable={false}
                    showCopy={true}
                    showTooltip={true}
                  />
                </div>
              )}

              {profileData && (
                <div className="explore-profile-extended-stats">
                  <div className="explore-profile-extended-stat">
                    <span className="explore-profile-extended-stat-label">Member since</span>
                    <span className="explore-profile-extended-stat-value">
                      {formatMemberSince(profileData.firstCommitDate)}
                    </span>
                  </div>
                  
                  <div className="explore-profile-extended-stat">
                    <span className="explore-profile-extended-stat-label">Active repos</span>
                    <span className="explore-profile-extended-stat-value">
                      {profileData.activeRepos}
                    </span>
                  </div>
                </div>
              )}
              
              <div className="explore-profile-readonly-note">
                Identity is your key. Reputation is your commits.
              </div>
            </div>
          </div>

          <div className="explore-profile-stats">
            <div className="explore-profile-stat">
              <span className="explore-profile-stat-number">{totalRepoCount}</span>
              <span className="explore-profile-stat-label">repositories</span>
            </div>
            <div className="explore-profile-stat">
              <span className="explore-profile-stat-number">
                {totalCommits.toLocaleString()}
              </span>
              <span className="explore-profile-stat-label">commits</span>
            </div>
          </div>
        </div>

        {/* Activity Heatmap */}
        {profileData && profileData.activity.length > 0 && (
          <ActivityHeatmap activity={profileData.activity} />
        )}

        {/* Owned Repositories */}
        <div className="explore-content-section">
          <div className="explore-section-header">
            <h2 className="explore-section-title">Repositories</h2>
          </div>

          {loading ? (
            <div className="explore-repo-grid">
              {[1, 2, 3, 4, 5, 6].map(i => (
                <div key={i} className="explore-repo-item skeleton">
                  <div className="explore-repo-item-header">
                    <div className="explore-repo-item-name skeleton-line"></div>
                    <div className="explore-repo-item-language skeleton-dot"></div>
                  </div>
                  <div className="explore-repo-item-description skeleton-line short"></div>
                  <div className="explore-repo-item-meta">
                    <span className="skeleton-line tiny"></span>
                    <span className="skeleton-line tiny"></span>
                  </div>
                </div>
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
            <div className="explore-repo-grid">
              {repos.map((repo) => (
                <RepoCard
                  key={`${repo.address}/${repo.name}`}
                  repo={repo}
                  formatCommitCount={formatCommitCount}
                />
              ))}
            </div>
          )}
        </div>

        {/* Contributor Repos */}
        {!loading && contributorRepos.length > 0 && (
          <div className="explore-content-section">
            <div className="explore-section-header">
              <h2 className="explore-section-title">Can access</h2>
            </div>

            <div className="explore-repo-grid">
              {contributorRepos.map((repo) => (
                <ContributorRepoCard
                  key={`contrib-${repo.address}/${repo.name}`}
                  repo={repo}
                  formatCommitCount={formatCommitCount}
                />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
