'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { CommitDetail } from '@/lib/git';
import { formatTimeAgo, formatAddress, copyToClipboard } from '@/lib/utils';
import FileChangeList from '@/components/FileChangeList';
import KeyboardShortcuts from '@/components/KeyboardShortcuts';
import ErrorBoundary from '@/components/ErrorBoundary';

export default function CommitDetailPage() {
  const params = useParams();
  const [commit, setCommit] = useState<CommitDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());
  const [hashCopied, setHashCopied] = useState(false);
  const [showKeyboardHelp, setShowKeyboardHelp] = useState(false);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const hash = Array.isArray(params.hash) ? params.hash[0] : params.hash;

  useEffect(() => {
    if (!address || !name || !hash) return;
    
    const fetchCommitDetail = async () => {
      try {
        setLoading(true);
        setError(null);
        
        const res = await fetch(`/api/explorer/repos/${address}/${name}/commits/${hash}`);
        
        if (!res.ok) {
          if (res.status === 404) {
            setError('Commit not found');
          } else {
            setError('Failed to load commit details');
          }
          return;
        }
        
        const data = await res.json();
        setCommit(data);
      } catch (err) {
        setError('Failed to load commit details');
        console.error('Error:', err);
      } finally {
        setLoading(false);
      }
    };
    
    fetchCommitDetail();
  }, [address, name, hash]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!commit) return;
      
      // Ignore if user is typing in an input field
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }
      
      switch (event.key.toLowerCase()) {
        case 'p':
          // Previous commit
          if (commit.parentHash) {
            window.location.href = `/explore/${address}/${name}/commit/${commit.parentHash}`;
          }
          break;
        case 'n':
          // Next commit
          if (commit.childHash) {
            window.location.href = `/explore/${address}/${name}/commit/${commit.childHash}`;
          }
          break;
        case 'b':
          // Back to repository
          window.location.href = `/explore/${address}/${name}`;
          break;
        case 'c':
          // Copy commit hash
          handleCopyHash(commit.hash);
          break;
        case 'escape':
          // Back to repository
          window.location.href = `/explore/${address}/${name}`;
          break;
        case '?':
          // Toggle keyboard shortcuts help
          setShowKeyboardHelp(!showKeyboardHelp);
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [commit, address, name]);

  const toggleFileExpansion = (filePath: string) => {
    const newExpanded = new Set(expandedFiles);
    if (newExpanded.has(filePath)) {
      newExpanded.delete(filePath);
    } else {
      newExpanded.add(filePath);
    }
    setExpandedFiles(newExpanded);
  };

  const handleCopyHash = async (hashToCopy: string) => {
    await copyToClipboard(hashToCopy);
    setHashCopied(true);
    setTimeout(() => setHashCopied(false), 2000);
  };

  if (!address || !name || !hash) return null;

  if (loading) {
    return (
      <div className="explore-page">
        <div className="explore-loading">
          <div className="explore-loading-spinner"></div>
          <p>Loading commit details...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="explore-page">
        <div className="explore-empty">
          <h3>{error}</h3>
          <Link href={`/explore/${address}/${name}`} className="explore-back-link">
            ← Back to Repository
          </Link>
        </div>
      </div>
    );
  }

  if (!commit) {
    return (
      <div className="explore-page">
        <div className="explore-empty">
          <h3>Commit not found</h3>
          <Link href={`/explore/${address}/${name}`} className="explore-back-link">
            ← Back to Repository
          </Link>
        </div>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <div className="explore-page">
      {/* Back Link */}
      <div className="explore-back">
        <Link href={`/explore/${address}/${name}`} className="explore-back-link">
          ← Back to Repository
        </Link>
      </div>

      {/* Commit Header */}
      <header className="commit-header">
        <div className="commit-message">
          <h1>{commit.message}</h1>
        </div>
        
        <div className="commit-metadata">
          <div className="commit-meta-row">
            <span className="commit-meta-label">Author:</span>
            <span className="commit-meta-value">{commit.author}</span>
          </div>
          
          <div className="commit-meta-row">
            <span className="commit-meta-label">Date:</span>
            <span className="commit-meta-value">
              {formatTimeAgo(new Date(commit.timestamp * 1000).toISOString())}
            </span>
          </div>
          
          <div className="commit-meta-row">
            <span className="commit-meta-label">Hash:</span>
            <button 
              onClick={() => handleCopyHash(commit.hash)}
              className="commit-hash-button"
              title="Click to copy full hash"
            >
              <code>{commit.shortHash}</code>
              <span className="commit-copy-hint">
                {hashCopied ? '✓ copied' : 'copy'}
              </span>
            </button>
          </div>
        </div>
      </header>

      {/* Commit Stats */}
      <div className="commit-stats">
        <div className="commit-stat-item">
          <span className="commit-stat-value commit-stat-additions">+{commit.stats.additions}</span>
          <span className="commit-stat-label">additions</span>
        </div>
        <div className="commit-stat-item">
          <span className="commit-stat-value commit-stat-deletions">-{commit.stats.deletions}</span>
          <span className="commit-stat-label">deletions</span>
        </div>
        <div className="commit-stat-item">
          <span className="commit-stat-value">{commit.stats.filesChanged}</span>
          <span className="commit-stat-label">files changed</span>
        </div>
      </div>

      {/* File Changes */}
      <div className="file-changes">
        <h3 className="file-changes-title">Changed Files</h3>
        <FileChangeList 
          changes={commit.fileChanges}
          expandedFiles={expandedFiles}
          onToggleExpand={toggleFileExpansion}
        />
      </div>

      {/* Navigation */}
      <div className="commit-navigation">
        {commit.parentHash && (
          <Link 
            href={`/explore/${address}/${name}/commit/${commit.parentHash}`}
            className="commit-nav-button commit-nav-parent"
          >
            ← Previous commit
          </Link>
        )}
        
        {commit.childHash && (
          <Link 
            href={`/explore/${address}/${name}/commit/${commit.childHash}`}
            className="commit-nav-button commit-nav-child"
          >
            Next commit →
          </Link>
        )}
      </div>

      {/* Keyboard Shortcuts Help */}
      <KeyboardShortcuts 
        hasParent={!!commit.parentHash}
        hasChild={!!commit.childHash}
      />
      </div>
    </ErrorBoundary>
  );
}