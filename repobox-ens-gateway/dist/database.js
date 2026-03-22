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
                this.aliases.set(entry.alias.toLowerCase(), entry.address);
                const addrLower = entry.address.toLowerCase();
                if (!this.reverseMap.has(addrLower)) {
                    this.reverseMap.set(addrLower, entry.alias);
                }
            }
            console.log(`Loaded ${this.aliases.size} aliases`);
        }
        catch (error) {
            console.error(`Failed to load aliases from ${dbPath}:`, error);
        }
    }
    async resolveAlias(alias) {
        return this.aliases.get(alias.toLowerCase()) || null;
    }
    reverseResolve(address) {
        return this.reverseMap.get(address.toLowerCase()) || null;
    }
    close() { }
}
exports.Database = Database;
//# sourceMappingURL=database.js.map