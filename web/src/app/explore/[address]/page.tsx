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
  const [ensName, setEnsName] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;

  if (!address) {
    return (
      <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)] p-8">
        <div className="max-w-6xl mx-auto">
          <div className="text-center py-20">
            <div className="text-[var(--bp-dim)]">Invalid address</div>
            <Link href="/explore" className="text-[var(--bp-accent)] hover:opacity-80 text-sm mt-4 inline-block">
              ← Back to explorer
            </Link>
          </div>
        </div>
      </div>
    );
  }

  useEffect(() => {

    const fetchRepos = async () => {
      try {
        // Fetch all repos and filter by owner
        const res = await fetch('/api/explorer/repos?limit=1000');
        if (res.ok) {
          const data = await res.json();
          const ownerRepos = data.repos.filter((repo: Repo) => 
            repo.owner_address.toLowerCase() === address.toLowerCase()
          );
          setRepos(ownerRepos);
        }

        // Try to resolve ENS name (simplified)
        try {
          const ensRes = await fetch(`/api/explorer/resolve/${address}`);
          if (ensRes.ok) {
            const ensData = await ensRes.json();
            setEnsName(ensData.name);
          }
        } catch {
          // ENS resolution failed, that's ok
        }
      } catch (error) {
        console.error('Error fetching repos:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchRepos();
  }, [address]);

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

  return (
    <div className="min-h-screen bg-[var(--bp-bg)] text-[var(--bp-text)]">
      {/* Header */}
      <header className="border-b border-[var(--bp-border)] bg-[var(--bp-surface)]">
        <div className="max-w-6xl mx-auto px-8 py-6">
          <div className="flex items-center justify-between">
            <div>
              <div className="flex items-center space-x-4 mb-2">
                <Link href="/explore" className="text-[var(--bp-accent)] hover:opacity-80">
                  ← Explorer
                </Link>
              </div>
              
              <div className="flex items-center space-x-4">
                <h1 className="text-2xl font-bold text-[var(--bp-heading)] font-mono">
                  {formatAddress(address)}
                </h1>
                <button
                  onClick={handleCopyAddress}
                  className="text-sm text-[var(--bp-dim)] hover:text-[var(--bp-accent)] border border-[var(--bp-border)] px-3 py-1 rounded"
                >
                  {copied ? 'Copied!' : 'Copy full address'}
                </button>
                {ensName && (
                  <span className="text-[var(--bp-accent)] text-lg">
                    {ensName}
                  </span>
                )}
              </div>
              
              <p className="text-[var(--bp-dim)] text-sm mt-2">
                {repos.length} repositories
              </p>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-8 py-8">
        {repos.length === 0 ? (
          <div className="bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-12 text-center">
            <div className="text-[var(--bp-dim)] text-lg mb-2">No repositories found</div>
            <div className="text-[var(--bp-dim)] text-sm">
              This address hasn't pushed any repositories yet.
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {repos.map((repo) => (
              <Link
                key={`${repo.address}/${repo.name}`}
                href={`/explore/${repo.address}/${repo.name}`}
                className="block bg-[var(--bp-surface)] border border-[var(--bp-border)] rounded-lg p-6 hover:border-[var(--bp-accent)] transition-colors"
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <h3 className="font-bold text-[var(--bp-heading)] text-lg mb-2">
                      {repo.name}
                    </h3>
                    
                    {repo.description && (
                      <p className="text-[var(--bp-text)] text-sm mb-3 line-clamp-2">
                        {repo.description}
                      </p>
                    )}
                    
                    <div className="flex items-center space-x-4 text-xs text-[var(--bp-dim)]">
                      <span>{repo.commit_count} commits</span>
                      <span>Updated {formatTimeAgo(repo.last_commit_date)}</span>
                      <span>Created {formatTimeAgo(repo.created_at)}</span>
                    </div>
                  </div>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}