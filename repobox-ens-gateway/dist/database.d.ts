export declare class Database {
    private aliases;
    private reverseMap;
    constructor(dbPath: string);
    resolveAlias(alias: string): Promise<string | null>;
    reverseResolve(address: string): {
        alias: string;
        tier: string;
    } | null;
    getAliasWithTier(alias: string): {
        address: string;
        tier: string;
    } | null;
    close(): void;
}
//# sourceMappingURL=database.d.ts.map