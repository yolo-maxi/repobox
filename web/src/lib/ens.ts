import { ethers } from 'ethers';

// Enhanced ENS cache
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