import {
  Folder, File, FileCode, FileJson, FileText, FileCog,
  FileTerminal, Lock, Image, Globe, Palette, FileType,
  Database, Shield, GitBranch, Package, Hash, Braces,
} from 'lucide-react';

const SIZE = 16;
const STROKE = 1.5;

const extMap: Record<string, { icon: typeof File; color: string }> = {
  // Rust
  rs: { icon: FileCode, color: '#dea584' },
  // TypeScript / JavaScript
  ts: { icon: FileCode, color: '#3178c6' },
  tsx: { icon: FileCode, color: '#3178c6' },
  js: { icon: FileCode, color: '#f7df1e' },
  jsx: { icon: FileCode, color: '#f7df1e' },
  // Config
  yml: { icon: FileCog, color: '#8fb0c8' },
  yaml: { icon: FileCog, color: '#8fb0c8' },
  toml: { icon: FileCog, color: '#8fb0c8' },
  // Data
  json: { icon: FileJson, color: '#8fb0c8' },
  // Text / docs
  md: { icon: FileText, color: '#8fb0c8' },
  txt: { icon: FileText, color: '#8fb0c8' },
  // Shell
  sh: { icon: FileTerminal, color: '#89e051' },
  bash: { icon: FileTerminal, color: '#89e051' },
  // Lockfiles
  lock: { icon: Lock, color: '#636c76' },
  // Styles
  css: { icon: Palette, color: '#563d7c' },
  scss: { icon: Palette, color: '#c6538c' },
  // Web
  html: { icon: Globe, color: '#e34c26' },
  svg: { icon: Image, color: '#8fb0c8' },
  // Images
  png: { icon: Image, color: '#8fb0c8' },
  jpg: { icon: Image, color: '#8fb0c8' },
  jpeg: { icon: Image, color: '#8fb0c8' },
  gif: { icon: Image, color: '#8fb0c8' },
  webp: { icon: Image, color: '#8fb0c8' },
  // Solidity
  sol: { icon: Braces, color: '#627eea' },
  // SQL
  sql: { icon: Database, color: '#8fb0c8' },
  // Git
  gitignore: { icon: GitBranch, color: '#636c76' },
  // Env
  env: { icon: Shield, color: '#636c76' },
  // Fonts
  woff: { icon: FileType, color: '#636c76' },
  woff2: { icon: FileType, color: '#636c76' },
};

// Special filenames
const nameMap: Record<string, { icon: typeof File; color: string }> = {
  'dockerfile': { icon: Package, color: '#384d54' },
  'makefile': { icon: FileTerminal, color: '#89e051' },
  'cargo.toml': { icon: Package, color: '#dea584' },
  'cargo.lock': { icon: Lock, color: '#dea584' },
  'package.json': { icon: Package, color: '#f7df1e' },
  'tsconfig.json': { icon: FileCog, color: '#3178c6' },
  'license': { icon: FileText, color: '#8fb0c8' },
  'readme.md': { icon: FileText, color: '#8fb0c8' },
};

export function FileIcon({ name, isDir }: { name: string; isDir: boolean }) {
  if (isDir) {
    return <Folder size={SIZE} strokeWidth={STROKE} style={{ color: 'var(--bp-accent)' }} />;
  }

  const lower = name.toLowerCase();

  // Check special filenames first
  const special = nameMap[lower];
  if (special) {
    const Icon = special.icon;
    return <Icon size={SIZE} strokeWidth={STROKE} style={{ color: special.color }} />;
  }

  // Check extension
  const ext = lower.split('.').pop() || '';
  // Handle dotfiles like .env, .gitignore
  const dotfile = lower.startsWith('.') ? lower.slice(1) : '';

  const match = extMap[ext] || (dotfile ? extMap[dotfile] : null);
  if (match) {
    const Icon = match.icon;
    return <Icon size={SIZE} strokeWidth={STROKE} style={{ color: match.color }} />;
  }

  return <File size={SIZE} strokeWidth={STROKE} style={{ color: '#636c76' }} />;
}

// For the explore listing page
export function RepoIcon() {
  return <Package size={16} strokeWidth={STROKE} style={{ color: 'var(--bp-accent)', opacity: 0.7 }} />;
}
