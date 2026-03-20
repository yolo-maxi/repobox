import { execSync } from 'child_process';
import path from 'path';

// Default data directory (configurable via env var)
const DATA_DIR = process.env.REPOBOX_DATA_DIR || '/var/lib/repobox/repos';
const DB_PATH = path.join(DATA_DIR, 'repobox.db');

export function runQuery<T = any>(query: string, params: any[] = []): T[] {
  try {
    // Create push_log table if it doesn't exist (run once)
    execSync(`sqlite3 "${DB_PATH}" "CREATE TABLE IF NOT EXISTS push_log (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      address TEXT NOT NULL,
      name TEXT NOT NULL,
      pusher_address TEXT,
      commit_hash TEXT,
      commit_message TEXT,
      pushed_at TEXT NOT NULL
    );"`, { encoding: 'utf8' });

    // Execute the query
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