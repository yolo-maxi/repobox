"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const dotenv_1 = __importDefault(require("dotenv"));
const gateway_1 = require("./gateway");
const database_1 = require("./database");
const names_indexer_1 = require("./names-indexer");
dotenv_1.default.config();
const app = (0, express_1.default)();
const port = process.env.PORT || 3001;
app.use((0, cors_1.default)());
app.use(express_1.default.json());
const dbPath = process.env.DB_PATH || './data/aliases.json';
const namesIndexDbPath = process.env.NAMES_INDEX_DB_PATH || '/var/lib/repobox/repos/repobox.db';
const namesContractAddress = process.env.NAMES_CONTRACT_ADDRESS;
const rpcUrl = process.env.RPC_URL || 'https://eth.drpc.org';
const privateKey = process.env.GATEWAY_PRIVATE_KEY;
if (!privateKey) {
    console.error('GATEWAY_PRIVATE_KEY environment variable is required');
    process.exit(1);
}
const database = new database_1.Database(dbPath, namesIndexDbPath);
const gateway = new gateway_1.CCIPGateway(database, privateKey);
let namesIndexer = null;
if (namesContractAddress) {
    namesIndexer = new names_indexer_1.NamesIndexer({
        sqlitePath: namesIndexDbPath,
        rpcUrl,
        contractAddress: namesContractAddress,
    });
}
app.get('/health', (req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
});
app.get('/resolve/:alias', async (req, res) => {
    try {
        const { alias } = req.params;
        const result = database.getAliasWithTier(alias);
        if (result) {
            res.json({ alias, address: result.address, tier: result.tier });
        }
        else {
            res.status(404).json({ error: 'Alias not found' });
        }
    }
    catch (error) {
        res.status(500).json({ error: 'Internal server error' });
    }
});
app.get('/reverse/:address', (req, res) => {
    try {
        const { address } = req.params;
        const result = database.reverseResolve(address);
        if (result) {
            res.json({ alias: result.alias, address, tier: result.tier });
        }
        else {
            res.status(404).json({ error: 'No alias found' });
        }
    }
    catch (error) {
        res.status(500).json({ error: 'Internal server error' });
    }
});
app.post('/reverse', (req, res) => {
    try {
        const { addresses } = req.body;
        if (!Array.isArray(addresses)) {
            return res.status(400).json({ error: 'addresses must be an array' });
        }
        const results = {};
        for (const addr of addresses) {
            const result = database.reverseResolve(addr);
            results[addr.toLowerCase()] = result ? { alias: result.alias, tier: result.tier } : null;
        }
        res.json(results);
    }
    catch (error) {
        res.status(500).json({ error: 'Internal server error' });
    }
});
app.get('/:sender/:data.json', async (req, res) => {
    try {
        const { sender, data } = req.params;
        console.log(`CCIP request: sender=${sender}, data=${data}`);
        const response = await gateway.handleRequest(sender, data);
        res.json(response);
    }
    catch (error) {
        console.error('Gateway error:', error);
        res.status(500).json({
            error: 'Internal server error',
            message: error instanceof Error ? error.message : 'Unknown error'
        });
    }
});
app.listen(port, async () => {
    console.log(`🚀 CCIP Gateway server running on port ${port}`);
    console.log(`📂 Using aliases JSON: ${dbPath}`);
    console.log(`🗄️ Using names index SQLite: ${namesIndexDbPath}`);
    console.log(`🔑 Gateway signer: ${gateway.getSignerAddress()}`);
    if (!namesIndexer) {
        console.warn('⚠️ NAMES_CONTRACT_ADDRESS not configured; on-chain names indexing disabled');
        return;
    }
    try {
        const result = await namesIndexer.syncOnce();
        console.log(`✅ On-chain names index sync complete: ${result.synced}/${result.total}`);
    }
    catch (error) {
        console.error('❌ Initial on-chain names index sync failed:', error);
    }
    const intervalMs = Number(process.env.NAMES_SYNC_INTERVAL_MS || 300000);
    setInterval(async () => {
        if (!namesIndexer)
            return;
        try {
            const result = await namesIndexer.syncOnce();
            console.log(`🔄 On-chain names index sync: ${result.synced}/${result.total}`);
        }
        catch (error) {
            console.error('❌ Periodic on-chain names index sync failed:', error);
        }
    }, intervalMs);
});
exports.default = app;
//# sourceMappingURL=index.js.map