'use client';

import { useState } from 'react';

interface CloneUrlWidgetProps {
  ownerAddress: string;
  repoName: string;
  className?: string;
}

// Helper functions
function generateCloneUrl(ownerAddress: string, repoName: string) {
  return `https://git.repo.box/${ownerAddress}/${repoName}.git`;
}

async function copyToClipboard(text: string): Promise<void> {
  if (navigator.clipboard) {
    await navigator.clipboard.writeText(text);
  } else {
    // Fallback for older browsers
    const textarea = document.createElement('textarea');
    textarea.value = text;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
  }
}

export default function CloneUrlWidget({ 
  ownerAddress, 
  repoName, 
  className 
}: CloneUrlWidgetProps) {
  const httpsUrl = generateCloneUrl(ownerAddress, repoName);
  const [expandedHelp, setExpandedHelp] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await copyToClipboard(httpsUrl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`clone-url-widget ${className || ''}`}>
      {/* Widget Title */}
      <div className="clone-url-widget-title">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
          <polyline points="3.27,6.96 12,12.01 20.73,6.96"></polyline>
          <line x1="12" y1="22.08" x2="12" y2="12"></line>
        </svg>
        Clone Repository
      </div>

      {/* HTTPS URL */}
      <div className="clone-url-group">
        <div className="clone-url-label">HTTPS (EVM-authenticated)</div>
        <div className="clone-url-input-container">
          <input 
            type="text" 
            value={httpsUrl}
            readOnly
            className="clone-url-input"
          />
          <button 
            onClick={handleCopy}
            className={`clone-url-copy-btn ${copied ? 'copied' : ''}`}
            title="Copy HTTPS clone URL"
          >
            {copied ? (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="20,6 9,17 4,12"></polyline>
              </svg>
            ) : (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
            )}
          </button>
        </div>
      </div>



      {/* Expandable Help Section */}
      <button 
        className="clone-help-toggle"
        onClick={() => setExpandedHelp(!expandedHelp)}
        aria-expanded={expandedHelp}
      >
        <svg 
          className="clone-help-chevron" 
          width="14" 
          height="14" 
          viewBox="0 0 24 24" 
          fill="none" 
          stroke="currentColor" 
          strokeWidth="2"
        >
          <polyline points="6,9 12,15 18,9"></polyline>
        </svg>
        Need authenticated access? Set up repobox
      </button>

      {/* Help Content */}
      {expandedHelp && (
        <div className="clone-help-content">
          {/* Step 1: Install repobox CLI */}
          <div className="clone-help-step">
            <div className="clone-help-step-title">1. Install repobox CLI</div>
            <div className="clone-help-code">
              curl -sSf https://repo.box/install.sh | sh
              <button 
                className="clone-help-code-copy"
                onClick={() => copyToClipboard('curl -sSf https://repo.box/install.sh | sh')}
                title="Copy install command"
              >
                Copy
              </button>
            </div>
          </div>

          {/* Step 2: Generate EVM identity */}
          <div className="clone-help-step">
            <div className="clone-help-step-title">2. Generate EVM identity</div>
            <div className="clone-help-code">
              $ repobox keys generate
            </div>
          </div>

          {/* Step 3: Configure git credential helper */}
          <div className="clone-help-step">
            <div className="clone-help-step-title">3. Configure git credential helper</div>
            <div className="clone-help-code">
              $ git config --global credential.helper \{'\n    '}"!repobox credential-helper"
              <button 
                className="clone-help-code-copy"
                onClick={() => copyToClipboard('git config --global credential.helper "!repobox credential-helper"')}
                title="Copy git config command"
              >
                Copy
              </button>
            </div>
          </div>

          {/* Step 4: Clone */}
          <div className="clone-help-step">
            <div className="clone-help-step-title">4. Clone with automatic authentication</div>
            <div className="clone-help-code">
              $ git clone [URL from above]
            </div>
          </div>

          {/* Info Box */}
          <div className="clone-help-info">
            Your EVM identity will be used for push authentication. No passwords required - everything is cryptographic signatures.
          </div>
        </div>
      )}
    </div>
  );
}