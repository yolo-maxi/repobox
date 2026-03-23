import { execFileSync } from 'child_process';
import { createPublicClient, http, type Address } from 'viem';

const NAMES_ABI = [
  {
    type: 'function',
    name: 'totalSupply',
    stateMutability: 'view',
    inputs: [],
    outputs: [{ type: 'uint256' }],
  },
  {
    type: 'function',
    name: 'tokenName',
    stateMutability: 'view',
    inputs: [{ name: 'tokenId', type: 'uint256' }],
    outputs: [{ type: 'string' }],
  },
  {
    type: 'function',
    name: 'resolvedAddress',
    stateMutability: 'view',
    inputs: [{ name: 'tokenId', type: 'uint256' }],
    outputs: [{ type: 'address' }],
  },
  {
    type: 'function',
    name: 'ownerOf',
    stateMutability: 'view',
    inputs: [{ name: 'tokenId', type: 'uint256' }],
    outputs: [{ type: 'address' }],
  },
] as const;

export class NamesIndexer {
  private sqlitePath: string;
  private contract: Address;
  private client: ReturnType<typeof createPublicClient>;

  constructor(opts: { sqlitePath: string; rpcUrl: string; contractAddress: string }) {
    this.sqlitePath = opts.sqlitePath;
    this.contract = opts.contractAddress as Address;
    this.client = createPublicClient({
      transport: http(opts.rpcUrl),
    });
  }

  private runSql(sql: string): void {
    execFileSync('sqlite3', [this.sqlitePath, sql], { encoding: 'utf8' });
  }

  private escapeSql(value: string): string {
    return value.replace(/'/g, "''");
  }

  initSchema(): void {
    this.runSql(`
CREATE TABLE IF NOT EXISTS ens_names_index (
  token_id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  owner_address TEXT NOT NULL,
  resolved_address TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ens_names_index_resolved ON ens_names_index(lower(resolved_address));
CREATE INDEX IF NOT EXISTS idx_ens_names_index_owner ON ens_names_index(lower(owner_address));
`);
  }

  async syncOnce(): Promise<{ total: number; synced: number }> {
    this.initSchema();

    const totalSupply = (await this.client.readContract({
      address: this.contract,
      abi: NAMES_ABI,
      functionName: 'totalSupply',
    })) as bigint;

    let synced = 0;

    for (let tokenId = 1n; tokenId <= totalSupply; tokenId++) {
      try {
        const [name, owner, resolved] = await Promise.all([
          this.client.readContract({
            address: this.contract,
            abi: NAMES_ABI,
            functionName: 'tokenName',
            args: [tokenId],
          }) as Promise<string>,
          this.client.readContract({
            address: this.contract,
            abi: NAMES_ABI,
            functionName: 'ownerOf',
            args: [tokenId],
          }) as Promise<Address>,
          this.client.readContract({
            address: this.contract,
            abi: NAMES_ABI,
            functionName: 'resolvedAddress',
            args: [tokenId],
          }) as Promise<Address>,
        ]);

        const sql = `
INSERT INTO ens_names_index (token_id, name, owner_address, resolved_address, updated_at)
VALUES (${tokenId.toString()}, '${this.escapeSql(name.toLowerCase())}', '${owner.toLowerCase()}', '${resolved.toLowerCase()}', datetime('now'))
ON CONFLICT(token_id) DO UPDATE SET
  name = excluded.name,
  owner_address = excluded.owner_address,
  resolved_address = excluded.resolved_address,
  updated_at = datetime('now');
`;
        this.runSql(sql);
        synced++;
      } catch (error) {
        console.error(`Failed syncing token ${tokenId.toString()}:`, error);
      }
    }

    return { total: Number(totalSupply), synced };
  }
}
