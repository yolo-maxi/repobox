import { execSync } from 'child_process';
import { getRepoPath } from './database';

export interface GitCommit {
  hash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
}

export interface GitFileEntry {
  type: 'blob' | 'tree';
  name: string;
  size?: number;
  path: string;
}

export function gitCommand(repoPath: string, command: string): string {
  try {
    return execSync(`git --git-dir="${repoPath}" ${command}`, { 
      encoding: 'utf8',
      timeout: 10000 
    }).trim();
  } catch (error: any) {
    if (error.status === 128) {
      // Empty repository or no commits
      return '';
    }
    throw error;
  }
}

export function getCommitCount(address: string, name: string): number {
  const repoPath = getRepoPath(address, name);
  try {
    const output = gitCommand(repoPath, 'rev-list --count HEAD');
    return parseInt(output) || 0;
  } catch {
    return 0;
  }
}

export function getLastCommitDate(address: string, name: string): string | null {
  const repoPath = getRepoPath(address, name);
  try {
    const output = gitCommand(repoPath, 'log -1 --format=%at');
    if (output) {
      return new Date(parseInt(output) * 1000).toISOString();
    }
  } catch {
    // Empty repo or no commits
  }
  return null;
}

export function getDefaultBranch(address: string, name: string): string {
  const repoPath = getRepoPath(address, name);
  try {
    // Try to get the default branch (HEAD points to it)
    const output = gitCommand(repoPath, 'symbolic-ref HEAD');
    return output.replace('refs/heads/', '') || 'main';
  } catch {
    return 'main';
  }
}

export function getCommitHistory(address: string, name: string, limit: number = 50): GitCommit[] {
  const repoPath = getRepoPath(address, name);
  try {
    const output = gitCommand(repoPath, `log --format='%H|%an|%ae|%at|%s' -n ${limit}`);
    if (!output) return [];
    
    return output.split('\n').map(line => {
      const [hash, author, email, timestamp, message] = line.split('|');
      return {
        hash,
        author,
        email,
        timestamp: parseInt(timestamp),
        message
      };
    });
  } catch {
    return [];
  }
}

export function getFileTree(address: string, name: string, path: string = ''): GitFileEntry[] {
  const repoPath = getRepoPath(address, name);
  try {
    const treePath = path ? `HEAD:${path}` : 'HEAD';
    const output = gitCommand(repoPath, `ls-tree ${treePath}`);
    if (!output) return [];
    
    return output.split('\n').map(line => {
      const match = line.match(/^(\d+)\s+(blob|tree)\s+([a-f0-9]+)\s+(.+)$/);
      if (!match) return null;
      
      const [, , type, , fileName] = match;
      const fullPath = path ? `${path}/${fileName}` : fileName;
      
      // For blobs, try to get file size
      let size: number | undefined;
      if (type === 'blob') {
        try {
          const sizeOutput = gitCommand(repoPath, `cat-file -s ${match[3]}`);
          size = parseInt(sizeOutput) || 0;
        } catch {
          // Size unavailable
        }
      }
      
      return {
        type: type as 'blob' | 'tree',
        name: fileName,
        size,
        path: fullPath
      };
    }).filter(Boolean) as GitFileEntry[];
  } catch {
    return [];
  }
}

export function getFileContent(address: string, name: string, filePath: string): string | null {
  const repoPath = getRepoPath(address, name);
  try {
    return gitCommand(repoPath, `show HEAD:${filePath}`);
  } catch {
    return null;
  }
}

export function getReadmeContent(address: string, name: string): string | null {
  const readmeFiles = ['README.md', 'readme.md', 'README', 'readme', 'README.txt'];
  
  for (const readmeFile of readmeFiles) {
    const content = getFileContent(address, name, readmeFile);
    if (content) {
      return content;
    }
  }
  
  return null;
}

export function getReadmeFirstLine(address: string, name: string): string | null {
  const content = getReadmeContent(address, name);
  if (!content) return null;
  
  // Get first non-empty line, remove markdown heading markers
  const lines = content.split('\n');
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed && !trimmed.startsWith('#')) {
      return trimmed;
    }
    if (trimmed.startsWith('#')) {
      return trimmed.replace(/^#+\s*/, '');
    }
  }
  
  return null;
}