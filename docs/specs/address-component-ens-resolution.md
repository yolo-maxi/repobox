# Address Component with ENS/Subdomain Resolution + Human-Readable URLs

**Technical Specification**  
**Status:** Implementation Ready  
**Author:** PM Agent  
**Created:** 2026-03-21  
**Priority:** P1  

## Overview

This specification defines the implementation of a reusable `<AddressDisplay>` component with ENS/subdomain resolution capabilities, and human-readable URL routing for the repo.box explorer. The solution provides a consistent address display pattern throughout the application while enabling user-friendly navigation via ENS names and repo.box subdomains.

## Requirements Summary

From KANBAN.md:
- ✅ Reusable `<AddressDisplay>` component
- ✅ Resolution: ENS → repo.box subdomain → truncated hex
- ✅ Hover: full address. Click: copy. All addresses clickable
- ✅ Human-readable URL routing: `/explore/{ens-name}/` and `/explore/{subdomain}/` resolve and show repos
- ✅ Add `/api/explorer/resolve/{name}` endpoint

## Current State Analysis

### Existing Infrastructure
- **ENS Module**: `/web/src/lib/ens.ts` - Basic ENS resolution with 5-minute cache
- **Address Formatting**: `formatAddress()` in `/web/src/lib/utils.ts` - Truncates to `0x1234...5678`
- **Explorer Structure**: Dynamic routes `/explore/[address]/[name]` for repositories
- **Resolve Endpoint**: `/api/explorer/resolve/[name]/route.ts` - Basic ENS resolution API

### Gaps to Address
1. No reusable AddressDisplay component
2. No subdomain resolution system
3. No human-readable routing (`/explore/vitalik.eth/`)
4. Limited ENS resolution implementation
5. No subdomain-to-address mapping

## Architecture Design

### 1. AddressDisplay Component

#### Component Interface
```typescript
interface AddressDisplayProps {
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
```

#### Implementation Plan
```tsx
'use client';

import { useState, useEffect } from 'react';
import { formatAddress, copyToClipboard } from '@/lib/utils';
import { resolveAddressDisplay } from '@/lib/addressResolver';
import Link from 'next/link';

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
```

#### Styling (CSS)
```css
/* /web/src/app/globals.css - Add address display styles */

.address-display {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-family: 'Monaco', 'Menlo', monospace;
  border-radius: 0.375rem;
  background: var(--bg-subtle);
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color);
  transition: all 0.15s ease;
}

.address-display:hover {
  background: var(--bg-hover);
  border-color: var(--border-hover);
}

/* Size variants */
.address-display--sm {
  padding: 0.125rem 0.375rem;
  font-size: 0.75rem;
}

.address-display--md {
  padding: 0.25rem 0.5rem;
  font-size: 0.875rem;
}

.address-display--lg {
  padding: 0.375rem 0.75rem;
  font-size: 1rem;
}

/* State variants */
.address-display--resolved .address-display__text {
  color: var(--text-primary);
  font-weight: 500;
}

.address-display--truncated .address-display__text {
  color: var(--text-secondary);
}

.address-display--clickable {
  cursor: pointer;
}

.address-display--loading {
  opacity: 0.7;
}

.address-display__text {
  display: flex;
  align-items: center;
}

.address-display__loading {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.address-display__spinner {
  width: 0.75rem;
  height: 0.75rem;
  border: 1px solid var(--border-color);
  border-top: 1px solid var(--accent-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.address-display__copy-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0.125rem;
  border-radius: 0.25rem;
  transition: color 0.15s ease;
}

.address-display__copy-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.address-display__icon {
  width: 0.875rem;
  height: 0.875rem;
}

.address-display-link {
  text-decoration: none;
  color: inherit;
}

.address-display-link:hover .address-display {
  background: var(--bg-hover);
  border-color: var(--accent-color);
}
```

### 2. Address Resolution System

#### Enhanced Address Resolver
```typescript
// /web/src/lib/addressResolver.ts
import { resolveENS } from './ens';

interface AddressResolution {
  address: string;
  displayName: string | null;
  type: 'ens' | 'subdomain' | 'address';
  isVerified: boolean;
}

// Cache for resolved addresses (5 minutes TTL)
const resolutionCache = new Map<string, {
  result: AddressResolution;
  timestamp: number;
}>();

const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

export async function resolveAddressDisplay(address: string): Promise<string | null> {
  // Check cache
  const cached = resolutionCache.get(address);
  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return cached.result.displayName;
  }
  
  try {
    // 1. Check for repo.box subdomain
    const subdomainName = await resolveRepoboxSubdomain(address);
    if (subdomainName) {
      const result: AddressResolution = {
        address,
        displayName: subdomainName,
        type: 'subdomain',
        isVerified: true
      };
      resolutionCache.set(address, { result, timestamp: Date.now() });
      return subdomainName;
    }
    
    // 2. Check for ENS name
    const ensName = await reverseResolveENS(address);
    if (ensName) {
      const result: AddressResolution = {
        address,
        displayName: ensName,
        type: 'ens',
        isVerified: true
      };
      resolutionCache.set(address, { result, timestamp: Date.now() });
      return ensName;
    }
    
    // 3. No resolution found
    const result: AddressResolution = {
      address,
      displayName: null,
      type: 'address',
      isVerified: false
    };
    resolutionCache.set(address, { result, timestamp: Date.now() });
    return null;
  } catch (error) {
    console.warn('Address resolution failed:', error);
    return null;
  }
}

export async function resolveNameToAddress(name: string): Promise<string | null> {
  // Check cache by scanning for reverse mapping
  for (const [addr, cached] of resolutionCache.entries()) {
    if (cached.result.displayName === name && Date.now() - cached.timestamp < CACHE_TTL) {
      return addr;
    }
  }
  
  try {
    // 1. Try repo.box subdomain resolution
    const address = await resolveSubdomainToAddress(name);
    if (address) return address;
    
    // 2. Try ENS resolution
    if (name.endsWith('.eth')) {
      const resolved = await resolveENS(name);
      if (resolved) return resolved;
    }
    
    // 3. Check if it's already an address
    if (/^0x[a-fA-F0-9]{40}$/.test(name)) {
      return name;
    }
    
    return null;
  } catch (error) {
    console.warn('Name resolution failed:', error);
    return null;
  }
}

async function resolveRepoboxSubdomain(address: string): Promise<string | null> {
  // Implementation will query subdomain registry
  // For now, return null - will be implemented with subdomain system
  return null;
}

async function reverseResolveENS(address: string): Promise<string | null> {
  try {
    // Use ENS reverse resolution
    const response = await fetch('/api/explorer/reverse-ens', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ address })
    });
    
    if (response.ok) {
      const data = await response.json();
      return data.name || null;
    }
    
    return null;
  } catch (error) {
    console.warn('ENS reverse resolution failed:', error);
    return null;
  }
}

async function resolveSubdomainToAddress(subdomain: string): Promise<string | null> {
  try {
    const response = await fetch(`/api/explorer/subdomains/${subdomain}`);
    if (response.ok) {
      const data = await response.json();
      return data.address || null;
    }
    return null;
  } catch (error) {
    console.warn('Subdomain resolution failed:', error);
    return null;
  }
}
```

#### Enhanced ENS Resolution
```typescript
// /web/src/lib/ens.ts - Enhanced implementation
import { ethers } from 'ethers';

const ENS_REGISTRY = '0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e';
const ENS_REVERSE_REGISTRAR = '0x084b1c3C81545d370f3634392De611CaaBFf8148';

// Simple cache
const ensCache = new Map<string, { 
  address: string | null; 
  timestamp: number; 
  name?: string; 
}>();
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

// Initialize provider
let provider: ethers.JsonRpcProvider | null = null;

function getProvider(): ethers.JsonRpcProvider {
  if (!provider) {
    provider = new ethers.JsonRpcProvider('https://eth.llamarpc.com');
  }
  return provider;
}

export async function resolveENS(name: string): Promise<string | null> {
  if (!name.endsWith('.eth')) return null;
  
  // Check cache
  const cached = ensCache.get(`forward:${name}`);
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.address;
  }
  
  try {
    const prov = getProvider();
    const address = await prov.resolveName(name);
    
    // Cache result
    ensCache.set(`forward:${name}`, {
      address,
      timestamp: Date.now()
    });
    
    return address;
  } catch (error) {
    console.error('ENS resolution error:', error);
    
    // Cache null result to prevent repeated failed lookups
    ensCache.set(`forward:${name}`, {
      address: null,
      timestamp: Date.now()
    });
    
    return null;
  }
}

export async function reverseResolveENS(address: string): Promise<string | null> {
  if (!/^0x[a-fA-F0-9]{40}$/i.test(address)) return null;
  
  // Check cache
  const cached = ensCache.get(`reverse:${address.toLowerCase()}`);
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.name || null;
  }
  
  try {
    const prov = getProvider();
    const name = await prov.lookupAddress(address);
    
    // Verify reverse resolution (prevent spoofing)
    if (name) {
      const verifyAddress = await prov.resolveName(name);
      if (verifyAddress?.toLowerCase() !== address.toLowerCase()) {
        console.warn('ENS reverse resolution verification failed');
        return null;
      }
    }
    
    // Cache result
    ensCache.set(`reverse:${address.toLowerCase()}`, {
      address: address.toLowerCase(),
      name: name || undefined,
      timestamp: Date.now()
    });
    
    return name;
  } catch (error) {
    console.error('ENS reverse resolution error:', error);
    
    // Cache null result
    ensCache.set(`reverse:${address.toLowerCase()}`, {
      address: address.toLowerCase(),
      name: undefined,
      timestamp: Date.now()
    });
    
    return null;
  }
}

export function formatAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
```

### 3. Human-Readable URL Routing

#### Dynamic Route Handler
```typescript
// /web/src/app/explore/[addressOrName]/route.ts - NEW
import { NextRequest, NextResponse } from 'next/server';
import { resolveNameToAddress } from '@/lib/addressResolver';

interface RouteContext {
  params: Promise<{ addressOrName: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  const { addressOrName } = await context.params;
  
  try {
    // If it's already an address, continue normally
    if (/^0x[a-fA-F0-9]{40}$/i.test(addressOrName)) {
      return NextResponse.redirect(new URL(`/explore/${addressOrName}`, request.url));
    }
    
    // Try to resolve name to address
    const resolvedAddress = await resolveNameToAddress(addressOrName);
    if (resolvedAddress) {
      return NextResponse.redirect(new URL(`/explore/${resolvedAddress}`, request.url));
    }
    
    // Name not found
    return NextResponse.json(
      { error: 'Address or name not found' },
      { status: 404 }
    );
  } catch (error) {
    console.error('Route resolution error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
```

#### Enhanced Repository Page
```typescript
// /web/src/app/explore/[addressOrName]/[name]/page.tsx - MODIFIED
'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { resolveNameToAddress } from '@/lib/addressResolver';
// ... other imports

export default function RepoPage() {
  const params = useParams();
  const router = useRouter();
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolving, setResolving] = useState(true);
  
  const addressOrName = Array.isArray(params.addressOrName) 
    ? params.addressOrName[0] 
    : params.addressOrName;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;

  // Resolve name to address if needed
  useEffect(() => {
    const resolveAddress = async () => {
      if (!addressOrName) return;
      
      // If it's already an address, use it directly
      if (/^0x[a-fA-F0-9]{40}$/i.test(addressOrName)) {
        setResolvedAddress(addressOrName);
        setResolving(false);
        return;
      }
      
      // Try to resolve name
      try {
        const resolved = await resolveNameToAddress(addressOrName);
        if (resolved) {
          setResolvedAddress(resolved);
          // Update URL to canonical address form
          router.replace(`/explore/${resolved}/${name}`);
        } else {
          // Name not found
          setResolvedAddress(null);
        }
      } catch (error) {
        console.error('Resolution failed:', error);
        setResolvedAddress(null);
      } finally {
        setResolving(false);
      }
    };
    
    resolveAddress();
  }, [addressOrName, name, router]);

  if (resolving) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-loading">
              <div className="explore-loading-spinner"></div>
              <p>Resolving address...</p>
            </div>
          </main>
        </div>
      </div>
    );
  }

  if (!resolvedAddress) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-empty">
              <h3>Address not found</h3>
              <p>Could not resolve "{addressOrName}" to an address.</p>
              <Link href="/explore" className="explore-back-link">← Back to Explorer</Link>
            </div>
          </main>
        </div>
      </div>
    );
  }

  // Continue with existing repo page logic using resolvedAddress
  // Replace all instances of `address` with `resolvedAddress`
  // ... rest of component implementation
}
```

### 4. API Endpoints

#### Enhanced Resolve Endpoint
```typescript
// /web/src/app/api/explorer/resolve/[name]/route.ts - ENHANCED
import { NextRequest, NextResponse } from 'next/server';
import { resolveNameToAddress, resolveAddressDisplay } from '@/lib/addressResolver';

interface RouteContext {
  params: Promise<{ name: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { name } = await context.params;
    
    if (!name) {
      return NextResponse.json(
        { error: 'Name is required' },
        { status: 400 }
      );
    }
    
    // Resolve name to address
    const address = await resolveNameToAddress(name);
    if (!address) {
      return NextResponse.json(
        { error: 'Name not found or not resolved' },
        { status: 404 }
      );
    }
    
    // Get display name for the resolved address
    const displayName = await resolveAddressDisplay(address);
    
    return NextResponse.json({
      name,
      address,
      displayName,
      type: name.endsWith('.eth') ? 'ens' : 
            /^0x[a-fA-F0-9]{40}$/i.test(name) ? 'address' : 'subdomain'
    });
  } catch (error) {
    console.error('Resolve endpoint error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
```

#### ENS Reverse Resolution API
```typescript
// /web/src/app/api/explorer/reverse-ens/route.ts - NEW
import { NextRequest, NextResponse } from 'next/server';
import { reverseResolveENS } from '@/lib/ens';

export async function POST(request: NextRequest) {
  try {
    const { address } = await request.json();
    
    if (!address || !/^0x[a-fA-F0-9]{40}$/i.test(address)) {
      return NextResponse.json(
        { error: 'Valid address is required' },
        { status: 400 }
      );
    }
    
    const name = await reverseResolveENS(address);
    
    return NextResponse.json({
      address,
      name,
      found: !!name
    });
  } catch (error) {
    console.error('Reverse ENS resolution error:', error);
    return NextResponse.json(
      { error: 'Resolution failed' },
      { status: 500 }
    );
  }
}
```

#### Subdomain Registry API
```typescript
// /web/src/app/api/explorer/subdomains/[name]/route.ts - NEW
import { NextRequest, NextResponse } from 'next/server';
// Note: Implementation will depend on subdomain registry system

interface RouteContext {
  params: Promise<{ name: string }>;
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { name } = await context.params;
    
    // For now, return 404 - will be implemented with subdomain system
    return NextResponse.json(
      { error: 'Subdomain registry not implemented' },
      { status: 404 }
    );
  } catch (error) {
    console.error('Subdomain resolution error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
```

### 5. Caching Strategy

#### Multi-Layer Cache Architecture
```typescript
// /web/src/lib/cache.ts - NEW
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

class Cache<T> {
  private store = new Map<string, CacheEntry<T>>();
  private maxSize: number;

  constructor(maxSize = 1000) {
    this.maxSize = maxSize;
  }

  set(key: string, data: T, ttlMs = 5 * 60 * 1000): void {
    // Evict oldest entries if at capacity
    if (this.store.size >= this.maxSize) {
      const oldestKey = this.store.keys().next().value;
      if (oldestKey) this.store.delete(oldestKey);
    }

    this.store.set(key, {
      data,
      timestamp: Date.now(),
      ttl: ttlMs
    });
  }

  get(key: string): T | null {
    const entry = this.store.get(key);
    if (!entry) return null;

    // Check if expired
    if (Date.now() - entry.timestamp > entry.ttl) {
      this.store.delete(key);
      return null;
    }

    return entry.data;
  }

  clear(): void {
    this.store.clear();
  }
}

// Export specialized caches
export const ensCache = new Cache<{ address?: string; name?: string }>(500);
export const addressDisplayCache = new Cache<string | null>(1000);
export const subdomainCache = new Cache<string>(200);
```

#### Database Caching (Future)
```sql
-- For future implementation with database backend
CREATE TABLE IF NOT EXISTS address_resolution_cache (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    display_name TEXT,
    resolution_type TEXT CHECK(resolution_type IN ('ens', 'subdomain', 'address')),
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT NOT NULL
);

CREATE INDEX idx_address_cache_addr ON address_resolution_cache(address);
CREATE INDEX idx_address_cache_name ON address_resolution_cache(display_name);
CREATE INDEX idx_address_cache_expires ON address_resolution_cache(expires_at);
```

### 6. URL Schema & Routing

#### Supported URL Patterns
```
# Current (Hex Addresses)
/explore/0x742d35Cc6635C0532925a3b8D093C7C85EC54C6e                    → User profile
/explore/0x742d35Cc6635C0532925a3b8D093C7C85EC54C6e/my-repo            → Repository

# NEW: ENS Names
/explore/vitalik.eth                                                    → Resolves to address profile  
/explore/vitalik.eth/ens-contracts                                      → Repository

# NEW: repo.box Subdomains (Future)
/explore/alice                                                          → Resolves to alice.repo.box → address
/explore/alice/dapp                                                     → alice.repo.box/dapp repository

# API Endpoints
/api/explorer/resolve/vitalik.eth                                       → { address: "0x...", displayName: "vitalik.eth" }
/api/explorer/resolve/alice                                             → { address: "0x...", displayName: "alice" }
/api/explorer/reverse-ens                                               → POST { address } → { name }
/api/explorer/subdomains/alice                                          → { address: "0x..." }
```

#### Next.js Route Structure
```
web/src/app/explore/
├── page.tsx                               # Explorer home
├── layout.tsx                             # Shared layout
├── [addressOrName]/                       # Dynamic address or name resolution
│   ├── page.tsx                           # User profile (redirect handler)
│   └── [name]/                           
│       ├── page.tsx                       # Repository page
│       ├── commits/[branch]/page.tsx      # Commit history
│       ├── blob/[branch]/[...path]/page.tsx  # File viewer
│       └── commit/[hash]/page.tsx         # Commit detail
└── api/explorer/
    ├── resolve/[name]/route.ts            # Name → Address resolution
    ├── reverse-ens/route.ts               # Address → ENS reverse lookup
    └── subdomains/[name]/route.ts         # Subdomain → Address resolution
```

## Implementation Plan

### Phase 1: Core Component (Priority: P0)
**Estimate: 1-2 days**

- [ ] Create `AddressDisplay` component with basic functionality
- [ ] Implement component styling and variants  
- [ ] Add to existing explorer pages (replace inline address formatting)
- [ ] Basic hover tooltips and click-to-copy

**Files to Create/Modify:**
- `web/src/components/AddressDisplay.tsx` (NEW)
- `web/src/lib/addressResolver.ts` (NEW)  
- `web/src/app/explore/[address]/[name]/page.tsx` (MODIFY)
- `web/src/app/globals.css` (MODIFY - add styles)

### Phase 2: ENS Resolution (Priority: P0)
**Estimate: 1 day**

- [ ] Enhance ENS resolution with reverse lookup
- [ ] Add ethers.js provider integration
- [ ] Implement caching layer
- [ ] Add ENS reverse resolution API endpoint

**Files to Create/Modify:**
- `web/src/lib/ens.ts` (ENHANCE)
- `web/src/app/api/explorer/reverse-ens/route.ts` (NEW)
- Package.json (ADD ethers dependency)

### Phase 3: Human-Readable Routing (Priority: P1)  
**Estimate: 2 days**

- [ ] Implement dynamic name resolution routing
- [ ] Add route handlers for `/explore/[addressOrName]/`
- [ ] Update repository page for name resolution
- [ ] Enhance resolve API endpoint

**Files to Create/Modify:**
- `web/src/app/explore/[addressOrName]/page.tsx` (NEW)
- `web/src/app/explore/[addressOrName]/[name]/page.tsx` (NEW)  
- `web/src/app/api/explorer/resolve/[name]/route.ts` (ENHANCE)

### Phase 4: Subdomain System (Priority: P2)
**Estimate: 3-4 days**

- [ ] Design subdomain registry system  
- [ ] Implement subdomain resolution APIs
- [ ] Add subdomain registration flow
- [ ] Update AddressDisplay for subdomain support

**Files to Create/Modify:**
- `web/src/app/api/explorer/subdomains/[name]/route.ts` (NEW)
- Subdomain registry implementation (TBD)
- Database schema updates (TBD)

### Phase 5: Testing & Polish (Priority: P1)
**Estimate: 1 day**

- [ ] Unit tests for components
- [ ] API endpoint testing  
- [ ] Error handling improvements
- [ ] Performance optimization

## Testing Strategy

### Unit Tests
```typescript
// tests/components/AddressDisplay.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import AddressDisplay from '@/components/AddressDisplay';

describe('AddressDisplay', () => {
  const mockAddress = '0x742d35Cc6635C0532925a3b8D093C7C85EC54C6e';
  
  test('renders formatted address by default', () => {
    render(<AddressDisplay address={mockAddress} />);
    expect(screen.getByText('0x742d...4C6e')).toBeInTheDocument();
  });
  
  test('displays custom display name when provided', () => {
    render(<AddressDisplay address={mockAddress} displayName="vitalik.eth" />);
    expect(screen.getByText('vitalik.eth')).toBeInTheDocument();
  });
  
  test('copies address to clipboard on click', async () => {
    Object.assign(navigator, {
      clipboard: { writeText: jest.fn() }
    });
    
    render(<AddressDisplay address={mockAddress} />);
    fireEvent.click(screen.getByRole('button'));
    
    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(mockAddress);
    });
  });
});
```

### Integration Tests
```typescript
// tests/api/resolve.test.ts
describe('/api/explorer/resolve/[name]', () => {
  test('resolves ENS name to address', async () => {
    const response = await fetch('/api/explorer/resolve/vitalik.eth');
    const data = await response.json();
    
    expect(response.ok).toBe(true);
    expect(data.address).toMatch(/^0x[a-fA-F0-9]{40}$/);
    expect(data.name).toBe('vitalik.eth');
    expect(data.type).toBe('ens');
  });
  
  test('returns 404 for non-existent names', async () => {
    const response = await fetch('/api/explorer/resolve/nonexistent.eth');
    expect(response.status).toBe(404);
  });
});
```

### Manual Testing Scenarios
1. **ENS Resolution**: Test with known ENS names (vitalik.eth, etc.)
2. **Invalid Names**: Test error handling for malformed/non-existent names
3. **Caching**: Verify cache behavior with repeated requests  
4. **Address Display**: Test all component variants and states
5. **Routing**: Navigate via human-readable URLs

## Security Considerations

### ENS Security
- **Reverse Resolution Verification**: Always verify reverse resolution by checking forward resolution matches
- **Cache Poisoning**: Use short TTLs and validate all cached data  
- **Input Sanitization**: Validate all ENS names and addresses before processing

### API Security  
- **Rate Limiting**: Implement rate limits on resolution endpoints
- **Input Validation**: Strict validation of address/name formats
- **Error Information**: Avoid leaking internal system details in errors

### Caching Security
- **TTL Enforcement**: Prevent stale data from being served
- **Memory Limits**: Prevent cache size from growing unbounded
- **Data Validation**: Validate cached data before serving

## Performance Considerations

### Resolution Performance
- **Parallel Resolution**: Resolve ENS and subdomain lookups in parallel where possible
- **Batch Processing**: Group multiple resolutions when practical
- **Background Updates**: Update cache entries in background for popular addresses

### Caching Strategy
- **Multi-Level Cache**: Memory cache → API cache → Database cache
- **Cache Warming**: Pre-populate cache for frequently accessed addresses  
- **Intelligent Eviction**: Evict based on access patterns, not just timestamp

### Bundle Size
- **Tree Shaking**: Ensure ethers.js is tree-shaken properly
- **Code Splitting**: Load resolution logic only when needed
- **Lazy Loading**: Defer ENS resolution until component is in viewport

## Error Handling & Fallbacks

### Resolution Failures
```typescript
// Graceful degradation pattern
export function AddressDisplayWithFallback({ address, ...props }: AddressDisplayProps) {
  return (
    <ErrorBoundary
      fallback={<code>{formatAddress(address)}</code>}
      onError={(error) => console.warn('AddressDisplay error:', error)}
    >
      <AddressDisplay address={address} {...props} />
    </ErrorBoundary>
  );
}
```

### Network Issues  
- **Timeout Handling**: 5-second timeout for ENS resolution
- **Retry Logic**: Exponential backoff for transient failures
- **Offline Graceful**: Fall back to cached data or formatted address

### Invalid Data
- **Address Validation**: Check address format before processing
- **ENS Format Check**: Validate ENS name format (.eth suffix, valid characters)  
- **Sanitization**: Clean user input to prevent injection attacks

## Success Metrics

### Functionality Metrics
- [ ] **Resolution Accuracy**: >99% correct ENS → Address resolution
- [ ] **Cache Hit Rate**: >80% cache hits for repeated lookups  
- [ ] **Component Coverage**: AddressDisplay used in 100% of address displays

### Performance Metrics  
- [ ] **Resolution Speed**: <2s average ENS resolution time
- [ ] **Page Load Impact**: <100ms additional load time for address resolution
- [ ] **Bundle Size**: <50KB increase from ENS dependencies

### User Experience Metrics
- [ ] **Error Rate**: <1% resolution failures for valid names
- [ ] **Copy Success**: >95% clipboard copy success rate
- [ ] **Tooltip Accuracy**: 100% accurate full address display on hover

## Future Enhancements

### Subdomain Registry Integration
- Custom repo.box subdomains (alice.repo.box → 0x...)
- Subdomain marketplace/auction system  
- Bulk subdomain registration for organizations

### Enhanced ENS Features
- Support for other ENS TLDs (.crypto, .nft, etc.)
- ENS avatar display integration
- ENS content hash resolution for IPFS repos

### Address Book & Contacts
- User-defined address labels/nicknames
- Contact sharing between users
- Organization/team address management

### Advanced Caching
- Redis/database-backed cache for production
- Cache analytics and optimization  
- Proactive cache warming for popular addresses

## Dependencies

### New Dependencies
```json
{
  "devDependencies": {
    "@types/react": "^18.0.0"
  },
  "dependencies": {
    "ethers": "^6.0.0"
  }
}
```

### Existing Dependencies (No Change)
- React 19+
- Next.js 14+  
- TypeScript
- Tailwind CSS

## Deployment Notes

### Environment Variables
```bash
# Optional: Custom ENS RPC endpoint
ENS_RPC_URL=https://eth.llamarpc.com

# Optional: ENS resolver cache TTL (milliseconds)  
ENS_CACHE_TTL=300000
```

### Build Considerations
- Ensure ethers.js is properly tree-shaken
- Test bundle size impact before deployment
- Verify ENS resolution works in production environment

---

## Summary

This specification provides a comprehensive implementation plan for adding ENS/subdomain resolution and human-readable URLs to repo.box. The solution is structured in phases for incremental delivery while maintaining backwards compatibility.

**Key Deliverables:**
1. Reusable `<AddressDisplay>` component with resolution
2. Human-readable URL routing (`/explore/vitalik.eth/repo`)  
3. Enhanced ENS resolution with reverse lookup
4. Robust caching and error handling
5. Clean API endpoints for name resolution

The implementation prioritizes user experience while maintaining performance and security standards.