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
dotenv_1.default.config();
const app = (0, express_1.default)();
const port = process.env.PORT || 3001;
app.use((0, cors_1.default)());
app.use(express_1.default.json());
const dbPath = process.env.DB_PATH || '../repobox-server/repobox.db';
const privateKey = process.env.GATEWAY_PRIVATE_KEY;
if (!privateKey) {
    console.error('GATEWAY_PRIVATE_KEY environment variable is required');
    process.exit(1);
}
const database = new database_1.Database(dbPath);
const gateway = new gateway_1.CCIPGateway(database, privateKey);
app.get('/health', (req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
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
app.listen(port, () => {
    console.log(`🚀 CCIP Gateway server running on port ${port}`);
    console.log(`📂 Using database: ${dbPath}`);
    console.log(`🔑 Gateway signer: ${gateway.getSignerAddress()}`);
});
exports.default = app;
//# sourceMappingURL=index.js.map