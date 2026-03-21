import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import { CCIPGateway } from './gateway';
import { Database } from './database';

dotenv.config();

const app = express();
const port = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());

// Initialize database and gateway
const dbPath = process.env.DB_PATH || '../repobox-server/repobox.db';
const privateKey = process.env.GATEWAY_PRIVATE_KEY;

if (!privateKey) {
    console.error('GATEWAY_PRIVATE_KEY environment variable is required');
    process.exit(1);
}

const database = new Database(dbPath);
const gateway = new CCIPGateway(database, privateKey);

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// Main CCIP gateway endpoint: /{sender}/{data}.json
app.get('/:sender/:data.json', async (req, res) => {
    try {
        const { sender, data } = req.params;

        console.log(`CCIP request: sender=${sender}, data=${data}`);

        const response = await gateway.handleRequest(sender, data);

        res.json(response);
    } catch (error) {
        console.error('Gateway error:', error);
        res.status(500).json({
            error: 'Internal server error',
            message: error instanceof Error ? error.message : 'Unknown error'
        });
    }
});

// Start server
app.listen(port, () => {
    console.log(`🚀 CCIP Gateway server running on port ${port}`);
    console.log(`📂 Using database: ${dbPath}`);
    console.log(`🔑 Gateway signer: ${gateway.getSignerAddress()}`);
});

export default app;