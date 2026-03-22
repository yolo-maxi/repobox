import { reverseResolveENS } from './ens';

/**
 * Identity resolution hierarchy:
 * 
 * Tier 1: ENS name with reverse resolution (e.g. "0xfran.eth")
 *   → Display: full name, standard weight
 * 
 * Tier 2: *.repobox.eth on-chain subdomain (purchased NFT name)
 *   → Display: just the name, accent color
 *   → (Not yet implemented — placeholder for post-hackathon)
 * 
 * Tier 3: *.repobox.eth auto-assigned alias (free, from gateway)
 *   → Display: name in muted/italic style
 * 
 * Fallback: truncated 0x address
 */

export type IdentityTier = 'ens' | 'purchased' | 'auto-alias' | 'address';

export interface ResolvedIdentity {
  address: string;
  displayName: string | null;
  tier: IdentityTier;
  /** The URL-safe slug to use in links (name or address) */
  slug: string;
}

// Cache: address (lowercase) -> resolution
const cache = new Map<string, { result: ResolvedIdentity; ts: number }>();
const CACHE_TTL = 5 * 60 * 1000;

// Batch queue for bulk resolution
let batchQueue: { address: string; resolve: (r: ResolvedIdentity) => void }[] = [];
let batchTimer: ReturnType<typeof setTimeout> | null = null;

function getCached(address: string): ResolvedIdentity | null {
  const entry = cache.get(address.toLowerCase());
  if (entry && Date.now() - entry.ts < CACHE_TTL) return entry.result;
  return null;
}

function setCache(result: ResolvedIdentity): void {
  cache.set(result.address.toLowerCase(), { result, ts: Date.now() });
}

function makeAddressFallback(address: string): ResolvedIdentity {
  return { address, displayName: null, tier: 'address', slug: address };
}

/**
 * Resolve a single address to its best identity.
 * Resolution order: ENS reverse → gateway reverse (auto-alias) → fallback
 */
export async function resolveIdentity(address: string): Promise<ResolvedIdentity> {
  const cached = getCached(address);
  if (cached) return cached;

  // 1. Check ENS reverse resolution
  try {
    const ensName = await reverseResolveENS(address);
    if (ensName) {
      const result: ResolvedIdentity = {
        address,
        displayName: ensName,
        tier: 'ens',
        slug: ensName
      };
      setCache(result);
      return result;
    }
  } catch (e) {
    // ENS failed, continue
  }

  // 2. Check gateway reverse (auto-alias)
  try {
    const res = await fetch(`/api/explorer/identity/reverse/${encodeURIComponent(address)}`);
    if (res.ok) {
      const data = await res.json();
      if (data.alias) {
        const result: ResolvedIdentity = {
          address,
          displayName: data.alias,
          tier: data.tier === 'purchased' ? 'purchased' : 'auto-alias',
          slug: data.alias
        };
        setCache(result);
        return result;
      }
    }
  } catch (e) {
    // Gateway failed, continue
  }

  const fallback = makeAddressFallback(address);
  setCache(fallback);
  return fallback;
}

/**
 * Bulk resolve multiple addresses in one call.
 */
export async function resolveIdentities(addresses: string[]): Promise<Map<string, ResolvedIdentity>> {
  const results = new Map<string, ResolvedIdentity>();
  const uncached: string[] = [];

  for (const addr of addresses) {
    const cached = getCached(addr);
    if (cached) {
      results.set(addr.toLowerCase(), cached);
    } else {
      uncached.push(addr);
    }
  }

  if (uncached.length === 0) return results;

  // Bulk call to gateway reverse
  try {
    const res = await fetch('/api/explorer/identity/reverse-bulk', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ addresses: uncached })
    });
    if (res.ok) {
      const data: Record<string, { alias: string; tier: string } | null> = await res.json();
      for (const addr of uncached) {
        const entry = data[addr.toLowerCase()];
        if (entry) {
          const result: ResolvedIdentity = {
            address: addr,
            displayName: entry.alias,
            tier: entry.tier === 'purchased' ? 'purchased' : 'auto-alias',
            slug: entry.alias
          };
          setCache(result);
          results.set(addr.toLowerCase(), result);
        } else {
          const fallback = makeAddressFallback(addr);
          setCache(fallback);
          results.set(addr.toLowerCase(), fallback);
        }
      }
    }
  } catch (e) {
    // Fallback: set address-only for uncached
    for (const addr of uncached) {
      if (!results.has(addr.toLowerCase())) {
        const fallback = makeAddressFallback(addr);
        results.set(addr.toLowerCase(), fallback);
      }
    }
  }

  return results;
}

/**
 * Forward resolve: name → address
 */
export async function resolveNameToAddress(name: string): Promise<string | null> {
  // Check cache reverse
  for (const [, entry] of cache) {
    if (entry.result.displayName?.toLowerCase() === name.toLowerCase() && Date.now() - entry.ts < CACHE_TTL) {
      return entry.result.address;
    }
  }

  try {
    // Try subdomain resolution first (handles both purchased and auto-alias)
    const response = await fetch(`/api/explorer/subdomains/${encodeURIComponent(name)}`);
    if (response.ok) {
      const data = await response.json();
      if (data.address) return data.address;
    }

    // Try ENS (.eth suffix)
    if (name.endsWith('.eth')) {
      const response = await fetch(`/api/explorer/resolve/${encodeURIComponent(name)}`);
      if (response.ok) {
        const data = await response.json();
        return data.address || null;
      }
    }

    // Already an address?
    if (/^0x[a-fA-F0-9]{40}$/i.test(name)) {
      return name;
    }

    return null;
  } catch (e) {
    return null;
  }
}

// Legacy compat
export async function resolveAddressDisplay(address: string): Promise<string | null> {
  const identity = await resolveIdentity(address);
  return identity.displayName;
}

// Legacy compat
export type AddressResolution = ResolvedIdentity;
