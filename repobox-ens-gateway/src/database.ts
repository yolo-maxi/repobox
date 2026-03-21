import sqlite3 from 'sqlite3';
import { promisify } from 'util';

/**
 * Database wrapper for accessing repo.box alias data
 */
export class Database {
    private db: sqlite3.Database;

    constructor(dbPath: string) {
        this.db = new sqlite3.Database(dbPath);
    }

    /**
     * Resolve an alias to an Ethereum address
     * @param alias The alias to resolve (e.g., "clever-green-canyon")
     * @returns The Ethereum address or null if not found
     */
    async resolveAlias(alias: string): Promise<string | null> {
        return new Promise((resolve, reject) => {
            this.db.get(
                'SELECT address FROM aliases WHERE alias = ?',
                [alias],
                (err, row: any) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    resolve(row ? row.address : null);
                }
            );
        });
    }

    /**
     * Close the database connection
     */
    close(): void {
        this.db.close();
    }
}