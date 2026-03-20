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

  useEffect(() => {
    if (!address) return;
    const fetchData = async () => {
      try {
        const res = await fetch(`/api/explorer/repos?owner=${address}`);
        if (res.ok) {
          const data = await res.json();
          setRepos(data.repos || []);
        }

        // Try ENS reverse lookup
        if (address.endsWith('.eth')) {
          const resolveRes = await fetch(`/api/explorer/resolve/${address}`);
          if (resolveRes.ok) {
            const data = await resolveRes.json();
            if (data.address) setEnsName(address);
          }
        }
      } catch (error) {
        console.error('Error:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [address]);

  if (!address) return null;

  return (
    <div className="p-6 md:p-10">
      <Link href="/explore" className="text-[var(--bp-accent)] text-sm hover:opacity-80 mb-6 inline-block">
        ← Explorer
      </Link>

      {/* Profile header */}
      <div className="mb-10">
        <div className="flex items-center gap-4 mb-3">
          <div className="w-14 h-14 rounded-full glass-stat flex items-center justify-center text-2xl">
            🔑
          </div>
          <div>
            {ensName && (
              <div className="text-lg font-bold text-[var(--bp-accent)] mb-0.5">{ensName}</div>
            )}
            <div className="flex items-center gap-2">
              <span className="text-sm font-mono text-[var(--bp-dim)]">{address}</span>
              <button
                onClick={async () => {
                  await copyToClipboard(address);
                  setCopied(true);
                  setTimeout(() => setCopied(false), 2000);
                }}
                className="glass-tab text-xs px-2 py-1 rounded"
              >
                {copied ? '✓' : 'Copy'}
              </button>
            </div>
          </div>
        </div>

        <div className="flex gap-4 mt-4">
          <div className="glass-stat rounded-lg px-4 py-2">
            <span className="text-lg font-bold text-[var(--bp-accent)] font-mono">{repos.length}</span>
            <span className="text-xs text-[var(--bp-dim)] ml-2">repos</span>
          </div>
          <div className="glass-stat rounded-lg px-4 py-2">
            <span className="text-lg font-bold text-[var(--bp-heading)] font-mono">
              {repos.reduce((sum, r) => sum + r.commit_count, 0)}
            </span>
            <span className="text-xs text-[var(--bp-dim)] ml-2">commits</span>
          </div>
        </div>
      </div>

      {/* Repos */}
      <h2 className="text-lg font-semibold text-[var(--bp-heading)] mb-5">Repositories</h2>

      {loading ? (
        <div className="text-center py-10">
          <div className="animate-pulse text-[var(--bp-accent)] text-sm">Loading...</div>
        </div>
      ) : repos.length === 0 ? (
        <div className="glass-panel-inner rounded-xl p-10 text-center">
          <div className="text-[var(--bp-dim)]">No repositories found for this address</div>
        </div>
      ) : (
        <div className="space-y-3">
          {repos.map((repo) => (
            <Link
              key={`${repo.address}/${repo.name}`}
              href={`/explore/${repo.address}/${repo.name}`}
              className="glass-card block rounded-xl p-5"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <h3 className="font-semibold text-[var(--bp-heading)] mb-1">{repo.name}</h3>
                  {repo.description && (
                    <p className="text-sm text-[var(--bp-text)] opacity-70 mb-2">{repo.description}</p>
                  )}
                  <div className="flex gap-4 text-xs text-[var(--bp-dim)] font-mono">
                    <span>{repo.commit_count} commits</span>
                    {repo.last_commit_date && <span>Updated {formatTimeAgo(repo.last_commit_date)}</span>}
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
