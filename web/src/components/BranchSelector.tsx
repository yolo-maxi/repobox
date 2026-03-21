'use client';

import { useState, useEffect, useRef } from 'react';

interface Branch {
  name: string;
  is_default: boolean;
  last_commit: {
    hash: string;
    timestamp: number;
    message: string;
  };
}

interface BranchSelectorProps {
  branches: Branch[];
  currentBranch: string;
  defaultBranch: string;
  onChange: (branch: string) => void;
  disabled?: boolean;
}

export default function BranchSelector({ 
  branches, 
  currentBranch, 
  defaultBranch, 
  onChange,
  disabled = false 
}: BranchSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [filter, setFilter] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setFilter('');
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Filter branches based on search
  const filteredBranches = branches.filter(branch =>
    branch.name.toLowerCase().includes(filter.toLowerCase())
  );

  const handleBranchSelect = (branchName: string) => {
    onChange(branchName);
    setIsOpen(false);
    setFilter('');
  };

  const currentBranchData = branches.find(b => b.name === currentBranch);

  return (
    <div className="branch-selector" ref={dropdownRef}>
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        className={`branch-selector-trigger ${disabled ? 'disabled' : ''}`}
        disabled={disabled}
      >
        <span className="branch-icon">🌿</span>
        <span className="branch-name">{currentBranch}</span>
        {currentBranchData?.is_default && (
          <span className="branch-default-badge">default</span>
        )}
        <span className={`branch-arrow ${isOpen ? 'open' : ''}`}>▼</span>
      </button>

      {isOpen && (
        <div className="branch-dropdown">
          {branches.length > 10 && (
            <div className="branch-filter">
              <input
                type="text"
                placeholder="Filter branches..."
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                className="branch-filter-input"
                autoFocus
              />
            </div>
          )}
          
          <div className="branch-list">
            {filteredBranches.length === 0 ? (
              <div className="branch-empty">No branches found</div>
            ) : (
              filteredBranches.map((branch) => (
                <button
                  key={branch.name}
                  onClick={() => handleBranchSelect(branch.name)}
                  className={`branch-item ${branch.name === currentBranch ? 'current' : ''}`}
                >
                  <div className="branch-item-info">
                    <span className="branch-item-name">
                      {branch.name === currentBranch && <span className="check">✓ </span>}
                      {branch.name}
                    </span>
                    {branch.is_default && (
                      <span className="branch-item-badge">default</span>
                    )}
                  </div>
                  <div className="branch-item-meta">
                    <span className="branch-last-commit">
                      {branch.last_commit.message.substring(0, 50)}
                      {branch.last_commit.message.length > 50 && '...'}
                    </span>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}