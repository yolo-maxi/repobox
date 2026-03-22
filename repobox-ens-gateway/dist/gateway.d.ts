import { type Address } from 'viem';
import { Database } from './database';
export declare class CCIPGateway {
    private database;
    private privateKey;
    private account;
    constructor(database: Database, privateKey: string);
    getSignerAddress(): Address;
    handleRequest(sender: string, dataHex: string): Promise<object>;
    private parseRequest;
    private extractAlias;
    private encodeAddressResult;
    private signResponse;
}
//# sourceMappingURL=gateway.d.ts.map