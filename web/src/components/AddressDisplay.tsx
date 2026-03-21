'use client';

import { useState, useEffect } from 'react';
import { formatAddress, copyToClipboard } from '@/lib/utils';
import { resolveAddressDisplay } from '@/lib/addressResolver';
import Link from 'next/link';

export interface AddressDisplayProps {
  address: string;
  /** Override display name (e.g., from ENS resolution) */
  displayName?: string;
  /** Show copy button */
  showCopy?: boolean;
  /** Enable click-to-copy */
  clickToCopy?: boolean;
  /** Show hover tooltip with full address */
  showTooltip?: boolean;
  /** Custom CSS classes */
  className?: string;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Link to address page */
  linkable?: boolean;
  /** Custom link destination */
  href?: string;
}

export default function AddressDisplay({
  address,
  displayName,
  showCopy = true,
  clickToCopy = true,
  showTooltip = true,
  className = '',
  size = 'md',
  linkable = true,
  href
}: AddressDisplayProps) {
  const [resolvedName, setResolvedName] = useState<string | null>(displayName || null);
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(false);

  // Resolve display name (ENS or subdomain)
  useEffect(() => {
    if (displayName) return; // Use provided displayName
    
    const resolveDisplay = async () => {
      setLoading(true);
      try {
        const resolved = await resolveAddressDisplay(address);
        setResolvedName(resolved);
      } catch (error) {
        console.warn('Failed to resolve address display:', error);
      } finally {
        setLoading(false);
      }
    };
    
    resolveDisplay();
  }, [address, displayName]);

  const handleCopy = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    try {
      await copyToClipboard(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Copy failed:', error);
    }
  };

  const handleClick = async (e: React.MouseEvent) => {
    if (clickToCopy && !href && !linkable) {
      await handleCopy(e);
    }
  };

  // Display logic: resolved name > formatted address
  const displayText = resolvedName || formatAddress(address);
  const isResolved = !!resolvedName;
  
  // Link logic
  const linkHref = href || (linkable ? `/explore/${address}` : undefined);

  const addressElement = (
    <span
      onClick={handleClick}
      className={`address-display ${className} address-display--${size} ${
        isResolved ? 'address-display--resolved' : 'address-display--truncated'
      } ${clickToCopy && !linkable ? 'address-display--clickable' : ''} ${
        loading ? 'address-display--loading' : ''
      }`}
      title={showTooltip ? `${address}${isResolved ? ` (${displayText})` : ''}` : undefined}
    >
      <code className="address-display__text">
        {loading ? (
          <span className="address-display__loading">
            <span className="address-display__spinner" />
            {formatAddress(address)}
          </span>
        ) : (
          displayText
        )}
      </code>
      
      {showCopy && (
        <button
          onClick={handleCopy}
          className="address-display__copy-btn"
          title="Copy address"
          aria-label="Copy address to clipboard"
        >
          {copied ? (
            <svg className="address-display__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="20,6 9,17 4,12" />
            </svg>
          ) : (
            <svg className="address-display__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
          )}
        </button>
      )}
    </span>
  );

  if (linkHref) {
    return (
      <Link href={linkHref} className="address-display-link">
        {addressElement}
      </Link>
    );
  }

  return addressElement;
}