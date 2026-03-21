'use client';

import React, { useState } from 'react';

interface KeyboardShortcutsProps {
  hasParent: boolean;
  hasChild: boolean;
}

export default function KeyboardShortcuts({ hasParent, hasChild }: KeyboardShortcutsProps) {
  const [isVisible, setIsVisible] = useState(false);

  const shortcuts = [
    { key: 'P', description: 'Previous commit', disabled: !hasParent },
    { key: 'N', description: 'Next commit', disabled: !hasChild },
    { key: 'B', description: 'Back to repository', disabled: false },
    { key: 'C', description: 'Copy commit hash', disabled: false },
    { key: 'ESC', description: 'Back to repository', disabled: false },
    { key: '?', description: 'Toggle this help', disabled: false },
  ];

  return (
    <>
      <button
        className="keyboard-shortcuts-toggle"
        onClick={() => setIsVisible(!isVisible)}
        title="Keyboard shortcuts (?)"
      >
        ?
      </button>
      
      {isVisible && (
        <div className="keyboard-shortcuts-overlay" onClick={() => setIsVisible(false)}>
          <div className="keyboard-shortcuts-panel" onClick={(e) => e.stopPropagation()}>
            <div className="keyboard-shortcuts-header">
              <h3>Keyboard Shortcuts</h3>
              <button 
                className="keyboard-shortcuts-close"
                onClick={() => setIsVisible(false)}
              >
                ×
              </button>
            </div>
            <div className="keyboard-shortcuts-list">
              {shortcuts.map((shortcut, index) => (
                <div 
                  key={index} 
                  className={`keyboard-shortcut ${shortcut.disabled ? 'disabled' : ''}`}
                >
                  <kbd className="keyboard-shortcut-key">{shortcut.key}</kbd>
                  <span className="keyboard-shortcut-description">{shortcut.description}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </>
  );
}