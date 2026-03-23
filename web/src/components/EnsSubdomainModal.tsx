'use client';

import { useEffect, useMemo, useState } from 'react';
import { ethers } from 'ethers';

const MINT_ADDRESS = '0x09c4D67e3491EeFBe2a51eaBF1E473e3Ee0B8518';
const MAINNET_CHAIN_ID_HEX = '0x1';

const MINT_ABI = [
  'function currentPrice() view returns (uint256)',
  'function nameExists(string) view returns (bool)',
  'function mint(string name, address resolvedAddress) payable returns (uint256)'
];

type Availability = 'unknown' | 'checking' | 'available' | 'taken';

interface EnsSubdomainModalProps {
  triggerLabel?: string;
  className?: string;
  initialOpen?: boolean;
  prefillName?: string;
}

function normalizeName(input: string): string {
  return input.trim().toLowerCase();
}

function isValidName(name: string): boolean {
  return /^[a-z0-9-]{3,32}$/.test(name);
}

export default function EnsSubdomainModal({
  triggerLabel = 'Get ENS Subdomain',
  className = '',
  initialOpen = false,
  prefillName = ''
}: EnsSubdomainModalProps) {
  const [open, setOpen] = useState(initialOpen);
  const [wallet, setWallet] = useState<string | null>(null);
  const [priceEth, setPriceEth] = useState<string | null>(null);
  const [name, setName] = useState(normalizeName(prefillName));
  const [availability, setAvailability] = useState<Availability>('unknown');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const normalizedName = useMemo(() => normalizeName(name), [name]);
  const fullName = normalizedName ? `${normalizedName}.repobox.eth` : '';

  useEffect(() => {
    if (initialOpen) setOpen(true);
  }, [initialOpen]);

  useEffect(() => {
    if (!prefillName) return;
    setName(normalizeName(prefillName));
  }, [prefillName]);

  async function getProvider(): Promise<ethers.BrowserProvider> {
    const eth = (window as any).ethereum;
    if (!eth) throw new Error('No wallet found. Please install MetaMask.');
    return new ethers.BrowserProvider(eth);
  }

  async function ensureMainnet(provider: ethers.BrowserProvider) {
    const network = await provider.getNetwork();
    if (Number(network.chainId) === 1) return;

    try {
      await (window as any).ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: MAINNET_CHAIN_ID_HEX }]
      });
    } catch {
      throw new Error('Please switch your wallet to Ethereum Mainnet.');
    }
  }

  async function connectWallet() {
    setError(null);
    setBusy(true);
    try {
      const provider = await getProvider();
      await provider.send('eth_requestAccounts', []);
      await ensureMainnet(provider);

      const signer = await provider.getSigner();
      const addr = await signer.getAddress();
      setWallet(addr);

      const contract = new ethers.Contract(MINT_ADDRESS, MINT_ABI, provider);
      const price = await contract.currentPrice();
      setPriceEth(ethers.formatEther(price));
    } catch (e: any) {
      setError(e?.message || 'Failed to connect wallet.');
    } finally {
      setBusy(false);
    }
  }

  async function checkAvailability() {
    setError(null);
    setTxHash(null);

    if (!isValidName(normalizedName)) {
      setAvailability('unknown');
      setError('Name must be 3-32 chars: lowercase letters, numbers, hyphens.');
      return;
    }

    setAvailability('checking');
    try {
      const provider = await getProvider();
      const contract = new ethers.Contract(MINT_ADDRESS, MINT_ABI, provider);
      const exists = await contract.nameExists(normalizedName);
      setAvailability(exists ? 'taken' : 'available');
    } catch {
      setAvailability('unknown');
      setError('Could not check availability.');
    }
  }

  async function mintName() {
    setError(null);
    setTxHash(null);

    if (!wallet) {
      setError('Connect your wallet first.');
      return;
    }
    if (!isValidName(normalizedName)) {
      setError('Enter a valid name first.');
      return;
    }

    setBusy(true);
    try {
      const provider = await getProvider();
      await ensureMainnet(provider);
      const signer = await provider.getSigner();
      const contract = new ethers.Contract(MINT_ADDRESS, MINT_ABI, signer);

      const exists = await contract.nameExists(normalizedName);
      if (exists) {
        setAvailability('taken');
        setError('That name is already taken.');
        return;
      }

      setAvailability('available');
      const price = await contract.currentPrice();
      const tx = await contract.mint(normalizedName, wallet, { value: price });
      setTxHash(tx.hash);
      await tx.wait();
    } catch (e: any) {
      setError(e?.shortMessage || e?.message || 'Mint transaction failed.');
    } finally {
      setBusy(false);
    }
  }

  function resetAndClose() {
    setOpen(false);
    setError(null);
    setTxHash(null);
  }

  return (
    <>
      <button className={`ens-cta-btn ${className}`} onClick={() => setOpen(true)}>
        {triggerLabel}
      </button>

      {open && (
        <div className="ens-modal-overlay" onClick={resetAndClose}>
          <div className="ens-modal" onClick={(e) => e.stopPropagation()}>
            <div className="ens-modal-header">
              <h3>Buy ENS Subdomain</h3>
              <button className="ens-close" onClick={resetAndClose}>✕</button>
            </div>

            <p className="ens-subtitle">Mainnet · Mint a permanent <code>.repobox.eth</code> name</p>

            <div className="ens-step">
              <strong>1) Connect wallet</strong>
              <button className="ens-action" onClick={connectWallet} disabled={busy}>
                {wallet ? 'Wallet connected' : 'Connect Wallet'}
              </button>
              {wallet && <div className="ens-muted">{wallet}</div>}
            </div>

            <div className="ens-step">
              <strong>2) Price</strong>
              <div className="ens-price">{priceEth ? `${priceEth} ETH` : 'Connect wallet to load price'}</div>
            </div>

            <div className="ens-step">
              <strong>3) Choose name</strong>
              <div className="ens-name-row">
                <input
                  className="ens-input"
                  placeholder="your-name"
                  value={name}
                  onChange={(e) => {
                    setName(e.target.value.toLowerCase());
                    setAvailability('unknown');
                    setError(null);
                    setTxHash(null);
                  }}
                />
                <span className="ens-suffix">.repobox.eth</span>
              </div>
              <div className="ens-actions-row">
                <button className="ens-action" onClick={checkAvailability} disabled={busy || !normalizedName}>
                  Check availability
                </button>
                <button
                  className="ens-action ens-action-primary"
                  onClick={mintName}
                  disabled={busy || !wallet || !normalizedName || availability === 'taken'}
                >
                  {busy ? 'Waiting for wallet…' : 'Sign TX'}
                </button>
              </div>

              {availability === 'available' && <div className="ens-ok">✅ {fullName} is available</div>}
              {availability === 'taken' && <div className="ens-err">❌ {fullName} is already taken</div>}
            </div>

            {txHash && (
              <div className="ens-ok">
                ✅ Tx submitted: <a href={`https://etherscan.io/tx/${txHash}`} target="_blank" rel="noreferrer">{txHash.slice(0, 10)}…</a>
              </div>
            )}

            {error && <div className="ens-err">{error}</div>}

            <div className="ens-note">
              After minting: <strong>transfer the NFT to the right address, or use the resolvedAddress function</strong>.
            </div>
          </div>
        </div>
      )}

      <style>{`
        .ens-cta-btn {
          border: 1px solid rgba(240, 184, 96, 0.45);
          background: rgba(240, 184, 96, 0.12);
          color: var(--bp-gold);
          font-family: var(--font-mono), monospace;
          font-size: 12px;
          padding: 6px 10px;
          border-radius: 8px;
          cursor: pointer;
          transition: all .15s ease;
        }
        .ens-cta-btn:hover {
          background: rgba(240, 184, 96, 0.18);
          border-color: rgba(240, 184, 96, 0.6);
        }
        .ens-modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0,0,0,.55);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1200;
          padding: 16px;
        }
        .ens-modal {
          width: min(560px, 100%);
          background: var(--bp-surface);
          border: 1px solid var(--bp-border);
          border-radius: 12px;
          padding: 16px;
          box-shadow: 0 20px 60px rgba(0,0,0,.45);
        }
        .ens-modal-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 8px;
        }
        .ens-modal-header h3 {
          margin: 0;
          font-size: 16px;
          color: var(--bp-heading);
        }
        .ens-close {
          background: transparent;
          border: 1px solid var(--bp-border);
          color: var(--bp-dim);
          border-radius: 6px;
          cursor: pointer;
          width: 28px;
          height: 28px;
        }
        .ens-subtitle {
          margin: 0 0 12px 0;
          color: var(--bp-dim);
          font-size: 12px;
        }
        .ens-step {
          border: 1px solid var(--bp-border);
          background: var(--bp-bg-soft, rgba(255,255,255,0.02));
          border-radius: 8px;
          padding: 10px;
          margin-bottom: 10px;
          display: flex;
          flex-direction: column;
          gap: 8px;
        }
        .ens-action {
          border: 1px solid var(--bp-border);
          background: transparent;
          color: var(--bp-text);
          font-family: var(--font-mono), monospace;
          font-size: 12px;
          padding: 7px 10px;
          border-radius: 8px;
          cursor: pointer;
          width: fit-content;
        }
        .ens-action:disabled { opacity: .6; cursor: not-allowed; }
        .ens-action-primary {
          border-color: rgba(240, 184, 96, 0.45);
          color: var(--bp-gold);
          background: rgba(240, 184, 96, 0.10);
        }
        .ens-name-row {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        .ens-input {
          flex: 1;
          min-width: 0;
          border: 1px solid var(--bp-border);
          background: var(--bp-bg);
          color: var(--bp-text);
          font-family: var(--font-mono), monospace;
          font-size: 12px;
          padding: 8px;
          border-radius: 8px;
        }
        .ens-suffix {
          color: var(--bp-dim);
          font-size: 12px;
        }
        .ens-actions-row {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }
        .ens-muted {
          color: var(--bp-dim);
          font-size: 11px;
          overflow-wrap: anywhere;
        }
        .ens-price {
          color: var(--bp-gold);
          font-weight: 600;
        }
        .ens-ok {
          color: #55d38a;
          font-size: 12px;
        }
        .ens-err {
          color: #ff7a7a;
          font-size: 12px;
        }
        .ens-note {
          margin-top: 8px;
          color: var(--bp-dim);
          font-size: 12px;
          border-top: 1px dashed var(--bp-border);
          padding-top: 10px;
        }
      `}</style>
    </>
  );
}
