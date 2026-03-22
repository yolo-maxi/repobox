import { readFileSync } from 'fs';
import { join } from 'path';

interface AliasEntry {
  alias: string;
  address: string;
}

export class Database {
    private aliases: Map<string, string>;

    constructor(dbPath: string) {
        this.aliases = new Map();
        try {
            const data = JSON.parse(readFileSync(dbPath, 'utf8')) as AliasEntry[];
            for (const entry of data) {
                this.aliases.set(entry.alias.toLowerCase(), entry.address);
            }
            console.log(`Loaded ${this.aliases.size} aliases`);
        } catch (error) {
            console.error(`Failed to load aliases from ${dbPath}:`, error);
        }
    }

    async resolveAlias(alias: string): Promise<string | null> {
        return this.aliases.get(alias.toLowerCase()) || null;
    }

    close(): void {}
}
