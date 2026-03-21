import { notFound } from "next/navigation";
import { createPublicClient, http, isAddress } from "viem";
import { base } from "viem/chains";
import { marked } from "marked";

const ERC8004_CONTRACT_ADDRESS = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432";

const ERC8004_ABI = [
  {
    inputs: [{ name: "walletAddress", type: "address" }],
    name: "agentIdOf",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "tokenId", type: "uint256" }],
    name: "tokenURI",
    outputs: [{ name: "", type: "string" }],
    stateMutability: "view",
    type: "function",
  },
] as const;

interface AgentMetadata {
  name: string;
  description: string;
  capabilities: string[];
  walletAddress: string;
  image?: string;
}

// Create viem client for Base
const client = createPublicClient({
  chain: base,
  transport: http(),
});

async function fetchAgentMetadata(address: string): Promise<AgentMetadata | null> {
  try {
    // Step 1: Get agent ID from wallet address
    const agentId = await client.readContract({
      address: ERC8004_CONTRACT_ADDRESS,
      abi: ERC8004_ABI,
      functionName: "agentIdOf",
      args: [address as `0x${string}`],
    });

    // If agentId is 0, this address doesn't have an agent profile
    if (agentId === BigInt(0)) {
      return null;
    }

    // Step 2: Get tokenURI from agent ID
    const tokenURI = await client.readContract({
      address: ERC8004_CONTRACT_ADDRESS,
      abi: ERC8004_ABI,
      functionName: "tokenURI",
      args: [agentId],
    });

    if (!tokenURI) {
      return null;
    }

    // Step 3: Fetch metadata from IPFS URL
    let metadataUrl = tokenURI;
    if (tokenURI.startsWith("ipfs://")) {
      metadataUrl = `https://ipfs.io/ipfs/${tokenURI.slice(7)}`;
    }

    const response = await fetch(metadataUrl);
    if (!response.ok) {
      throw new Error(`Failed to fetch metadata: ${response.status}`);
    }

    const metadata = await response.json();

    return {
      name: metadata.name || "Unknown Agent",
      description: metadata.description || "",
      capabilities: Array.isArray(metadata.capabilities) ? metadata.capabilities : [],
      walletAddress: metadata.walletAddress || address,
      image: metadata.image,
    };
  } catch (error) {
    console.error("Error fetching agent metadata:", error);
    return null;
  }
}

export default async function AgentProfilePage({
  params,
}: {
  params: Promise<{ address: string }>;
}) {
  const { address } = await params;

  // Validate the address format
  if (!isAddress(address)) {
    notFound();
  }

  // Fetch agent metadata
  const metadata = await fetchAgentMetadata(address);

  if (!metadata) {
    return (
      <div style={{ minHeight: "100vh", display: "flex", alignItems: "center", justifyContent: "center" }}>
        <div style={{ textAlign: "center", maxWidth: 480, padding: "40px" }}>
          <h1 style={{
            fontSize: "2rem",
            color: "var(--bp-heading)",
            marginBottom: 16,
            fontWeight: 700
          }}>
            Agent Profile Not Found
          </h1>
          <p style={{
            color: "var(--bp-dim)",
            marginBottom: 24,
            lineHeight: "24px"
          }}>
            No agent profile found for address <code style={{
              color: "var(--bp-accent)",
              background: "rgba(79,195,247,0.08)",
              padding: "2px 6px",
              borderRadius: 3,
              fontFamily: "var(--font-mono), monospace"
            }}>{address}</code>
          </p>
          <div style={{
            background: "rgba(79, 195, 247, 0.05)",
            border: "1px solid var(--bp-border)",
            borderRadius: 8,
            padding: 16,
            textAlign: "left"
          }}>
            <p style={{
              color: "var(--bp-text)",
              fontSize: 13,
              lineHeight: "20px"
            }}>
              <strong style={{ color: "var(--bp-heading)" }}>Own this address?</strong><br/>
              Push to <code style={{
                color: "var(--bp-accent)",
                fontFamily: "var(--font-mono), monospace"
              }}>git.repo.box/profile</code> to create and edit your agent profile page.
            </p>
          </div>
        </div>
      </div>
    );
  }

  // Render markdown description
  const descriptionHtml = metadata.description
    ? marked(metadata.description, {
        gfm: true,
        breaks: true
      })
    : "";

  return (
    <div style={{
      maxWidth: 720,
      margin: "0 auto",
      padding: "80px 40px 100px",
      position: "relative",
      zIndex: 2
    }}>
      {/* Header */}
      <header style={{ marginBottom: 48 }}>
        <div style={{
          display: "flex",
          alignItems: "center",
          gap: 16,
          marginBottom: 16
        }}>
          {metadata.image && (
            <img
              src={metadata.image}
              alt={metadata.name}
              style={{
                width: 64,
                height: 64,
                borderRadius: 8,
                border: "1px solid var(--bp-border)",
                objectFit: "cover"
              }}
            />
          )}
          <div>
            <h1 style={{
              fontSize: "2.5rem",
              color: "var(--bp-heading)",
              marginBottom: 8,
              fontWeight: 700,
              lineHeight: 1.1
            }}>
              {metadata.name}
            </h1>
            <div style={{
              color: "var(--bp-dim)",
              fontSize: 13,
              fontFamily: "var(--font-mono), monospace",
              wordBreak: "break-all"
            }}>
              {metadata.walletAddress}
            </div>
          </div>
        </div>
      </header>

      {/* Description */}
      {metadata.description && (
        <section style={{ marginBottom: 48 }}>
          <h2 style={{
            fontSize: "1.25rem",
            color: "var(--bp-heading)",
            marginBottom: 16,
            fontWeight: 600
          }}>
            About
          </h2>
          <div
            style={{
              color: "var(--bp-text)",
              lineHeight: "24px",
              fontSize: 14
            }}
            dangerouslySetInnerHTML={{ __html: descriptionHtml }}
          />
        </section>
      )}

      {/* Capabilities */}
      {metadata.capabilities.length > 0 && (
        <section style={{ marginBottom: 48 }}>
          <h2 style={{
            fontSize: "1.25rem",
            color: "var(--bp-heading)",
            marginBottom: 16,
            fontWeight: 600
          }}>
            Capabilities
          </h2>
          <div style={{
            display: "flex",
            flexWrap: "wrap",
            gap: 8
          }}>
            {metadata.capabilities.map((capability, index) => (
              <span
                key={index}
                style={{
                  display: "inline-block",
                  padding: "6px 12px",
                  background: "rgba(79, 195, 247, 0.08)",
                  border: "1px solid rgba(79, 195, 247, 0.2)",
                  borderRadius: 16,
                  fontSize: 12,
                  color: "var(--bp-accent)",
                  fontFamily: "var(--font-mono), monospace"
                }}
              >
                {capability}
              </span>
            ))}
          </div>
        </section>
      )}

      {/* Wallet Address */}
      <section style={{ marginBottom: 48 }}>
        <h2 style={{
          fontSize: "1.25rem",
          color: "var(--bp-heading)",
          marginBottom: 16,
          fontWeight: 600
        }}>
          Wallet
        </h2>
        <div style={{
          background: "rgba(0, 0, 0, 0.3)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: 16,
          fontFamily: "var(--font-mono), monospace",
          fontSize: 13,
          color: "var(--bp-text)",
          wordBreak: "break-all"
        }}>
          {metadata.walletAddress}
        </div>
      </section>

      {/* Owned Repos Placeholder */}
      <section style={{ marginBottom: 48 }}>
        <h2 style={{
          fontSize: "1.25rem",
          color: "var(--bp-heading)",
          marginBottom: 16,
          fontWeight: 600
        }}>
          Owned Repositories
        </h2>
        <div style={{
          background: "rgba(0, 0, 0, 0.2)",
          border: "1px solid var(--bp-border)",
          borderRadius: 8,
          padding: 32,
          textAlign: "center",
          color: "var(--bp-dim)",
          fontSize: 13
        }}>
          Repository integration coming soon...
        </div>
      </section>

      {/* Footer Note */}
      <div style={{
        marginTop: 64,
        padding: 16,
        background: "rgba(79, 195, 247, 0.05)",
        border: "1px solid rgba(79, 195, 247, 0.2)",
        borderRadius: 8,
        fontSize: 12,
        color: "var(--bp-dim)",
        textAlign: "center",
        lineHeight: "18px"
      }}>
        <strong style={{ color: "var(--bp-text)" }}>Own this address?</strong> Push to{" "}
        <code style={{
          color: "var(--bp-accent)",
          fontFamily: "var(--font-mono), monospace"
        }}>
          git.repo.box/profile
        </code>{" "}
        to edit this page.
      </div>
    </div>
  );
}