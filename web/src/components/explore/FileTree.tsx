'use client';

import { useState, useCallback } from 'react';
import { formatBytes } from '@/lib/utils';

interface FileEntry {
  type: 'blob' | 'tree';
  name: string;
  size?: number;
  path: string;
}

interface FileTreeProps {
  address: string;
  repoName: string;
  branch: string;
  initialFiles: FileEntry[];
  onFileClick: (path: string) => void;
}

// File type icon mapping
function fileIcon(name: string, isDir: boolean): string {
  if (isDir) return '📁';
  const ext = name.split('.').pop()?.toLowerCase() || '';
  const map: Record<string, string> = {
    rs: '🦀', ts: '🟦', tsx: '🟦', js: '🟨', jsx: '🟨',
    json: '📋', yml: '⚙️', yaml: '⚙️', toml: '⚙️',
    md: '📝', txt: '📄', sh: '🔧', lock: '🔒',
    css: '🎨', html: '🌐', svg: '🖼️', png: '🖼️', jpg: '🖼️',
    sol: '💎', env: '🔐', gitignore: '👁️',
  };
  return map[ext] || '📄';
}

function TreeNode({
  entry,
  depth,
  address,
  repoName,
  branch,
  onFileClick,
}: {
  entry: FileEntry;
  depth: number;
  address: string;
  repoName: string;
  branch: string;
  onFileClick: (path: string) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [children, setChildren] = useState<FileEntry[] | null>(null);
  const [loading, setLoading] = useState(false);

  const toggle = useCallback(async () => {
    if (entry.type !== 'tree') {
      onFileClick(entry.path);
      return;
    }

    if (expanded) {
      setExpanded(false);
      return;
    }

    if (children) {
      setExpanded(true);
      return;
    }

    // Fetch subdirectory
    setLoading(true);
    try {
      const branchParam = branch !== 'HEAD' ? `&branch=${branch}` : '';
      const res = await fetch(
        `/api/explorer/repos/${address}/${repoName}/tree?path=${encodeURIComponent(entry.path)}${branchParam}`
      );
      if (res.ok) {
        const data = await res.json();
        const sorted = (data.files || []).sort((a: FileEntry, b: FileEntry) => {
          if (a.type !== b.type) return a.type === 'tree' ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
        setChildren(sorted);
        setExpanded(true);
      }
    } catch (e) {
      console.error('Failed to load tree:', e);
    } finally {
      setLoading(false);
    }
  }, [entry, expanded, children, address, repoName, branch, onFileClick]);

  const isDir = entry.type === 'tree';

  return (
    <>
      <button onClick={toggle} className="ft-row" style={{ paddingLeft: 16 + depth * 20 }}>
        {isDir && (
          <span className={`ft-chevron ${expanded ? 'open' : ''}`}>
            {loading ? '⟳' : '▶'}
          </span>
        )}
        {!isDir && <span className="ft-chevron-spacer" />}
        <span className="ft-icon">{fileIcon(entry.name, isDir)}</span>
        <span className={`ft-name ${isDir ? 'dir' : ''}`}>{entry.name}</span>
        {!isDir && entry.size !== undefined && (
          <span className="ft-size">{formatBytes(entry.size)}</span>
        )}
      </button>
      {expanded && children && children.map((child, i) => (
        <TreeNode
          key={child.path}
          entry={child}
          depth={depth + 1}
          address={address}
          repoName={repoName}
          branch={branch}
          onFileClick={onFileClick}
        />
      ))}
    </>
  );
}

export default function FileTree({ address, repoName, branch, initialFiles, onFileClick }: FileTreeProps) {
  const sorted = [...initialFiles].sort((a, b) => {
    if (a.type !== b.type) return a.type === 'tree' ? -1 : 1;
    return a.name.localeCompare(b.name);
  });

  return (
    <div className="ft-root">
      {sorted.length === 0 ? (
        <div className="ft-empty">No files</div>
      ) : (
        sorted.map((entry) => (
          <TreeNode
            key={entry.path}
            entry={entry}
            depth={0}
            address={address}
            repoName={repoName}
            branch={branch}
            onFileClick={onFileClick}
          />
        ))
      )}

      <style>{`
        .ft-root {
          border: 1px solid var(--bp-border);
          border-radius: 8px;
          overflow: hidden;
        }
        .ft-row {
          display: flex;
          align-items: center;
          width: 100%;
          padding: 8px 16px;
          background: none;
          border: none;
          border-bottom: 1px solid var(--bp-border);
          cursor: pointer;
          font-family: inherit;
          font-size: 13px;
          color: var(--bp-text);
          text-align: left;
          transition: background 0.1s;
          gap: 6px;
        }
        .ft-row:last-child { border-bottom: none; }
        .ft-row:hover { background: rgba(79, 195, 247, 0.03); }

        .ft-chevron {
          width: 14px;
          font-size: 8px;
          color: var(--bp-dim);
          transition: transform 0.15s;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }
        .ft-chevron.open { transform: rotate(90deg); }
        .ft-chevron-spacer { width: 14px; flex-shrink: 0; }

        .ft-icon {
          width: 18px;
          text-align: center;
          flex-shrink: 0;
          font-size: 14px;
        }
        .ft-name {
          flex: 1;
          min-width: 0;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        .ft-name.dir {
          color: var(--bp-accent);
          font-weight: 500;
        }
        .ft-size {
          color: var(--bp-dim);
          font-size: 11px;
          opacity: 0.6;
          flex-shrink: 0;
          margin-left: auto;
        }
        .ft-empty {
          padding: 32px;
          text-align: center;
          color: var(--bp-dim);
        }
      `}</style>
    </div>
  );
}
