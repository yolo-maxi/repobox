"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Database = void 0;
const fs_1 = require("fs");
class Database {
    aliases;
    reverseMap;
    constructor(dbPath) {
        this.aliases = new Map();
        this.reverseMap = new Map();
        try {
            const data = JSON.parse((0, fs_1.readFileSync)(dbPath, 'utf8'));
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
            console.log(`Loaded ${this.aliases.size} aliases`);
        }
        catch (error) {
            console.error(`Failed to load aliases from ${dbPath}:`, error);
        }
    }
    async resolveAlias(alias) {
        const data = this.aliases.get(alias.toLowerCase());
        return data ? data.address : null;
    }
    reverseResolve(address) {
        return this.reverseMap.get(address.toLowerCase()) || null;
    }
    getAliasWithTier(alias) {
        const data = this.aliases.get(alias.toLowerCase());
        return data || null;
    }
    close() { }
}
exports.Database = Database;
//# sourceMappingURL=database.js.map