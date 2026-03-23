'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { formatTimeAgo, formatAddress } from '@/lib/utils';
import AddressDisplay from '@/components/AddressDisplay';
import { repoUrls } from '@/lib/repoUrls';
import { SiteNav } from '@/components/SiteNav';

type DiffLine = {
  type: 'context' | 'addition' | 'deletion';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
};

type DiffHunk = {
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: DiffLine[];
};

type FileChange = {
  path: string;
  status: 'added' | 'modified' | 'deleted' | 'renamed';
  oldPath?: string;
  additions: number;
  deletions: number;
  hunks: DiffHunk[];
};

interface CommitDetails {
  hash: string;
  shortHash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
  parentHash: string | null;
  childHash: string | null;
  fileChanges: FileChange[];
  stats: { additions: number; deletions: number; filesChanged: number };
  signer?: string | null;
}

function commitSignerAddress(commit: CommitDetails): string | null {
  if (commit.signer && /^0x[a-fA-F0-9]{40}$/.test(commit.signer)) return commit.signer;
  if (/^0x[a-fA-F0-9]{40}$/.test(commit.author)) return commit.author;
  return null;
}

function lineBg(type: DiffLine['type']) {
  if (type === 'addition') return 'rgba(46, 160, 67, 0.14)';
  if (type === 'deletion') return 'rgba(248, 81, 73, 0.14)';
  return 'transparent';
}

function linePrefix(type: DiffLine['type']) {
  if (type === 'addition') return '+';
  if (type === 'deletion') return '-';
  return ' ';
}

export default function CommitPage() {
  const params = useParams();
  const [commit, setCommit] = useState<CommitDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const hash = Array.isArray(params.hash) ? params.hash[0] : params.hash;

  useEffect(() => {
    if (!address || !name || !hash) return;

    fetch(`/api/explorer/repos/${address}/${name}/commits/${hash}`)
      .then(async (r) => {
        if (!r.ok) throw new Error((await r.json().catch(() => null))?.error || 'Commit not found');
        return r.json();
      })
      .then((d) => setCommit(d))
      .catch((e) => setError(e instanceof Error ? e.message : 'Failed to load commit'))
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
          <Link href={`/${address}`} style={{ color: 'var(--bp-accent)', textDecoration: 'none' }}>{formatAddress(address)}</Link>
          <span>/</span>
          <Link href={repoUrls.home(address, name)} style={{ color: 'var(--bp-accent)', textDecoration: 'none' }}>{name}</Link>
          <span>/</span>
          <span style={{ color: 'var(--bp-heading)', fontWeight: 600 }}>commit {hash.slice(0, 7)}</span>
        </div>
      </div>

      <div style={{ maxWidth: 1280, margin: '0 auto', padding: '24px 32px 40px' }}>
        {loading ? (
          <p style={{ color: 'var(--bp-dim)' }}>Loading commit…</p>
        ) : error ? (
          <div style={{ background: 'var(--bp-surface)', border: '1px solid var(--bp-border)', borderRadius: 8, padding: 20 }}>
            <h3 style={{ margin: '0 0 8px', color: 'var(--bp-heading)' }}>Commit not found</h3>
            <p style={{ margin: 0, color: 'var(--bp-dim)' }}>{error}</p>
          </div>
        ) : commit ? (
          <>
            <div style={{ background: 'var(--bp-surface)', border: '1px solid var(--bp-border)', borderRadius: 8, padding: 20, marginBottom: 16 }}>
              <h1 style={{ fontSize: 18, margin: 0, color: 'var(--bp-heading)', lineHeight: 1.4 }}>{commit.message.split('\n')[0]}</h1>
              {commit.message.split('\n').length > 1 && (
                <pre style={{ margin: '10px 0 0', whiteSpace: 'pre-wrap', color: 'var(--bp-dim)', fontSize: 12, lineHeight: 1.5 }}>
                  {commit.message.split('\n').slice(1).join('\n').trim()}
                </pre>
              )}

              <div style={{ display: 'flex', alignItems: 'center', gap: 14, flexWrap: 'wrap', marginTop: 14, color: 'var(--bp-dim)' }}>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
                  {commitSignerAddress(commit) ? (
                    <AddressDisplay address={commitSignerAddress(commit)!} size="sm" showCopy={false} linkable={true} />
                  ) : (
                    <code style={{ color: 'var(--bp-gold)' }}>unsigned-commit</code>
                  )}
                  <span>committed</span>
                </span>
                <span>{formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}</span>
                <code style={{ color: 'var(--bp-accent)' }}>{commit.hash}</code>
              </div>

              <div style={{ display: 'flex', gap: 12, marginTop: 10, fontSize: 12, color: 'var(--bp-dim)' }}>
                <span>{commit.stats.filesChanged} files changed</span>
                <span style={{ color: '#3fb950' }}>+{commit.stats.additions}</span>
                <span style={{ color: '#f85149' }}>-{commit.stats.deletions}</span>
              </div>
            </div>

            {commit.fileChanges.length === 0 ? (
              <div style={{ background: 'var(--bp-surface)', border: '1px solid var(--bp-border)', borderRadius: 8, padding: 16, color: 'var(--bp-dim)' }}>
                No diff available for this commit.
              </div>
            ) : (
              <div style={{ display: 'grid', gap: 16 }}>
                {commit.fileChanges.map((file) => (
                  <section key={`${file.path}-${file.status}`} style={{ background: 'var(--bp-surface)', border: '1px solid var(--bp-border)', borderRadius: 8, overflow: 'hidden' }}>
                    <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '10px 14px', borderBottom: '1px solid var(--bp-border)', background: 'rgba(0,0,0,0.14)' }}>
                      <div style={{ display: 'flex', gap: 8, alignItems: 'center', minWidth: 0 }}>
                        <code style={{ color: 'var(--bp-heading)', fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis' }}>{file.path}</code>
                        <span style={{ fontSize: 10, padding: '2px 6px', borderRadius: 999, border: '1px solid var(--bp-border)', color: 'var(--bp-dim)' }}>{file.status}</span>
                      </div>
                      <div style={{ display: 'flex', gap: 8, fontSize: 12 }}>
                        <span style={{ color: '#3fb950' }}>+{file.additions}</span>
                        <span style={{ color: '#f85149' }}>-{file.deletions}</span>
                      </div>
                    </header>

                    <div style={{ overflowX: 'auto' }}>
                      {file.hunks.map((h, idx) => (
                        <div key={`${file.path}-h-${idx}`}>
                          <div style={{ padding: '6px 10px', fontSize: 11, color: 'var(--bp-dim)', borderBottom: '1px solid rgba(255,255,255,0.05)', background: 'rgba(31,111,235,0.08)' }}>
                            @@ -{h.oldStart},{h.oldCount} +{h.newStart},{h.newCount} @@
                          </div>
                          {h.lines.map((line, lidx) => (
                            <div key={`${file.path}-l-${idx}-${lidx}`} style={{ display: 'grid', gridTemplateColumns: '56px 56px 1fr', gap: 10, padding: '2px 10px', fontSize: 12, lineHeight: 1.5, background: lineBg(line.type), borderBottom: '1px solid rgba(255,255,255,0.03)' }}>
                              <span style={{ color: 'var(--bp-dim)', textAlign: 'right' }}>{line.oldLineNumber ?? ''}</span>
                              <span style={{ color: 'var(--bp-dim)', textAlign: 'right' }}>{line.newLineNumber ?? ''}</span>
                              <code style={{ whiteSpace: 'pre', color: 'var(--bp-text)' }}>{linePrefix(line.type)}{line.content}</code>
                            </div>
                          ))}
                        </div>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
            )}
          </>
        ) : null}
      </div>
    </div>
  );
}
