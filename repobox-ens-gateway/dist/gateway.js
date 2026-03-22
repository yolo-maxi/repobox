"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CCIPGateway = void 0;
const viem_1 = require("viem");
const accounts_1 = require("viem/accounts");
class CCIPGateway {
    database;
    privateKey;
    account;
    constructor(database, privateKey) {
        this.privateKey = privateKey.startsWith('0x') ? privateKey : `0x${privateKey}`;
        this.database = database;
        this.account = (0, accounts_1.privateKeyToAccount)(this.privateKey);
    }
    getSignerAddress() {
        return this.account.address;
    }
    async handleRequest(sender, dataHex) {
        try {
            const data = dataHex.startsWith('0x') ? dataHex : `0x${dataHex}`;
            const { name, funcCall } = this.parseRequest(data);
            const alias = this.extractAlias(name);
            console.log(`Resolving alias: ${alias}`);
            const address = await this.database.resolveAlias(alias);
            if (!address) {
                throw new Error(`Alias not found: ${alias}`);
            }
            console.log(`Resolved ${alias} → ${address}`);
            const result = this.encodeAddressResult(address);
            const signature = await this.signResponse(name, funcCall, result);
            const encodedResponse = (0, viem_1.encodeAbiParameters)((0, viem_1.parseAbiParameters)('bytes, bytes'), [result, signature]);
            return { data: encodedResponse };
        }
        catch (error) {
            console.error('Request handling error:', error);
            throw error;
        }
    }
    parseRequest(data) {
        try {
            const decoded = (0, viem_1.decodeAbiParameters)((0, viem_1.parseAbiParameters)('bytes name, bytes data'), data);
            return { name: decoded[0], funcCall: decoded[1] };
        }
        catch (error) {
            throw new Error(`Failed to parse request data: ${error}`);
        }
    }
    extractAlias(nameBytes) {
        try {
            const buffer = Buffer.from(nameBytes.slice(2), 'hex');
            let offset = 0;
            const labels = [];
            while (offset < buffer.length) {
                const len = buffer[offset];
                if (len === 0)
                    break;
                offset++;
                const label = buffer.slice(offset, offset + len).toString('utf8');
                labels.push(label);
                offset += len;
            }
            if (labels.length >= 3 && labels[labels.length - 2] === 'repobox' && labels[labels.length - 1] === 'eth') {
                return labels[0];
            }
            if (labels.length === 2 && labels[0] === 'repobox' && labels[1] === 'eth') {
                return 'repobox';
            }
            throw new Error(`Not a repobox.eth name: ${labels.join('.')}`);
        }
        catch (error) {
            throw new Error(`Failed to extract alias: ${error}`);
        }
    }
    encodeAddressResult(address) {
        const addressHex = address.startsWith('0x') ? address : `0x${address}`;
        return (0, viem_1.encodeAbiParameters)((0, viem_1.parseAbiParameters)('address'), [addressHex]);
    }
    async signResponse(name, data, result) {
        const packed = `0x${name.slice(2)}${data.slice(2)}${result.slice(2)}`;
        const messageHash = (0, viem_1.keccak256)(packed);
        const signature = await this.account.signMessage({
            message: { raw: messageHash }
        });
        return signature;
    }
}
exports.CCIPGateway = CCIPGateway;
//# sourceMappingURL=gateway.js.map