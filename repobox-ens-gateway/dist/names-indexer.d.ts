export declare class NamesIndexer {
    private sqlitePath;
    private contract;
    private client;
    constructor(opts: {
        sqlitePath: string;
        rpcUrl: string;
        contractAddress: string;
    });
    private runSql;
    private escapeSql;
    initSchema(): void;
    syncOnce(): Promise<{
        total: number;
        synced: number;
    }>;
}
//# sourceMappingURL=names-indexer.d.ts.map