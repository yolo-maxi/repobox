'use client';

import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

const langMap: Record<string, string> = {
  rs: 'rust', ts: 'typescript', tsx: 'tsx', js: 'javascript', jsx: 'jsx',
  json: 'json', yml: 'yaml', yaml: 'yaml', toml: 'toml',
  md: 'markdown', html: 'markup', css: 'css', scss: 'scss',
  sh: 'bash', bash: 'bash', zsh: 'bash',
  py: 'python', rb: 'ruby', go: 'go', java: 'java',
  c: 'c', cpp: 'cpp', cc: 'cpp', h: 'c', hpp: 'cpp',
  cs: 'csharp', swift: 'swift', kt: 'kotlin',
  sol: 'solidity', vy: 'python',
  sql: 'sql', graphql: 'graphql', gql: 'graphql',
  xml: 'xml', svg: 'markup',
  dockerfile: 'docker', makefile: 'makefile',
  lua: 'lua', r: 'r', dart: 'dart', zig: 'zig',
  php: 'php', pl: 'perl', ex: 'elixir', erl: 'erlang',
  hs: 'haskell', ml: 'ocaml', nim: 'nim',
  tf: 'hcl', hcl: 'hcl',
  proto: 'protobuf',
  env: 'bash', gitignore: 'bash',
  lock: 'json',
};

function getLanguage(filePath: string): string {
  const name = filePath.split('/').pop()?.toLowerCase() || '';
  // Exact filename matches
  if (name === 'dockerfile') return 'docker';
  if (name === 'makefile' || name === 'gnumakefile') return 'makefile';
  if (name === 'cmakelists.txt') return 'cmake';
  if (name === 'cargo.toml' || name === 'cargo.lock') return 'toml';
  if (name.startsWith('.env')) return 'bash';
  
  const ext = name.split('.').pop() || '';
  return langMap[ext] || 'text';
}

// Custom theme overrides to match --bp-* design system
const customStyle: Record<string, React.CSSProperties> = {
  ...vscDarkPlus,
  'pre[class*="language-"]': {
    ...(vscDarkPlus['pre[class*="language-"]'] as any),
    background: 'var(--bp-bg)',
    margin: 0,
    padding: '16px 20px',
    fontSize: '12px',
    lineHeight: '1.6',
    borderRadius: 0,
  },
  'code[class*="language-"]': {
    ...(vscDarkPlus['code[class*="language-"]'] as any),
    background: 'none',
    fontSize: '12px',
    fontFamily: 'var(--font-mono, "JetBrains Mono", monospace)',
  },
};

interface CodeViewerProps {
  code: string;
  filePath: string;
  showLineNumbers?: boolean;
}

export default function CodeViewer({ code, filePath, showLineNumbers = true }: CodeViewerProps) {
  const language = getLanguage(filePath);
  
  return (
    <SyntaxHighlighter
      language={language}
      style={customStyle}
      showLineNumbers={showLineNumbers}
      lineNumberStyle={{
        color: 'rgba(122, 154, 180, 0.3)',
        fontSize: '11px',
        minWidth: '3em',
        paddingRight: '12px',
        userSelect: 'none',
      }}
      wrapLongLines={false}
      customStyle={{
        background: 'var(--bp-bg)',
        margin: 0,
        overflow: 'auto',
      }}
    >
      {code}
    </SyntaxHighlighter>
  );
}
