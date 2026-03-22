import { readFileSync } from 'fs';
import { join } from 'path';

interface AliasEntry {
  alias: string;
  address: string;
  tier?: string;
}

interface AliasData {
  address: string;
  tier: string;
}

export class Database {
    private aliases: Map<string, AliasData>;
    private reverseMap: Map<string, { alias: string; tier: string }>; // address -> { alias, tier }

    constructor(dbPath: string) {
        this.aliases = new Map();
        this.reverseMap = new Map();
        try {
            const data = JSON.parse(readFileSync(dbPath, 'utf8')) as AliasEntry[];
            for (const entry of data) {
                const aliasData = {
                    address: entry.address,
                    tier: entry.tier || 'auto-alias'
                };
                this.aliases.set(entry.alias.toLowerCase(), aliasData);
                
                // For reverse: prioritize purchased > auto-alias
                const addrLower = entry.address.toLowerCase();
                const existing = this.reverseMap.get(addrLower);
                if (!existing || (entry.tier === 'purchased' && existing.tier !== 'purchased')) {
                    this.reverseMap.set(addrLower, {
                        alias: entry.alias,
                        tier: entry.tier || 'auto-alias'
                    });
                }
            }
            console.log(`Loaded ${this.aliases.size} aliases`);
        } catch (error) {
            console.error(`Failed to load aliases from ${dbPath}:`, error);
        }
    }

    async resolveAlias(alias: string): Promise<string | null> {
        const data = this.aliases.get(alias.toLowerCase());
        return data ? data.address : null;
    }

    reverseResolve(address: string): { alias: string; tier: string } | null {
        return this.reverseMap.get(address.toLowerCase()) || null;
    }

    getAliasWithTier(alias: string): { address: string; tier: string } | null {
        const data = this.aliases.get(alias.toLowerCase());
        return data || null;
    }

    close(): void {}
}
