const { execSync } = require('child_process');
const path = require('path');

// Test the repo stats logic directly
const address = '0x2C8c967964B0991554f696B9C99e96B1475d8017';
const name = 'demo-hackathon-1774098057';
const branch = 'HEAD';

const DATA_DIR = '/var/lib/repobox/repos';
const repoPath = path.join(DATA_DIR, address, `${name}.git`);

console.log('Testing repo stats for:', repoPath);

try {
  // Check if repo exists
  const result = execSync(`ls -la "${repoPath}"`, { encoding: 'utf8' });
  console.log('Repo exists:', result.split('\n')[0]);
  
  // Test git commands
  console.log('\n--- Testing git commands ---');
  
  // Get all tracked files
  const filesOutput = execSync(`git --git-dir="${repoPath}" ls-tree -r HEAD --name-only`, { encoding: 'utf8' });
  console.log('Files found:', filesOutput.split('\n').filter(Boolean));
  
  const allFiles = filesOutput.split('\n').filter(Boolean);
  console.log('Total files:', allFiles.length);
  
  // Test language detection
  const LANGUAGE_MAP = {
    'js': 'JavaScript', 'jsx': 'JavaScript', 'ts': 'TypeScript', 'tsx': 'TypeScript',
    'rs': 'Rust', 'go': 'Go', 'py': 'Python', 'java': 'Java',
    'c': 'C', 'cpp': 'C++', 'cc': 'C++', 'cxx': 'C++',
    'html': 'HTML', 'css': 'CSS', 'php': 'PHP', 'rb': 'Ruby',
    'sh': 'Shell', 'md': 'Markdown', 'json': 'JSON', 'yml': 'YAML',
    'xml': 'XML', 'sql': 'SQL'
  };
  
  function detectLanguageFromPath(filePath) {
    const fileName = filePath.split('/').pop() || '';
    if (fileName === 'Dockerfile') return 'Docker';
    if (fileName === 'Makefile') return 'Makefile';
    if (fileName.startsWith('.')) return 'Configuration';
    
    const lastDot = fileName.lastIndexOf('.');
    if (lastDot === -1) return 'Other';
    
    const extension = fileName.substring(lastDot + 1).toLowerCase();
    return LANGUAGE_MAP[extension] || 'Other';
  }
  
  // Analyze first few files
  let totalLines = 0;
  const languageStats = new Map();
  
  for (const filePath of allFiles.slice(0, 5)) {
    try {
      const fileContent = execSync(`git --git-dir="${repoPath}" show HEAD:${filePath}`, { encoding: 'utf8' });
      const lineCount = fileContent ? fileContent.split('\n').length : 0;
      const language = detectLanguageFromPath(filePath);
      
      console.log(`File: ${filePath}, Language: ${language}, Lines: ${lineCount}`);
      
      if (!languageStats.has(language)) {
        languageStats.set(language, { lines: 0, files: 0 });
      }
      
      const stats = languageStats.get(language);
      stats.lines += lineCount;
      stats.files += 1;
      totalLines += lineCount;
    } catch (error) {
      console.log(`Skipping file ${filePath}:`, error.message.split('\n')[0]);
    }
  }
  
  console.log('\n--- Language Statistics ---');
  console.log('Total lines analyzed:', totalLines);
  
  // Convert to API format
  const languageBreakdown = Array.from(languageStats.entries())
    .map(([name, stats]) => ({
      name,
      lines: stats.lines,
      files: stats.files,
      percentage: totalLines > 0 ? (stats.lines / totalLines) * 100 : 0
    }))
    .sort((a, b) => b.lines - a.lines);
  
  console.log('Language breakdown:', languageBreakdown);
  
} catch (error) {
  console.error('Error testing repo stats:', error.message);
}