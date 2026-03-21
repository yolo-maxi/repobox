'use client';

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { notFound } from 'next/navigation';
import Link from 'next/link';
import { getFileIcon } from '@/lib/utils';
import { repoUrls, generateBreadcrumbs } from '@/lib/repoUrls';
import ExploreHeader from '@/components/explore/ExploreHeader';
import ExploreSidebar from '@/components/explore/ExploreSidebar';
import MarkdownRenderer from '@/components/markdown/MarkdownRenderer';

interface RepoDetails {
  address: string;
  name: string;
  owner_address: string;
  default_branch: string;
}

interface FileContent {
  path: string;
  content: string;
  size: number;
}

function getFileLanguage(filePath: string): string {
  const extension = filePath.split('.').pop()?.toLowerCase();
  const langMap: { [key: string]: string } = {
    'js': 'javascript',
    'jsx': 'jsx',
    'ts': 'typescript',
    'tsx': 'tsx',
    'rs': 'rust',
    'go': 'go',
    'py': 'python',
    'java': 'java',
    'c': 'c',
    'cpp': 'cpp',
    'cc': 'cpp',
    'cxx': 'cpp',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'php': 'php',
    'rb': 'ruby',
    'sh': 'bash',
    'bash': 'bash',
    'json': 'json',
    'yml': 'yaml',
    'yaml': 'yaml',
    'toml': 'toml',
    'xml': 'xml',
    'sql': 'sql',
    'md': 'markdown',
    'dockerfile': 'dockerfile',
    'makefile': 'makefile'
  };
  
  return langMap[extension || ''] || 'text';
}

function isMarkdown(filePath: string): boolean {
  const fileName = filePath.split('/').pop()?.toLowerCase() || '';
  return fileName.endsWith('.md') || fileName.endsWith('.markdown');
}

function isBinaryFile(filePath: string): boolean {
  const extension = filePath.split('.').pop()?.toLowerCase() || '';
  const binaryExtensions = [
    'png', 'jpg', 'jpeg', 'gif', 'webp', 'ico', 'svg',
    'mp3', 'mp4', 'avi', 'mov', 'wav',
    'zip', 'tar', 'gz', 'exe', 'dll', 'so', 'dylib',
    'pdf', 'doc', 'docx', 'xls', 'xlsx'
  ];
  return binaryExtensions.includes(extension);
}

export default function BlobPage() {
  const params = useParams();
  const [repo, setRepo] = useState<RepoDetails | null>(null);
  const [fileContent, setFileContent] = useState<FileContent | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const address = Array.isArray(params.address) ? params.address[0] : params.address;
  const name = Array.isArray(params.name) ? params.name[0] : params.name;
  const branch = Array.isArray(params.branch) ? params.branch[0] : params.branch;
  const pathSegments = params.path ? (Array.isArray(params.path) ? params.path : [params.path]) : [];
  const filePath = pathSegments.join('/');

  useEffect(() => {
    if (!address || !name || !branch || !filePath) return;

    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);

        // Get repository details
        const repoRes = await fetch(`/api/explorer/repos/${address}/${name}`);
        if (!repoRes.ok) {
          throw new Error('Repository not found');
        }
        const repoData = await repoRes.json();
        setRepo(repoData);

        // Get file content
        const fileRes = await fetch(`/api/explorer/repos/${address}/${name}/blob/${filePath}?branch=${branch}`);
        if (!fileRes.ok) {
          if (fileRes.status === 404) {
            throw new Error('File not found');
          }
          throw new Error('Failed to load file');
        }
        
        const fileData = await fileRes.json();
        setFileContent(fileData);
      } catch (err) {
        console.error('Blob page error:', err);
        setError(err instanceof Error ? err.message : 'Failed to load file');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [address, name, branch, filePath]);

  if (!address || !name || !branch || !filePath) {
    return notFound();
  }

  if (loading) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-loading">
              <div className="explore-loading-spinner"></div>
              <p>Loading file...</p>
            </div>
          </main>
        </div>
      </div>
    );
  }

  if (error || !repo || !fileContent) {
    return (
      <div className="explore-layout">
        <ExploreHeader />
        <div className="explore-container">
          <ExploreSidebar />
          <main className="explore-main">
            <div className="explore-empty">
              <h3>File not found</h3>
              <p>{error}</p>
              <Link href={repoUrls.home(address, name)} className="explore-back-link">
                ← Back to repository
              </Link>
            </div>
          </main>
        </div>
      </div>
    );
  }

  // Generate breadcrumb navigation
  const breadcrumbs = generateBreadcrumbs(address, name, branch, filePath, true);
  const fileName = filePath.split('/').pop() || '';
  const parentPath = pathSegments.slice(0, -1).join('/');

  return (
    <div className="explore-layout">
      <ExploreHeader />
      
      <div className="explore-breadcrumb-nav">
        <div className="explore-main-header-content">
          <Link href="/explore" className="explore-breadcrumb-link">Explore</Link>
          <span className="explore-breadcrumb-separator">/</span>
          <Link href={`/explore/${repo.owner_address}`} className="explore-breadcrumb-link">
            {repo.owner_address.slice(0, 6)}...{repo.owner_address.slice(-4)}
          </Link>
          <span className="explore-breadcrumb-separator">/</span>
          {breadcrumbs.map((crumb, index) => (
            <span key={index}>
              {index > 0 && <span className="explore-breadcrumb-separator">/</span>}
              <Link href={crumb.href} className="explore-breadcrumb-link">
                {crumb.label}
              </Link>
            </span>
          ))}
          <span className="explore-breadcrumb-separator">@</span>
          <span className="explore-breadcrumb-current">{branch}</span>
        </div>
      </div>

      <div className="explore-container">
        <ExploreSidebar />
        
        <main className="explore-main">
          <div className="explore-file-viewer">
            {/* File Header */}
            <div className="explore-file-header">
              <div className="explore-file-info">
                <span className="explore-file-icon">
                  {getFileIcon(fileName, false)}
                </span>
                <span className="explore-file-name">{fileName}</span>
                <span className="explore-file-size">
                  {fileContent.size ? `${Math.round(fileContent.size / 1024)}KB` : ''}
                </span>
              </div>
              <div className="explore-file-actions">
                {parentPath && (
                  <Link
                    href={repoUrls.tree(address, name, branch, parentPath)}
                    className="explore-file-action-btn"
                  >
                    ↑ Parent directory
                  </Link>
                )}
                <Link
                  href={repoUrls.home(address, name)}
                  className="explore-file-action-btn"
                >
                  Repository home
                </Link>
              </div>
            </div>

            {/* File Content */}
            <div className="explore-file-content-container">
              {isBinaryFile(filePath) ? (
                <div className="explore-file-binary">
                  <div className="explore-file-binary-icon">📄</div>
                  <h3>Binary file</h3>
                  <p>This file cannot be displayed as text.</p>
                  <p>File size: {Math.round(fileContent.size / 1024)}KB</p>
                </div>
              ) : isMarkdown(filePath) ? (
                <div className="explore-file-markdown">
                  <MarkdownRenderer 
                    content={fileContent.content}
                    baseUrl={`/api/explorer/repos/${address}/${name}/blob/`}
                    className="explore-readme-markdown"
                  />
                </div>
              ) : (
                <div className="explore-file-source">
                  <pre className="explore-file-content">
                    <code className={`language-${getFileLanguage(filePath)}`}>
                      {fileContent.content}
                    </code>
                  </pre>
                </div>
              )}
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}