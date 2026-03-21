'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress, copyToClipboard } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import { resolveNameToAddress } from '@/lib/addressResolver';

interface Repo {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
  commit_count: number;
  last_commit_date: string | null;
  description: string | null;
}

// Get language color from GitHub language colors
function getLanguageColor(fileName: string): string {
  const ext = fileName.split('.').pop()?.toLowerCase();
  const languageColors: { [key: string]: string } = {
    js: '#f1e05a', jsx: '#f1e05a', ts: '#3178c6', tsx: '#3178c6',
    py: '#3572a5', go: '#00add8', rs: '#dea584', java: '#b07219',
    c: '#555555', cpp: '#f34b7d', css: '#563d7c', html: '#e34c26',
    vue: '#41b883', php: '#4f5d95', rb: '#701516', swift: '#fa7343',
    kt: '#a97bff', scala: '#c22d40', sh: '#89e051', sql: '#e38c00'
  };
  return languageColors[ext || ''] || '#666666';
}

// Detect primary language from file list (simplified)
function detectLanguage(repoName: string): { name: string; color: string } {
  // Simple heuristic - could be enhanced with actual file analysis
  const languages = [
    { name: 'TypeScript', color: '#3178c6' },
    { name: 'JavaScript', color: '#f1e05a' },
    { name: 'Python', color: '#3572a5' },
    { name: 'Go', color: '#00add8' },
    { name: 'Rust', color: '#dea584' },
    { name: 'Java', color: '#b07219' },
  ];
  
  // Random selection for demo - in real app would analyze files
  return languages[Math.floor(Math.random() * languages.length)];
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

  // Resolve name to address if needed
  useEffect(() => {
    const resolveAddress = async () => {
      if (!addressOrName) return;
      
      // If it's already an address, use it directly
      if (/^0x[a-fA-F0-9]{40}$/i.test(addressOrName)) {
        setResolvedAddress(addressOrName);
        setResolving(false);
        return;
      }
      
      // Try to resolve name
      try {
        const resolved = await resolveNameToAddress(addressOrName);
        if (resolved) {
          setResolvedAddress(resolved);
          // Update URL to canonical address form (optional - keeps human readable URL)
          // router.replace(`/explore/${resolved}`);
        } else {
          // Name not found
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

  // Fetch repos data
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



  // Format commit count properly
  const formatCommitCount = (count: number): string => {
    if (count === 1) return '1 commit';
    return `${count} commits`;
  };

  if (!addressOrName) return null;

  if (resolving) {
    return (
      <div className="explore-page">
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
              <span className="explore-breadcrumb-current">Resolving...</span>
            </div>
          </div>
        </header>
        
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
              <span className="explore-breadcrumb-current">Not Found</span>
            </div>
          </div>
        </header>
        
        <div className="explore-main-content">
          <div className="explore-empty">
            <h3>Address not found</h3>
            <p>Could not resolve "{addressOrName}" to an address.</p>
            <Link href="/explore" className="explore-back-link">← Back to Explorer</Link>
          </div>
        </div>
      </div>
    );
  }

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
            <span className="explore-breadcrumb-current">Developer</span>
          </div>
        </div>
      </header>

      <div className="explore-main-content">
        {/* Address Header */}
        <div className="explore-profile-header">
          <div className="explore-profile-info">
            <div className="explore-profile-avatar">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
              </svg>
            </div>
            <div className="explore-profile-details">
              <h1 className="explore-profile-title">Developer</h1>
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
            </div>
          </div>

          <div className="explore-profile-stats">
            <div className="explore-profile-stat">
              <span className="explore-profile-stat-number">{repos.length}</span>
              <span className="explore-profile-stat-label">repositories</span>
            </div>
            <div className="explore-profile-stat">
              <span className="explore-profile-stat-number">
                {repos.reduce((sum, r) => sum + r.commit_count, 0).toLocaleString()}
              </span>
              <span className="explore-profile-stat-label">commits</span>
            </div>
          </div>
        </div>

        {/* Repositories */}
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
            <div className="explore-empty">
              <svg className="explore-empty-icon" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
              </svg>
              <h3>No repositories found</h3>
              <p>This developer hasn't published any repositories yet</p>
            </div>
          ) : (
            <div className="explore-repo-grid">
              {repos.map((repo) => {
                const language = detectLanguage(repo.name);
                const isRecentlyActive = repo.last_commit_date && 
                  new Date(repo.last_commit_date).getTime() > Date.now() - (7 * 24 * 60 * 60 * 1000);
                
                return (
                  <Link
                    key={`${repo.address}/${repo.name}`}
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
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}