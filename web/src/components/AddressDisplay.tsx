'use client';

import { useState, useEffect } from 'react';
import { formatAddress, copyToClipboard } from '@/lib/utils';
import { resolveIdentity, type ResolvedIdentity, type IdentityTier } from '@/lib/addressResolver';
import Link from 'next/link';

export interface AddressDisplayProps {
  address: string;
  /** Override display name */
  displayName?: string;
  /** Override tier */
  tier?: IdentityTier;
  /** Show copy button */
  showCopy?: boolean;
  /** Show hover tooltip with full address */
  showTooltip?: boolean;
  /** Custom CSS classes */
  className?: string;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Link to address page (uses name slug when available) */
  linkable?: boolean;
  /** Custom link destination */
  href?: string;
}

export default function AddressDisplay({
  address,
  displayName: overrideName,
  tier: overrideTier,
  showCopy = true,
  showTooltip = true,
  className = '',
  size = 'md',
  linkable = true,
  href
}: AddressDisplayProps) {
  const [identity, setIdentity] = useState<ResolvedIdentity | null>(
    overrideName ? {
      address,
      displayName: overrideName,
      tier: overrideTier || 'auto-alias',
      slug: overrideName
    } : null
  );
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(!overrideName);

  useEffect(() => {
    if (overrideName) return;
    
    let cancelled = false;
    setLoading(true);
    
    resolveIdentity(address).then(result => {
      if (!cancelled) {
        setIdentity(result);
        setLoading(false);
      }
    }).catch(() => {
      if (!cancelled) setLoading(false);
    });

    return () => { cancelled = true; };
  }, [address, overrideName]);

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

  const displayText = identity?.displayName || formatAddress(address);
  const tier = identity?.tier || 'address';
  
  // Link uses name slug when available, otherwise address
  const slug = identity?.slug || address;
  const linkHref = href || (linkable ? `/${encodeURIComponent(slug)}` : undefined);

  // Tier-specific CSS class
  const tierClass = `address-display--tier-${tier}`;

  const addressElement = (
    <span
      className={`address-display ${className} address-display--${size} ${tierClass} ${
        loading ? 'address-display--loading' : ''
      }`}
      title={showTooltip ? address : undefined}
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
