'use client';

import AddressDisplay from '@/components/AddressDisplay';

export default function TestAddressPage() {
  const testAddress = '0x742d35Cc6635C0532925a3b8D093C7C85EC54C6e';
  
  return (
    <div style={{ padding: '2rem', background: 'var(--bp-bg)', minHeight: '100vh' }}>
      <h1 style={{ color: 'var(--bp-heading)', marginBottom: '2rem' }}>AddressDisplay Component Test</h1>
      
      <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', maxWidth: '600px' }}>
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>Small variant</h3>
          <AddressDisplay address={testAddress} size="sm" />
        </div>
        
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>Medium variant (default)</h3>
          <AddressDisplay address={testAddress} size="md" />
        </div>
        
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>Large variant</h3>
          <AddressDisplay address={testAddress} size="lg" />
        </div>
        
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>With custom display name</h3>
          <AddressDisplay address={testAddress} displayName="vitalik.eth" size="md" />
        </div>
        
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>Non-linkable (click to copy)</h3>
          <AddressDisplay address={testAddress} size="md" linkable={false} />
        </div>
        
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>Without copy button</h3>
          <AddressDisplay address={testAddress} size="md" showCopy={false} />
        </div>
        
        <div>
          <h3 style={{ color: 'var(--bp-text)', marginBottom: '0.5rem' }}>ENS resolution test (if ENS works)</h3>
          <AddressDisplay address="0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045" size="md" />
          <p style={{ color: 'var(--bp-dim)', fontSize: '0.8rem', marginTop: '0.5rem' }}>
            This should resolve to vitalik.eth if ENS resolution is working
          </p>
        </div>
      </div>
    </div>
  );
}