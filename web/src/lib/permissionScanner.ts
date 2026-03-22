import { execSync } from 'child_process';
import * as yaml from 'js-yaml';
import { runQuery, runExec, getRepoPath, type Repo } from './database';

// Verbs that 'own' expands to
const OWN_VERBS = ['push', 'merge', 'delete', 'force-push', 'edit'];

interface RepoBoxConfig {
  groups?: Record<string, string[]>;
  permissions?: {
    default?: string;
    rules?: string[];
  };
}

interface ParsedRule {
  subject: string;
  verb: string;
  target: string | null;
}

/**
 * Parse a single permission rule string.
 * Format: <subject> <verb> [>branch] [path]
 * Examples:
 *   '* read ./**'
 *   'founders edit ./**'
 *   'evm:0x1234... push >main ./src/**'
 *   'not * delete ./**'  (deny rule — skipped)
 */
function parseRule(rule: string): ParsedRule[] {
  const parts = rule.trim().split(/\s+/);
  if (parts.length < 2) return [];

  // Skip deny rules
  if (parts[0] === 'not') return [];

  const subject = parts[0];
  const verb = parts[1];

  // Collect the rest as target (branch + path combined)
  const targetParts = parts.slice(2);
  const target = targetParts.length > 0 ? targetParts.join(' ') : null;

  // Expand 'own' verb
  if (verb === 'own') {
    return OWN_VERBS.map(v => ({ subject, verb: v, target }));
  }

  return [{ subject, verb, target }];
}

/**
 * Expand a subject reference to a list of identities.
 * - '*' stays as '*'
 * - 'evm:0x...' stays as-is
 * - group name gets expanded to its members
 */
function expandSubject(subject: string, groups: Record<string, string[]>): string[] {
  if (subject === '*') return ['*'];
  if (subject.startsWith('evm:')) return [subject];

  // It's a group name
  const members = groups[subject];
  if (members) return members;

  // Unknown group, treat as identity
  return [subject];
}

/**
 * Read and parse .repobox/config.yml from a bare git repo.
 */
function readRepoConfig(repoPath: string): RepoBoxConfig | null {
  try {
    const output = execSync(
      `git --git-dir="${repoPath}" show HEAD:.repobox/config.yml 2>/dev/null`,
      { encoding: 'utf8', timeout: 5000 }
    );
    if (!output.trim()) return null;
    return yaml.load(output) as RepoBoxConfig;
  } catch {
    return null;
  }
}

/**
 * Ensure the repo_permissions table exists.
 */
function ensureTable(): void {
  runExec(
    `CREATE TABLE IF NOT EXISTS repo_permissions (
      repo_address TEXT NOT NULL,
      repo_name TEXT NOT NULL,
      identity TEXT NOT NULL,
      verb TEXT NOT NULL,
      target TEXT,
      PRIMARY KEY(repo_address, repo_name, identity, verb, target)
    )`.replace(/\n/g, ' ')
  );
}

/**
 * Scan all repos and populate the repo_permissions table.
 */
export function scanAllPermissions(): void {
  ensureTable();

  // Clear existing permissions
  runExec('DELETE FROM repo_permissions');

  // Get all repos from the database
  const repos = runQuery<Repo>('SELECT * FROM repos');

  for (const repo of repos) {
    try {
      const repoPath = getRepoPath(repo.address, repo.name);
      const config = readRepoConfig(repoPath);
      if (!config?.permissions?.rules) continue;

      const groups = config.groups || {};

      for (const ruleStr of config.permissions.rules) {
        const parsed = parseRule(ruleStr);
        for (const rule of parsed) {
          const identities = expandSubject(rule.subject, groups);
          for (const identity of identities) {
            // Use INSERT OR IGNORE to handle duplicate primary keys
            runExec(
              'INSERT OR IGNORE INTO repo_permissions (repo_address, repo_name, identity, verb, target) VALUES (?, ?, ?, ?, ?)',
              [repo.address, repo.name, identity, rule.verb, rule.target]
            );
          }
        }
      }
    } catch (error) {
      console.warn(`Error scanning permissions for ${repo.address}/${repo.name}:`, error);
    }
  }
}

/**
 * Check if the permissions table has been populated.
 */
function isTablePopulated(): boolean {
  const result = runQuery<{ count: number }>('SELECT COUNT(*) as count FROM repo_permissions');
  return (result[0]?.count ?? 0) > 0;
}

let lastScanTime = 0;
const SCAN_INTERVAL_MS = 5 * 60 * 1000; // 5 minutes

/**
 * Ensure permissions are scanned. Call this from API routes.
 * Only rescans if the table is empty or stale (>5 minutes).
 */
export function ensurePermissionsScanned(): void {
  ensureTable();

  const now = Date.now();
  if (now - lastScanTime < SCAN_INTERVAL_MS && isTablePopulated()) {
    return;
  }

  scanAllPermissions();
  lastScanTime = now;
}

/**
 * Force a fresh scan of all permissions.
 */
export function refreshPermissions(): void {
  lastScanTime = 0;
  scanAllPermissions();
  lastScanTime = Date.now();
}
