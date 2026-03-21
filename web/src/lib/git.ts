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

export interface CommitDetail {
  hash: string;
  shortHash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
  parentHash: string | null;
  childHash: string | null;
  fileChanges: FileChange[];
  stats: {
    additions: number;
    deletions: number;
    filesChanged: number;
  };
}

export interface FileChange {
  path: string;
  status: 'added' | 'modified' | 'deleted' | 'renamed';
  oldPath?: string;
  additions: number;
  deletions: number;
  hunks: DiffHunk[];
}

export interface DiffHunk {
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: DiffLine[];
}

export interface DiffLine {
  type: 'context' | 'addition' | 'deletion';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
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

export function getCommitDetail(address: string, name: string, hash: string): CommitDetail | null {
  const repoPath = getRepoPath(address, name);
  
  try {
    // Normalize hash (handle short hashes)
    const fullHash = gitCommand(repoPath, `rev-parse ${hash}^{commit}`);
    
    // Get commit metadata - using format separators that won't conflict
    const metaOutput = gitCommand(
      repoPath, 
      `show -s --format='%H|||%an|||%ae|||%at|||%B' ${fullHash}`
    );
    
    const parts = metaOutput.split('|||');
    if (parts.length < 5) {
      throw new Error('Invalid commit format');
    }
    
    const [hashPart, author, email, timestamp, ...messageParts] = parts;
    const message = messageParts.join('|||').trim();
    
    // Get parent hash
    let parentHash: string | null = null;
    try {
      parentHash = gitCommand(repoPath, `rev-parse ${fullHash}^`);
    } catch {
      // Root commit has no parent
    }
    
    // Get next commit (child)
    const childHash = getChildCommit(repoPath, fullHash);
    
    // Get file changes with stats
    const fileChanges = getCommitFileChanges(repoPath, fullHash);
    
    // Calculate total stats
    const stats = calculateDiffStats(fileChanges);
    
    return {
      hash: fullHash,
      shortHash: fullHash.substring(0, 7),
      author,
      email,
      timestamp: parseInt(timestamp),
      message,
      parentHash,
      childHash,
      fileChanges,
      stats
    };
  } catch (error) {
    console.error('Error getting commit detail:', error);
    return null;
  }
}

function getChildCommit(repoPath: string, hash: string): string | null {
  try {
    // Find commits that have this hash as parent
    const output = gitCommand(repoPath, `rev-list --all --parents | grep " ${hash}"`);
    const lines = output.split('\n').filter(line => line.trim());
    
    if (lines.length > 0) {
      const firstChild = lines[0].split(' ')[0];
      return firstChild;
    }
    
    return null;
  } catch {
    return null;
  }
}

function getCommitFileChanges(repoPath: string, hash: string): FileChange[] {
  try {
    // Get list of changed files with status
    const filesOutput = gitCommand(repoPath, `diff-tree --name-status ${hash}^..${hash}`);
    if (!filesOutput) return [];
    
    const fileEntries = filesOutput.split('\n').filter(Boolean);
    
    return fileEntries.map(entry => {
      const parts = entry.split('\t');
      const status = parts[0];
      const path = parts[1];
      const oldPath = parts.length > 2 ? parts[1] : undefined;
      const actualPath = parts.length > 2 ? parts[2] : path;
      
      // Get diff for this specific file
      let diffOutput = '';
      try {
        diffOutput = gitCommand(
          repoPath, 
          `diff --unified=3 ${hash}^..${hash} -- "${actualPath}"`
        );
      } catch {
        // File might be binary or deleted
      }
      
      const hunks = parseDiffOutput(diffOutput);
      const fileStats = calculateFileStats(hunks);
      
      return {
        path: actualPath,
        status: mapGitStatus(status),
        oldPath: status.startsWith('R') ? oldPath : undefined,
        additions: fileStats.additions,
        deletions: fileStats.deletions,
        hunks
      };
    });
  } catch (error) {
    console.error('Error getting file changes:', error);
    return [];
  }
}

function parseDiffOutput(diffOutput: string): DiffHunk[] {
  if (!diffOutput) return [];
  
  const lines = diffOutput.split('\n');
  const hunks: DiffHunk[] = [];
  let currentHunk: Partial<DiffHunk> | null = null;
  
  for (const line of lines) {
    // Parse hunk header: @@ -oldStart,oldCount +newStart,newCount @@
    const hunkMatch = line.match(/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/);
    if (hunkMatch) {
      if (currentHunk?.lines) {
        hunks.push(currentHunk as DiffHunk);
      }
      
      currentHunk = {
        oldStart: parseInt(hunkMatch[1]),
        oldCount: parseInt(hunkMatch[2] || '1'),
        newStart: parseInt(hunkMatch[3]),
        newCount: parseInt(hunkMatch[4] || '1'),
        lines: []
      };
      continue;
    }
    
    // Parse diff lines
    if (currentHunk && (line.startsWith(' ') || line.startsWith('+') || line.startsWith('-'))) {
      const type = line[0] === '+' ? 'addition' : 
                   line[0] === '-' ? 'deletion' : 'context';
      
      const content = line.substring(1);
      const lineNumbers = calculateLineNumbers(currentHunk.lines!, currentHunk.oldStart!, currentHunk.newStart!, type);
      
      currentHunk.lines!.push({
        type,
        content,
        oldLineNumber: lineNumbers.oldLineNumber,
        newLineNumber: lineNumbers.newLineNumber
      });
    }
  }
  
  if (currentHunk?.lines) {
    hunks.push(currentHunk as DiffHunk);
  }
  
  return hunks;
}

function calculateLineNumbers(lines: DiffLine[], oldStart: number, newStart: number, type: string): { oldLineNumber?: number; newLineNumber?: number } {
  let oldLineNumber: number | undefined = undefined;
  let newLineNumber: number | undefined = undefined;
  
  const oldRelevantLines = lines.filter(l => l.type !== 'addition').length;
  const newRelevantLines = lines.filter(l => l.type !== 'deletion').length;
  
  if (type !== 'addition') {
    oldLineNumber = oldStart + oldRelevantLines;
  }
  
  if (type !== 'deletion') {
    newLineNumber = newStart + newRelevantLines;
  }
  
  return { oldLineNumber, newLineNumber };
}

function mapGitStatus(status: string): FileChange['status'] {
  const statusChar = status[0];
  switch (statusChar) {
    case 'A': return 'added';
    case 'M': return 'modified';
    case 'D': return 'deleted';
    case 'R': return 'renamed';
    default: return 'modified';
  }
}

function calculateFileStats(hunks: DiffHunk[]): { additions: number; deletions: number } {
  let additions = 0;
  let deletions = 0;
  
  for (const hunk of hunks) {
    for (const line of hunk.lines) {
      if (line.type === 'addition') additions++;
      if (line.type === 'deletion') deletions++;
    }
  }
  
  return { additions, deletions };
}

function calculateDiffStats(fileChanges: FileChange[]): { additions: number; deletions: number; filesChanged: number } {
  let additions = 0;
  let deletions = 0;
  
  for (const change of fileChanges) {
    additions += change.additions;
    deletions += change.deletions;
  }
  
  return {
    additions,
    deletions,
    filesChanged: fileChanges.length
  };
}