"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Database = void 0;
const fs_1 = require("fs");
const child_process_1 = require("child_process");
const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';
class Database {
    aliases;
    reverseMap;
    sqlitePath;
    constructor(jsonPath, sqlitePath) {
        this.aliases = new Map();
        this.reverseMap = new Map();
        this.sqlitePath = sqlitePath;
        try {
            const data = JSON.parse((0, fs_1.readFileSync)(jsonPath, 'utf8'));
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
        }
        catch (error) {
            console.error(`Failed to load aliases from ${jsonPath}:`, error);
        }
    }
    sqliteJsonQuery(sql) {
        if (!this.sqlitePath)
            return [];
        try {
            const out = (0, child_process_1.execFileSync)('sqlite3', ['-json', this.sqlitePath, sql], {
                encoding: 'utf8'
            });
            if (!out.trim())
                return [];
            return JSON.parse(out);
        }
        catch (error) {
            console.error('SQLite query failed:', error);
            return [];
        }
    }
    async resolveAlias(alias) {
        const norm = alias.toLowerCase();
        const indexed = this.sqliteJsonQuery(`SELECT lower(resolved_address) AS resolved_address, lower(owner_address) AS owner_address
             FROM ens_names_index
             WHERE lower(name) = '${norm.replace(/'/g, "''")}'
             LIMIT 1;`);
        if (indexed.length > 0) {
            const row = indexed[0];
            if (row.resolved_address && row.resolved_address !== ZERO_ADDRESS) {
                return row.resolved_address;
            }
            if (row.owner_address) {
                return row.owner_address;
            }
        }
        const data = this.aliases.get(norm);
        return data ? data.address : null;
    }
    reverseResolve(address) {
        const norm = address.toLowerCase();
        const byResolved = this.sqliteJsonQuery(`SELECT name FROM ens_names_index
             WHERE lower(resolved_address) = '${norm.replace(/'/g, "''")}'
               AND lower(resolved_address) != '${ZERO_ADDRESS}'
             ORDER BY token_id ASC
             LIMIT 1;`);
        if (byResolved.length > 0) {
            return { alias: byResolved[0].name, tier: 'purchased' };
        }
        const byOwner = this.sqliteJsonQuery(`SELECT name FROM ens_names_index
             WHERE lower(owner_address) = '${norm.replace(/'/g, "''")}'
             ORDER BY token_id ASC
             LIMIT 1;`);
        if (byOwner.length > 0) {
            return { alias: byOwner[0].name, tier: 'purchased' };
        }
        return this.reverseMap.get(norm) || null;
    }
    getAliasWithTier(alias) {
        const data = this.aliases.get(alias.toLowerCase());
        return data || null;
    }
    close() { }
}
exports.Database = Database;
//# sourceMappingURL=database.js.map