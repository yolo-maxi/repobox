import { readFileSync } from 'fs';
import { execFileSync } from 'child_process';

interface AliasEntry {
  alias: string;
  address: string;
  tier?: string;
}

interface AliasData {
  address: string;
  tier: string;
}

interface ReverseResult {
  alias: string;
  tier: string;
}

const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

export class Database {
    private aliases: Map<string, AliasData>;
    private reverseMap: Map<string, { alias: string; tier: string }>;
    private sqlitePath?: string;

    constructor(jsonPath: string, sqlitePath?: string) {
        this.aliases = new Map();
        this.reverseMap = new Map();
        this.sqlitePath = sqlitePath;

        try {
            const data = JSON.parse(readFileSync(jsonPath, 'utf8')) as AliasEntry[];
            for (const entry of data) {
                const aliasData = {
                    address: entry.address,
                    tier: entry.tier || 'auto-alias'
                };
                this.aliases.set(entry.alias.toLowerCase(), aliasData);

                const addrLower = entry.address.toLowerCase();
                const existing = this.reverseMap.get(addrLower);
                if (!existing || (entry.tier === 'purchased' && existing.tier !== 'purchased')) {
                    this.reverseMap.set(addrLower, {
                        alias: entry.alias,
                        tier: entry.tier || 'auto-alias'
                    });
                }
            }
            console.log(`Loaded ${this.aliases.size} aliases from JSON fallback`);
        } catch (error) {
            console.error(`Failed to load aliases from ${jsonPath}:`, error);
        }
    }

    private sqliteJsonQuery<T = any>(sql: string): T[] {
        if (!this.sqlitePath) return [];
        try {
            const out = execFileSync('sqlite3', ['-json', this.sqlitePath, sql], {
                encoding: 'utf8'
            });
            if (!out.trim()) return [];
            return JSON.parse(out) as T[];
        } catch (error) {
            console.error('SQLite query failed:', error);
            return [];
        }
    }

    async resolveAlias(alias: string): Promise<string | null> {
        const norm = alias.toLowerCase();

        // Primary: on-chain indexed names table (purchased names)
        const indexed = this.sqliteJsonQuery<{ resolved_address: string; owner_address: string }>(
            `SELECT lower(resolved_address) AS resolved_address, lower(owner_address) AS owner_address
             FROM ens_names_index
             WHERE lower(name) = '${norm.replace(/'/g, "''")}'
             LIMIT 1;`
        );

        if (indexed.length > 0) {
            const row = indexed[0];
            if (row.resolved_address && row.resolved_address !== ZERO_ADDRESS) {
                return row.resolved_address;
            }
            if (row.owner_address) {
                return row.owner_address;
            }
        }

        // Fallback: static aliases.json
        const data = this.aliases.get(norm);
        return data ? data.address : null;
    }

    reverseResolve(address: string): ReverseResult | null {
        const norm = address.toLowerCase();

        // Priority 1: reverse by resolved_address
        const byResolved = this.sqliteJsonQuery<{ name: string }>(
            `SELECT name FROM ens_names_index
             WHERE lower(resolved_address) = '${norm.replace(/'/g, "''")}'
               AND lower(resolved_address) != '${ZERO_ADDRESS}'
             ORDER BY token_id ASC
             LIMIT 1;`
        );
        if (byResolved.length > 0) {
            return { alias: byResolved[0].name, tier: 'purchased' };
        }

        // Priority 2: fallback reverse by owner_address
        const byOwner = this.sqliteJsonQuery<{ name: string }>(
            `SELECT name FROM ens_names_index
             WHERE lower(owner_address) = '${norm.replace(/'/g, "''")}'
             ORDER BY token_id ASC
             LIMIT 1;`
        );
        if (byOwner.length > 0) {
            return { alias: byOwner[0].name, tier: 'purchased' };
        }

        // Fallback: static aliases.json reverse map
        return this.reverseMap.get(norm) || null;
    }

    getAliasWithTier(alias: string): { address: string; tier: string } | null {
        const data = this.aliases.get(alias.toLowerCase());
        return data || null;
    }

    close(): void {}
}
