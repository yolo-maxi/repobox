import { createHash, createSign } from 'crypto';
import {
    decodeAbiParameters,
    encodeAbiParameters,
    keccak256,
    toHex,
    parseAbiParameters,
    createWalletClient,
    privateKeyToAccount,
    http,
    Address,
    Hex
} from 'viem';
import { mainnet } from 'viem/chains';
import { Database } from './database';

/**
 * CCIP Gateway for repo.box ENS resolution
 */
export class CCIPGateway {
    private database: Database;
    private privateKey: Hex;
    private account: any;

    constructor(database: Database, privateKey: string) {
        this.database = database;
        this.privateKey = privateKey.startsWith('0x') ? privateKey as Hex : `0x${privateKey}` as Hex;
        this.account = privateKeyToAccount(this.privateKey);
    }

    /**
     * Get the signer address for this gateway
     */
    getSignerAddress(): Address {
        return this.account.address;
    }

    /**
     * Handle a CCIP request from the resolver contract
     * @param sender The sender address (resolver contract)
     * @param dataHex The encoded request data
     * @returns The signed response object
     */
    async handleRequest(sender: string, dataHex: string): Promise<object> {
        try {
            // Decode the request data
            const data = dataHex.startsWith('0x') ? dataHex as Hex : `0x${dataHex}` as Hex;

            // Parse the CCIP request to extract the ENS name and function call
            const { name, funcCall } = this.parseRequest(data);

            // Extract the alias from the ENS name
            const alias = this.extractAlias(name);

            console.log(`Resolving alias: ${alias}`);

            // Look up the address in the database
            const address = await this.database.resolveAlias(alias);

            if (!address) {
                throw new Error(`Alias not found: ${alias}`);
            }

            console.log(`Resolved ${alias} → ${address}`);

            // Encode the result (address as bytes32 for addr() function)
            const result = this.encodeAddressResult(address);

            // Sign the response
            const signature = await this.signResponse(name, funcCall, result);

            // Return the signed response
            return {
                data: toHex(result),
                signature: signature
            };

        } catch (error) {
            console.error('Request handling error:', error);
            throw error;
        }
    }

    /**
     * Parse the CCIP request to extract name and function call data
     */
    private parseRequest(data: Hex): { name: Hex, funcCall: Hex } {
        try {
            // The data contains the encoded resolve() call with name and funcCall parameters
            // resolve(bytes name, bytes data)
            const decoded = decodeAbiParameters(
                parseAbiParameters('bytes name, bytes data'),
                data
            );

            return {
                name: decoded[0] as Hex,
                funcCall: decoded[1] as Hex
            };
        } catch (error) {
            throw new Error(`Failed to parse request data: ${error}`);
        }
    }

    /**
     * Extract the repo.box alias from an ENS name
     */
    private extractAlias(nameBytes: Hex): string {
        try {
            // Convert bytes to string and extract the subdomain
            // For ENS names like "clever-green-canyon.repo.box", we want "clever-green-canyon"
            const nameStr = Buffer.from(nameBytes.slice(2), 'hex').toString('utf8');

            // Handle DNS encoding where each label is prefixed with its length
            if (nameBytes.length > 2) {
                // DNS-encoded name: parse the labels
                const buffer = Buffer.from(nameBytes.slice(2), 'hex');
                let offset = 0;
                const labels: string[] = [];

                while (offset < buffer.length) {
                    const len = buffer[offset];
                    if (len === 0) break; // End of name

                    offset++;
                    const label = buffer.slice(offset, offset + len).toString('utf8');
                    labels.push(label);
                    offset += len;
                }

                // For repo.box subdomains, we want the first label
                if (labels.length >= 2 && labels[labels.length - 2] === 'repo' && labels[labels.length - 1] === 'box') {
                    return labels[0];
                }
            }

            // Fallback: simple string parsing
            const parts = nameStr.split('.');
            if (parts.length >= 3 && parts[parts.length - 2] === 'repo' && parts[parts.length - 1] === 'box') {
                return parts[0];
            }

            throw new Error(`Invalid repo.box subdomain: ${nameStr}`);

        } catch (error) {
            throw new Error(`Failed to extract alias: ${error}`);
        }
    }

    /**
     * Encode an Ethereum address as the result for addr() function
     */
    private encodeAddressResult(address: string): Uint8Array {
        // For addr() function, return the address as bytes
        const addressHex = address.startsWith('0x') ? address : `0x${address}`;
        return encodeAbiParameters(
            parseAbiParameters('address'),
            [addressHex as Address]
        );
    }

    /**
     * Sign the gateway response
     */
    private async signResponse(name: Hex, funcCall: Hex, result: Uint8Array): Promise<Hex> {
        // Create the message hash for signing
        const messageHash = keccak256(
            encodeAbiParameters(
                parseAbiParameters('bytes, bytes, bytes'),
                [name, funcCall, toHex(result)]
            )
        );

        // Sign the message hash with the account
        const signature = await this.account.signMessage({
            message: { raw: messageHash }
        });

        return signature;
    }
}