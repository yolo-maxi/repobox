import { execSync } from 'child_process';
import path from 'path';

// Default data directory (configurable via env var)
const DATA_DIR = process.env.REPOBOX_DATA_DIR || '/var/lib/repobox/repos';
const DB_PATH = path.join(DATA_DIR, 'repobox.db');

export function runQuery<T = any>(query: string, params: any[] = []): T[] {
  try {
    // Execute the query (table creation is handled by backend)
    let sqliteCommand = query;
    
    // Replace parameter placeholders with actual values (simple implementation)
    params.forEach((param, index) => {
      const placeholder = '?';
      const paramValue = typeof param === 'string' ? `'${param.replace(/'/g, "''")}'` : String(param);
      sqliteCommand = sqliteCommand.replace(placeholder, paramValue);
    });

    const output = execSync(`sqlite3 -json "${DB_PATH}" "${sqliteCommand}"`, { 
      encoding: 'utf8',
      timeout: 5000
    });
    
    if (!output.trim()) {
      return [];
    }
    
    return JSON.parse(output) as T[];
  } catch (error: any) {
    console.error('SQLite query error:', error.message);
    return [];
  }
}

export function runQueryOne<T = any>(query: string, params: any[] = []): T | undefined {
  const results = runQuery<T>(query, params);
  return results[0];
}

export function getRepoPath(address: string, name: string): string {
  return path.join(DATA_DIR, address, `${name}.git`);
}

export interface Repo {
  address: string;
  name: string;
  owner_address: string;
  created_at: string;
}

export interface PushLog {
  id: number;
  address: string;
  name: string;
  pusher_address?: string;
  commit_hash?: string;
  commit_message?: string;
  pushed_at: string;
}

export interface RepoPermission {
  repo_address: string;
  repo_name: string;
  identity: string;
  verb: string;
  target: string | null;
}

export function runExec(query: string, params: any[] = []): void {
  try {
    let sqliteCommand = query;
    params.forEach((param) => {
      const placeholder = '?';
      const paramValue = typeof param === 'string' ? `'${param.replace(/'/g, "''")}'` : param === null ? 'NULL' : String(param);
      sqliteCommand = sqliteCommand.replace(placeholder, paramValue);
    });

    execSync(`sqlite3 "${DB_PATH}" "${sqliteCommand}"`, {
      encoding: 'utf8',
      timeout: 5000
    });
  } catch (error: any) {
    console.error('SQLite exec error:', error.message);
  }
}