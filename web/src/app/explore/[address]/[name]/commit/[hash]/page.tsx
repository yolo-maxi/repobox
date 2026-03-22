'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import { repoUrls } from '@/lib/repoUrls';
import { SiteNav } from '@/components/SiteNav';

interface CommitDetails {
  hash: string; author: string; email: string;
  timestamp: number; message: string;
}

export default function CommitPage() {
  const params = useParams();
  const [commit, setCommit] = useState<CommitDetails | null>(null);
  const [loading, setLoading] = useState(true);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const hash = Array.isArray(params.hash) ? params.hash[0] : params.hash;

  useEffect(() => {
    if (!address || !name || !hash) return;
    // Try to find the commit in the commits list
    fetch(`/api/explorer/repos/${address}/${name}/commits?limit=100`)
      .then(r => r.ok ? r.json() : null)
      .then(d => {
        const found = (d?.commits || []).find((c: any) => c.hash === hash || c.hash.startsWith(hash));
        if (found) setCommit(found);
        else setCommit({ hash, author: 'Unknown', email: '', timestamp: Date.now() / 1000, message: 'Commit details unavailable' });
      })
      .catch(() => setCommit({ hash, author: 'Unknown', email: '', timestamp: Date.now() / 1000, message: 'Commit details unavailable' }))
      .finally(() => setLoading(false));
  }, [address, name, hash]);

  if (!address || !name || !hash) return notFound();

  return (
    <div style={{ minHeight: '100vh', background: 'var(--bp-bg)', color: 'var(--bp-text)', fontFamily: 'var(--font-mono, monospace)', fontSize: 13 }}>
      <SiteNav />
      <div style={{ borderBottom: '1px solid var(--bp-border)', padding: '16px 32px' }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--bp-dim)', flexWrap: 'wrap' }}>
          <Link href="/explore" style={{ color: 'var(--bp-accent)', textDecoration: 'none' }}>explore</Link>
          <span>/</span>
          <Link href={`/explore/${address}`} style={{ color: 'var(--bp-accent)', textDecoration: 'none' }}>{formatAddress(address)}</Link>
          <span>/</span>
          <Link href={repoUrls.home(address, name)} style={{ color: 'var(--bp-accent)', textDecoration: 'none' }}>{name}</Link>
          <span>/</span>
          <span style={{ color: 'var(--bp-heading)', fontWeight: 600 }}>commit {hash.slice(0, 7)}</span>
        </div>
      </div>

      <div style={{ maxWidth: 1280, margin: '0 auto', padding: '32px 32px' }}>
        {loading ? (
          <p style={{ color: 'var(--bp-dim)' }}>Loading…</p>
        ) : commit ? (
          <>
            <div style={{
              background: 'var(--bp-surface)', border: '1px solid var(--bp-border)',
              borderRadius: 8, padding: 24, marginBottom: 24,
            }}>
              <h1 style={{ fontSize: 18, fontWeight: 600, color: 'var(--bp-heading)', marginBottom: 12, lineHeight: 1.4 }}>
                {commit.message.split('\n')[0]}
              </h1>
              {commit.message.split('\n').length > 1 && (
                <pre style={{ color: 'var(--bp-dim)', fontSize: 12, margin: '12px 0 0', whiteSpace: 'pre-wrap', lineHeight: 1.5 }}>
                  {commit.message.split('\n').slice(1).join('\n').trim()}
                </pre>
              )}
              <div style={{ display: 'flex', gap: 16, marginTop: 16, fontSize: 12, color: 'var(--bp-dim)', flexWrap: 'wrap', alignItems: 'center' }}>
                <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                  <AddressDisplay address={address} size="sm" showCopy={false} linkable={true} />
                  <span>committed</span>
                </span>
                <span>{formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}</span>
              </div>
            </div>

            <div style={{
              background: 'var(--bp-surface)', border: '1px solid var(--bp-border)',
              borderRadius: 8, padding: 20,
            }}>
              <div style={{ fontSize: 10, fontWeight: 600, textTransform: 'uppercase' as const, letterSpacing: '0.1em', color: 'var(--bp-dim)', marginBottom: 12 }}>Commit Hash</div>
              <code style={{ fontSize: 12, color: 'var(--bp-accent)', wordBreak: 'break-all' as const }}>{commit.hash}</code>
            </div>

            <div style={{ display: 'flex', gap: 8, marginTop: 20 }}>
              <Link href={repoUrls.home(address, name)} style={{
                padding: '8px 16px', border: '1px solid var(--bp-border)', borderRadius: 6,
                color: 'var(--bp-dim)', fontSize: 12, textDecoration: 'none',
              }}>← Repository</Link>
            </div>
          </>
        ) : null}
      </div>
    </div>
  );
}
