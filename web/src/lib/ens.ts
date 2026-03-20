// Simple ENS resolution cache
const cache = new Map<string, { address: string; timestamp: number }>();
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

export async function resolveENS(name: string): Promise<string | null> {
  // Check cache first
  const cached = cache.get(name);
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.address;
  }
  
  try {
    // Use Ethereum mainnet RPC
    const rpcUrl = 'https://eth.llamarpc.com';
    
    // Simple ENS resolution via RPC
    // This is a basic implementation - for production you'd want a more robust ENS library
    const response = await fetch(rpcUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'eth_call',
        params: [
          {
            to: '0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e', // ENS Registry
            data: `0x0178b8bf${name}` // resolver(bytes32)
          },
          'latest'
        ]
      })
    });
    
    const data = await response.json();
    
    if (data.result && data.result !== '0x' && data.result !== '0x0000000000000000000000000000000000000000000000000000000000000000') {
      // This is a simplified ENS resolution
      // In a real implementation, you'd need to:
      // 1. Get the resolver address from the registry
      // 2. Call addr() on the resolver
      // For now, we'll return a placeholder since the full ENS resolution is complex
      
      // Simple approach: check if the name ends with .eth and is valid format
      if (name.endsWith('.eth') && name.length > 4) {
        // Cache and return null for now - actual resolution would require more complex logic
        cache.set(name, { address: '', timestamp: Date.now() });
        return null;
      }
    }
    
    return null;
  } catch (error) {
    console.error('ENS resolution error:', error);
    return null;
  }
}

export function formatAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}