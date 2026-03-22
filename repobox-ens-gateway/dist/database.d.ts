export declare class Database {
    private aliases;
    constructor(dbPath: string);
    resolveAlias(alias: string): Promise<string | null>;
    close(): void;
}
//# sourceMappingURL=database.d.ts.map