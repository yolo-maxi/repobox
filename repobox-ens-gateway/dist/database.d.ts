export declare class Database {
    private aliases;
    private reverseMap;
    constructor(dbPath: string);
    resolveAlias(alias: string): Promise<string | null>;
    reverseResolve(address: string): string | null;
    close(): void;
}
//# sourceMappingURL=database.d.ts.map