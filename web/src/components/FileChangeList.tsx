'use client';

import React from 'react';
import { FileChange } from '@/lib/git';
import DiffViewer from './DiffViewer';

interface FileChangeListProps {
  changes: FileChange[];
  expandedFiles: Set<string>;
  onToggleExpand: (filePath: string) => void;
}

export default function FileChangeList({ changes, expandedFiles, onToggleExpand }: FileChangeListProps) {
  if (changes.length === 0) {
    return (
      <div className="explore-empty">
        <p>No file changes in this commit</p>
      </div>
    );
  }

  return (
    <div className="file-change-list">
      {changes.map((change, index) => (
        <div key={index} className="file-change-item">
          <div 
            className="file-change-header"
            onClick={() => onToggleExpand(change.path)}
          >
            <div className="file-change-info">
              <span className={`file-change-status file-change-status-${change.status}`}>
                {change.status}
              </span>
              <span className="file-change-path">{change.path}</span>
              {change.oldPath && (
                <span className="file-change-old-path">← {change.oldPath}</span>
              )}
            </div>
            
            <div className="file-change-stats">
              {change.additions > 0 && (
                <span className="file-stat file-stat-additions">+{change.additions}</span>
              )}
              {change.deletions > 0 && (
                <span className="file-stat file-stat-deletions">-{change.deletions}</span>
              )}
              <span className="file-expand-icon">
                {expandedFiles.has(change.path) ? '▼' : '▶'}
              </span>
            </div>
          </div>
          
          {expandedFiles.has(change.path) && (
            <div className="diff-viewer">
              <DiffViewer hunks={change.hunks} filePath={change.path} />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}