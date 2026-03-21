import { NextRequest, NextResponse } from 'next/server';
import { runQuery } from '@/lib/database';
import { getRepoPath } from '@/lib/database';
import { gitCommand, sanitizeBranchName, branchExists } from '@/lib/git';

interface RouteContext {
  params: Promise<{ address: string; name: string }>;
}

interface LanguageStats {
  name: string;
  lines: number;
  files: number;
  percentage: number;
  color: string;
  extensions: string[];
}

interface RepoStats {
  language_breakdown: LanguageStats[];
  total_lines: number;
  total_files: number;
  unique_signers: number;
  repository_age_days: number;
  last_computed: string;
  computation_time_ms: number;
  branch: string;
}

// Language detection map
const LANGUAGE_MAP: { [key: string]: string } = {
  'js': 'JavaScript',
  'jsx': 'JavaScript', 
  'ts': 'TypeScript',
  'tsx': 'TypeScript',
  'rs': 'Rust',
  'go': 'Go',
  'py': 'Python',
  'java': 'Java',
  'c': 'C',
  'cpp': 'C++',
  'cc': 'C++',
  'cxx': 'C++',
  'html': 'HTML',
  'css': 'CSS',
  'scss': 'CSS',
  'sass': 'CSS',
  'less': 'CSS',
  'php': 'PHP',
  'rb': 'Ruby',
  'sh': 'Shell',
  'bash': 'Shell',
  'zsh': 'Shell',
  'fish': 'Shell',
  'md': 'Markdown',
  'json': 'JSON',
  'yml': 'YAML',
  'yaml': 'YAML',
  'toml': 'TOML',
  'xml': 'XML',
  'sql': 'SQL',
  'lua': 'Lua',
  'r': 'R',
  'swift': 'Swift',
  'kt': 'Kotlin',
  'scala': 'Scala',
  'clj': 'Clojure',
  'hs': 'Haskell',
  'elm': 'Elm',
  'dart': 'Dart',
  'vue': 'Vue'
};

// GitHub language colors
const LANGUAGE_COLORS: { [key: string]: string } = {
  'TypeScript': '#3178c6',
  'JavaScript': '#f1e05a', 
  'Rust': '#dea584',
  'Go': '#00ADD8',
  'Python': '#3572A5',
  'Java': '#b07219',
  'C++': '#f34b7d',
  'C': '#555555',
  'HTML': '#e34c26',
  'CSS': '#563d7c',
  'PHP': '#4F5D95',
  'Ruby': '#701516',
  'Shell': '#89e051',
  'Markdown': '#083fa1',
  'JSON': '#292929',
  'YAML': '#cb171e',
  'TOML': '#9c4221',
  'XML': '#0060ac',
  'SQL': '#e34c26',
  'Lua': '#000080',
  'R': '#198CE7',
  'Swift': '#ffac45',
  'Kotlin': '#A97BFF',
  'Scala': '#c22d40',
  'Clojure': '#db5855',
  'Haskell': '#5e5086',
  'Elm': '#60B5CC',
  'Dart': '#00B4AB',
  'Vue': '#4FC08D'
};

// Binary file extensions to exclude
const BINARY_EXTENSIONS = new Set([
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'ico', 'svg',
  'mp3', 'mp4', 'avi', 'mov', 'wav', 'flv', 'wmv',
  'zip', 'tar', 'gz', '7z', 'rar', 'bz2',
  'exe', 'dll', 'so', 'dylib', 'app',
  'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx',
  'woff', 'woff2', 'ttf', 'eot',
  'bin', 'dat', 'db', 'sqlite', 'sqlite3'
]);

function shouldIncludeFile(filePath: string): boolean {
  const fileName = filePath.split('/').pop() || '';
  const extension = fileName.includes('.') ? fileName.split('.').pop()?.toLowerCase() : '';
  
  // Exclude binary files
  if (extension && BINARY_EXTENSIONS.has(extension)) {
    return false;
  }
  
  // Exclude certain directories/paths
  if (filePath.includes('node_modules/') || 
      filePath.includes('/.git/') ||
      filePath.includes('/target/') ||
      filePath.includes('/dist/') ||
      filePath.includes('/build/')) {
    return false;
  }
  
  return true;
}

function detectLanguageFromPath(filePath: string): string | null {
  if (!shouldIncludeFile(filePath)) {
    return null; // File should be excluded entirely
  }
  
  const fileName = filePath.split('/').pop() || '';
  
  // Special files
  if (fileName === 'Dockerfile') return 'Docker';
  if (fileName === 'Makefile') return 'Makefile';
  if (fileName.startsWith('.')) return 'Configuration';
  
  // Extract extension
  const lastDot = fileName.lastIndexOf('.');
  if (lastDot === -1) return 'Other';
  
  const extension = fileName.substring(lastDot + 1).toLowerCase();
  return LANGUAGE_MAP[extension] || 'Other';
}

function getLanguageColor(language: string): string {
  return LANGUAGE_COLORS[language] || '#cccccc';
}

async function analyzeRepository(address: string, name: string, branch: string): Promise<RepoStats> {
  const startTime = Date.now();
  const repoPath = getRepoPath(address, name);
  
  // Sanitize branch if not HEAD
  if (branch !== 'HEAD') {
    branch = sanitizeBranchName(branch);
    if (!branchExists(address, name, branch)) {
      throw new Error(`Branch '${branch}' does not exist`);
    }
  }
  
  const ref = branch === 'HEAD' ? 'HEAD' : `refs/heads/${branch}`;
  
  try {
    // Get all tracked files
    const filesOutput = gitCommand(repoPath, `ls-tree -r ${ref} --name-only`);
    if (!filesOutput) {
      throw new Error('No files found in repository');
    }
    
    const allFiles = filesOutput.split('\n').filter(Boolean);
    const totalFiles = allFiles.length;
    
    // Language analysis
    const languageStats = new Map<string, { lines: number; files: number; extensions: Set<string> }>();
    let totalLines = 0;
    
    // Process files in batches to avoid command length limits
    const batchSize = 50;
    for (let i = 0; i < allFiles.length; i += batchSize) {
      const batch = allFiles.slice(i, i + batchSize);
      
      for (const filePath of batch) {
        try {
          // Detect language and skip if excluded
          const language = detectLanguageFromPath(filePath);
          if (language === null) continue; // File excluded
          
          // Count lines in this file
          const fileContent = gitCommand(repoPath, `show ${ref}:${filePath}`);
          if (!fileContent) continue; // Skip empty or binary files
          
          const lineCount = fileContent.split('\n').length;
          if (lineCount === 0) continue;
          
          const extension = filePath.includes('.') ? '.' + filePath.split('.').pop() : '';
          
          if (!languageStats.has(language)) {
            languageStats.set(language, { lines: 0, files: 0, extensions: new Set() });
          }
          
          const stats = languageStats.get(language)!;
          stats.lines += lineCount;
          stats.files += 1;
          if (extension) {
            stats.extensions.add(extension);
          }
          
          totalLines += lineCount;
        } catch (error) {
          // Skip files that can't be read (likely binary or corrupted)
          console.warn(`Skipping file ${filePath}:`, error);
        }
      }
    }
    
    // Convert to API format and calculate percentages
    let languageBreakdown: LanguageStats[] = Array.from(languageStats.entries())
      .map(([name, stats]) => ({
        name,
        lines: stats.lines,
        files: stats.files,
        percentage: totalLines > 0 ? (stats.lines / totalLines) * 100 : 0,
        color: getLanguageColor(name),
        extensions: Array.from(stats.extensions)
      }))
      .sort((a, b) => b.lines - a.lines); // Sort by lines descending
    
    // Aggregate all "Other" categories into a single entry
    const otherEntries = languageBreakdown.filter(lang => 
      lang.name === 'Other' || lang.name === 'Configuration'
    );
    
    if (otherEntries.length > 1) {
      const aggregatedOther: LanguageStats = {
        name: 'Other',
        lines: otherEntries.reduce((sum, entry) => sum + entry.lines, 0),
        files: otherEntries.reduce((sum, entry) => sum + entry.files, 0),
        percentage: 0, // Will be recalculated
        color: '#cccccc',
        extensions: [...new Set(otherEntries.flatMap(entry => entry.extensions))]
      };
      
      // Recalculate percentage
      aggregatedOther.percentage = totalLines > 0 ? (aggregatedOther.lines / totalLines) * 100 : 0;
      
      // Remove individual Other entries and add the aggregated one
      languageBreakdown = languageBreakdown
        .filter(lang => lang.name !== 'Other' && lang.name !== 'Configuration')
        .concat(aggregatedOther)
        .sort((a, b) => b.lines - a.lines);
    }
    
    // Count unique signers from push_log table
    const signers = runQuery<{ pusher_address: string }>(
      'SELECT DISTINCT pusher_address FROM push_log WHERE address = ? AND name = ? AND pusher_address IS NOT NULL',
      [address, name]
    );
    const uniqueSigners = signers.length;
    
    // Calculate repository age
    let repositoryAgeDays = 0;
    try {
      const firstCommitTimestamp = gitCommand(repoPath, `log --reverse --format=%at -1 ${ref}`);
      if (firstCommitTimestamp) {
        const firstCommitDate = new Date(parseInt(firstCommitTimestamp) * 1000);
        const now = new Date();
        repositoryAgeDays = Math.floor((now.getTime() - firstCommitDate.getTime()) / (24 * 60 * 60 * 1000));
      }
    } catch (error) {
      console.warn('Could not calculate repository age:', error);
    }
    
    const computationTime = Date.now() - startTime;
    
    return {
      language_breakdown: languageBreakdown,
      total_lines: totalLines,
      total_files: totalFiles,
      unique_signers: uniqueSigners,
      repository_age_days: repositoryAgeDays,
      last_computed: new Date().toISOString(),
      computation_time_ms: computationTime,
      branch: branch === 'HEAD' ? 'HEAD' : branch
    };
  } catch (error) {
    throw new Error(`Repository analysis failed: ${error}`);
  }
}

export async function GET(
  request: NextRequest,
  context: RouteContext
) {
  try {
    const { address, name } = await context.params;
    const { searchParams } = new URL(request.url);
    const branch = searchParams.get('branch') || 'HEAD';
    
    if (!address || !name) {
      return NextResponse.json(
        { error: 'Address and name are required' },
        { status: 400 }
      );
    }
    
    // Check if repo exists in database
    const repo = runQuery('SELECT * FROM repos WHERE address = ? AND name = ?', [address, name])[0];
    if (!repo) {
      return NextResponse.json(
        { error: 'Repository not found' },
        { status: 404 }
      );
    }
    
    // Analyze repository
    const stats = await analyzeRepository(address, name, branch);
    
    return NextResponse.json(stats);
  } catch (error: any) {
    console.error('Stats analysis error:', error);
    return NextResponse.json(
      { error: error.message || 'Failed to analyze repository' },
      { status: 500 }
    );
  }
}