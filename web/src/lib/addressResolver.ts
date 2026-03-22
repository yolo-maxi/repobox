import { reverseResolveENS } from './ens';

// Known identities from .boxconfig — local fallback for display names
const KNOWN_IDENTITIES: Record<string, string> = {
  '0xdbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048': 'Ocean',
  '0x69C2920CA309577bcd79e4e6e3afdda93287Cc8b': '0xfran.eth',
  '0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b': 'PM Agent',
  '0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00': 'Claude',
  '0x82240a161Fea724D059a74F948C8E18674c0fA09': 'Codex',
  '0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307': 'Reviewer',
};

function lookupKnownIdentity(address: string): string | null {
  const lower = address.toLowerCase();
  for (const [addr, name] of Object.entries(KNOWN_IDENTITIES)) {
    if (addr.toLowerCase() === lower) return name;
  }
  return null;
}

export interface AddressResolution {
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
    // 0. Check known identities (instant, local)
    const known = lookupKnownIdentity(address);
    if (known) {
      const result: AddressResolution = {
        address,
        displayName: known,
        type: 'subdomain',
        isVerified: true
      };
      resolutionCache.set(address, { result, timestamp: Date.now() });
      return known;
    }

    // 1. Check for repo.box subdomain (future implementation)
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
    // 1. Try repo.box subdomain resolution (future implementation)
    const address = await resolveSubdomainToAddress(name);
    if (address) return address;
    
    // 2. Try ENS resolution
    if (name.endsWith('.eth')) {
      const response = await fetch(`/api/explorer/resolve/${encodeURIComponent(name)}`);
      if (response.ok) {
        const data = await response.json();
        return data.address || null;
      }
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

async function resolveSubdomainToAddress(subdomain: string): Promise<string | null> {
  try {
    const response = await fetch(`/api/explorer/subdomains/${encodeURIComponent(subdomain)}`);
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