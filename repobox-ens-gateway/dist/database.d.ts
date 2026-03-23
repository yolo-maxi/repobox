interface ReverseResult {
    alias: string;
    tier: string;
}
export declare class Database {
    private aliases;
    private reverseMap;
    private sqlitePath?;
    constructor(jsonPath: string, sqlitePath?: string);
    private sqliteJsonQuery;
    resolveAlias(alias: string): Promise<string | null>;
    reverseResolve(address: string): ReverseResult | null;
    getAliasWithTier(alias: string): {
        address: string;
        tier: string;
    } | null;
    close(): void;
}
export {};
//# sourceMappingURL=database.d.ts.map