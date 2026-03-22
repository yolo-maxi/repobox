import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import { CCIPGateway } from './gateway';
import { Database } from './database';

dotenv.config();

const app: ReturnType<typeof express> = express();
const port = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());

// Initialize database and gateway
const dbPath = process.env.DB_PATH || './data/aliases.json';
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

// Forward lookup: alias -> address
app.get('/resolve/:alias', async (req, res) => {
    try {
        const { alias } = req.params;
        const result = database.getAliasWithTier(alias);
        if (result) {
            res.json({ alias, address: result.address, tier: result.tier });
        } else {
            res.status(404).json({ error: 'Alias not found' });
        }
    } catch (error) {
        res.status(500).json({ error: 'Internal server error' });
    }
});

// Reverse lookup: address -> alias  
app.get('/reverse/:address', (req, res) => {
    try {
        const { address } = req.params;
        const result = database.reverseResolve(address);
        if (result) {
            res.json({ alias: result.alias, address, tier: result.tier });
        } else {
            res.status(404).json({ error: 'No alias found' });
        }
    } catch (error) {
        res.status(500).json({ error: 'Internal server error' });
    }
});

// Bulk reverse: POST /reverse with { addresses: string[] }
app.post('/reverse', (req, res) => {
    try {
        const { addresses } = req.body;
        if (!Array.isArray(addresses)) {
            return res.status(400).json({ error: 'addresses must be an array' });
        }
        const results: Record<string, { alias: string; tier: string } | null> = {};
        for (const addr of addresses) {
            const result = database.reverseResolve(addr);
            results[addr.toLowerCase()] = result ? { alias: result.alias, tier: result.tier } : null;
        }
        res.json(results);
    } catch (error) {
        res.status(500).json({ error: 'Internal server error' });
    }
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
