import {
    decodeAbiParameters,
    encodeAbiParameters,
    keccak256,
    toHex,
    parseAbiParameters,
    type Address,
    type Hex
} from 'viem';
import { privateKeyToAccount } from 'viem/accounts';
import { Database } from './database';

/**
 * CCIP Gateway for repo.box ENS resolution
 */
export class CCIPGateway {
    private database: Database;
    private privateKey: Hex;
    private account: ReturnType<typeof privateKeyToAccount>;

    constructor(database: Database, privateKey: string) {
        this.privateKey = privateKey.startsWith('0x') ? privateKey as Hex : `0x${privateKey}` as Hex;
        this.database = database;
        this.account = privateKeyToAccount(this.privateKey);
    }

    getSignerAddress(): Address {
        return this.account.address;
    }

    async handleRequest(sender: string, dataHex: string): Promise<object> {
        try {
            const data = dataHex.startsWith('0x') ? dataHex as Hex : `0x${dataHex}` as Hex;
            const { name, funcCall } = this.parseRequest(data);
            const alias = this.extractAlias(name);

            console.log(`Resolving alias: ${alias}`);

            const address = await this.database.resolveAlias(alias);
            if (!address) {
                throw new Error(`Alias not found: ${alias}`);
            }

            console.log(`Resolved ${alias} → ${address}`);

            const result = this.encodeAddressResult(address);
            
            // Sign: keccak256(name ++ data ++ result) — matching RepoBoxResolver.sol
            const signature = await this.signResponse(name, funcCall, result);
            
            // ABI-encode: (bytes result, bytes signature) — matching resolveWithProof decoder
            const encodedResponse = encodeAbiParameters(
                parseAbiParameters('bytes, bytes'),
                [result, signature]
            );

            return { data: encodedResponse };
        } catch (error) {
            console.error('Request handling error:', error);
            throw error;
        }
    }

    private parseRequest(data: Hex): { name: Hex, funcCall: Hex } {
        try {
            const decoded = decodeAbiParameters(
                parseAbiParameters('bytes name, bytes data'),
                data
            );
            return { name: decoded[0] as Hex, funcCall: decoded[1] as Hex };
        } catch (error) {
            throw new Error(`Failed to parse request data: ${error}`);
        }
    }

    private extractAlias(nameBytes: Hex): string {
        try {
            const buffer = Buffer.from(nameBytes.slice(2), 'hex');
            let offset = 0;
            const labels: string[] = [];

            while (offset < buffer.length) {
                const len = buffer[offset];
                if (len === 0) break;
                offset++;
                const label = buffer.slice(offset, offset + len).toString('utf8');
                labels.push(label);
                offset += len;
            }

            // repobox.eth subnames: ocean.repobox.eth → labels = ['ocean', 'repobox', 'eth']
            if (labels.length >= 3 && labels[labels.length - 2] === 'repobox' && labels[labels.length - 1] === 'eth') {
                return labels[0];
            }

            // Root name: repobox.eth → return null/root indicator
            if (labels.length === 2 && labels[0] === 'repobox' && labels[1] === 'eth') {
                return 'repobox'; // resolve root to a known identity
            }

            throw new Error(`Not a repobox.eth name: ${labels.join('.')}`);
        } catch (error) {
            throw new Error(`Failed to extract alias: ${error}`);
        }
    }

    private encodeAddressResult(address: string): Hex {
        const addressHex = address.startsWith('0x') ? address : `0x${address}`;
        return encodeAbiParameters(
            parseAbiParameters('address'),
            [addressHex as Address]
        );
    }

    private async signResponse(name: Hex, data: Hex, result: Hex): Promise<Hex> {
        // Match RepoBoxResolver.sol: keccak256(abi.encodePacked(name, data, result))
        // Using concat of raw bytes (encodePacked)
        const packed = `0x${name.slice(2)}${data.slice(2)}${result.slice(2)}` as Hex;
        const messageHash = keccak256(packed);

        // The contract uses: ecrecover(keccak256("\x19Ethereum Signed Message:\n32", messageHash), ...)
        // account.signMessage automatically adds the EIP-191 prefix
        const signature = await this.account.signMessage({
            message: { raw: messageHash as `0x${string}` }
        });

        return signature;
    }
}
