"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NamesIndexer = void 0;
const child_process_1 = require("child_process");
const viem_1 = require("viem");
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
];
class NamesIndexer {
    sqlitePath;
    contract;
    client;
    constructor(opts) {
        this.sqlitePath = opts.sqlitePath;
        this.contract = opts.contractAddress;
        this.client = (0, viem_1.createPublicClient)({
            transport: (0, viem_1.http)(opts.rpcUrl),
        });
    }
    runSql(sql) {
        (0, child_process_1.execFileSync)('sqlite3', [this.sqlitePath, sql], { encoding: 'utf8' });
    }
    escapeSql(value) {
        return value.replace(/'/g, "''");
    }
    initSchema() {
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
    async syncOnce() {
        this.initSchema();
        const totalSupply = (await this.client.readContract({
            address: this.contract,
            abi: NAMES_ABI,
            functionName: 'totalSupply',
        }));
        let synced = 0;
        for (let tokenId = 1n; tokenId <= totalSupply; tokenId++) {
            try {
                const [name, owner, resolved] = await Promise.all([
                    this.client.readContract({
                        address: this.contract,
                        abi: NAMES_ABI,
                        functionName: 'tokenName',
                        args: [tokenId],
                    }),
                    this.client.readContract({
                        address: this.contract,
                        abi: NAMES_ABI,
                        functionName: 'ownerOf',
                        args: [tokenId],
                    }),
                    this.client.readContract({
                        address: this.contract,
                        abi: NAMES_ABI,
                        functionName: 'resolvedAddress',
                        args: [tokenId],
                    }),
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
            }
            catch (error) {
                console.error(`Failed syncing token ${tokenId.toString()}:`, error);
            }
        }
        return { total: Number(totalSupply), synced };
    }
}
exports.NamesIndexer = NamesIndexer;
//# sourceMappingURL=names-indexer.js.map