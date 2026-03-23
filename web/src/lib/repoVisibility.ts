import { execSync } from 'child_process';
import * as yaml from 'js-yaml';
import { getRepoPath } from './database';

interface RepoBoxConfig {
  permissions?: {
    default?: string;
    rules?: string[];
  };
}

function readRepoConfig(address: string, name: string): RepoBoxConfig | null {
  try {
    const repoPath = getRepoPath(address, name);
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

function parseRule(rule: string): { deny: boolean; subject: string; verb: string } | null {
  const parts = rule.trim().split(/\s+/);
  if (parts.length < 2) return null;

  if (parts[0] === 'not') {
    if (parts.length < 3) return null;
    return { deny: true, subject: parts[1], verb: parts[2] };
  }

  return { deny: false, subject: parts[0], verb: parts[1] };
}

/**
 * Public visibility policy for explorer:
 * - no config => public
 * - parse failure => hidden (fail closed)
 * - explicit public read/own allow => public
 * - explicit public read/own deny => hidden
 * - no read rules => follow permissions.default
 */
export function isRepoPublicVisible(address: string, name: string): boolean {
  const config = readRepoConfig(address, name);

  if (!config) {
    // Legacy repos without config are currently treated as public.
    return true;
  }

  const defaultPolicy = (config.permissions?.default || 'allow').toLowerCase();
  const rules = config.permissions?.rules || [];

  const readOwnRules = rules
    .map(parseRule)
    .filter((r): r is { deny: boolean; subject: string; verb: string } => !!r)
    .filter((r) => r.subject === '*' && (r.verb === 'read' || r.verb === 'own'));

  if (readOwnRules.length > 0) {
    // Last matching rule wins (align with parser ordering semantics)
    const last = readOwnRules[readOwnRules.length - 1];
    return !last.deny;
  }

  // No explicit public read/own rule; if there are any read rules at all, it's non-public.
  const hasAnyReadOrOwnRule = rules
    .map(parseRule)
    .some((r) => r && (r.verb === 'read' || r.verb === 'own'));

  if (hasAnyReadOrOwnRule) {
    return false;
  }

  return defaultPolicy === 'allow';
}
