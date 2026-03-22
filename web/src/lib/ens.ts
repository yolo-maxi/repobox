// ENS resolution via the repo.box server's Alchemy-backed resolver.
// Public RPCs (llamarpc, cloudflare-eth, 1rpc) all block ENS calls,
// so we proxy through the server which has an Alchemy key.

const GIT_SERVER = process.env.NEXT_PUBLIC_GIT_SERVER || 'http://127.0.0.1:3490';

const ensCache = new Map<string, {
  address: string | null;
  timestamp: number;
  name?: string;
}>();
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

export async function resolveENS(name: string): Promise<string | null> {
  if (!name.endsWith('.eth')) return null;

  const cached = ensCache.get(`forward:${name}`);
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.address;
  }

  try {
    const res = await fetch(`${GIT_SERVER}/api/resolve?name=${encodeURIComponent(name)}`, {
      signal: AbortSignal.timeout(10000),
    });
    if (!res.ok) {
      ensCache.set(`forward:${name}`, { address: null, timestamp: Date.now() });
      return null;
    }
    const data = await res.json();
    const address = data.address || null;
    ensCache.set(`forward:${name}`, { address, timestamp: Date.now() });
    return address;
  } catch (error) {
    console.error('ENS resolution error:', error);
    ensCache.set(`forward:${name}`, { address: null, timestamp: Date.now() });
    return null;
  }
}

export async function reverseResolveENS(address: string): Promise<string | null> {
  if (!/^0x[a-fA-F0-9]{40}$/i.test(address)) return null;

  const cached = ensCache.get(`reverse:${address.toLowerCase()}`);
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.name || null;
  }

  // Reverse resolution requires an API call we don't have server-side yet.
  // Cache null for now — forward resolution is the priority.
  ensCache.set(`reverse:${address.toLowerCase()}`, {
    address: address.toLowerCase(),
    name: undefined,
    timestamp: Date.now()
  });
  return null;
}

export function formatAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}