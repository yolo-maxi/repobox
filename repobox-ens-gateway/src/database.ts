import { readFileSync } from 'fs';
import { join } from 'path';

interface AliasEntry {
  alias: string;
  address: string;
}

export class Database {
    private aliases: Map<string, string>;
    private reverseMap: Map<string, string>; // address -> alias

    constructor(dbPath: string) {
        this.aliases = new Map();
        this.reverseMap = new Map();
        try {
            const data = JSON.parse(readFileSync(dbPath, 'utf8')) as AliasEntry[];
            for (const entry of data) {
                this.aliases.set(entry.alias.toLowerCase(), entry.address);
                // For reverse: first alias wins (don't overwrite with 'repobox' root alias)
                const addrLower = entry.address.toLowerCase();
                if (!this.reverseMap.has(addrLower)) {
                    this.reverseMap.set(addrLower, entry.alias);
                }
            }
            console.log(`Loaded ${this.aliases.size} aliases`);
        } catch (error) {
            console.error(`Failed to load aliases from ${dbPath}:`, error);
        }
    }

    async resolveAlias(alias: string): Promise<string | null> {
        return this.aliases.get(alias.toLowerCase()) || null;
    }

    reverseResolve(address: string): string | null {
        return this.reverseMap.get(address.toLowerCase()) || null;
    }

    close(): void {}
}
