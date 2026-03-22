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

  // Try reverse resolution via ethers with fallback RPCs
  const ETH_RPCS = [
    'https://eth.drpc.org',
    'https://ethereum-rpc.publicnode.com',
    'https://rpc.mevblocker.io',
  ];

  try {
    const { ethers } = await import('ethers');
    for (const rpc of ETH_RPCS) {
      try {
        const provider = new ethers.JsonRpcProvider(rpc);
        const name = await Promise.race([
          provider.lookupAddress(address),
          new Promise<null>((_, reject) => setTimeout(() => reject(new Error('timeout')), 8000))
        ]);
        if (name) {
          ensCache.set(`reverse:${address.toLowerCase()}`, {
            address: address.toLowerCase(),
            name,
            timestamp: Date.now()
          });
          return name;
        }
      } catch {
        continue;
      }
    }
  } catch (e) {
    console.error('Reverse ENS error:', e);
  }

  ensCache.set(`reverse:${address.toLowerCase()}`, {
    address: address.toLowerCase(),
    name: undefined,
    timestamp: Date.now()
  });
  return null;
}

const ALIAS_ADJ_1 = ['deep', 'wild', 'bright', 'silent', 'swift', 'lunar', 'solar', 'frost', 'ember', 'neon', 'misty', 'stone', 'velvet', 'cosmic', 'golden', 'azure'];
const ALIAS_ADJ_2 = ['blue', 'green', 'coral', 'silver', 'crimson', 'violet', 'amber', 'teal', 'indigo', 'scarlet', 'cobalt', 'pearl', 'obsidian', 'jade', 'sunset', 'aqua'];
const ALIAS_ANIMAL = ['kraken', 'otter', 'falcon', 'fox', 'wolf', 'lynx', 'orca', 'raven', 'viper', 'tiger', 'panda', 'eagle', 'whale', 'manta', 'gecko', 'badger'];

export function formatAddress(address: string): string {
  if (!address) return '';
  if (!/^0x[a-fA-F0-9]{40}$/.test(address)) return address;

  const hex = address.slice(2).toLowerCase();
  const a = parseInt(hex.slice(0, 2), 16) % ALIAS_ADJ_1.length;
  const b = parseInt(hex.slice(2, 4), 16) % ALIAS_ADJ_2.length;
  const c = parseInt(hex.slice(4, 6), 16) % ALIAS_ANIMAL.length;

  return `${ALIAS_ADJ_1[a]}-${ALIAS_ADJ_2[b]}-${ALIAS_ANIMAL[c]}`;
}