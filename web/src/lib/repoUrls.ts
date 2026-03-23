/**
 * Utility functions for generating repo URLs in GitHub-style format
 */

export const repoUrls = {
  home: (addr: string, name: string) => 
    `/${addr}/${name}`,
  
  tree: (addr: string, name: string, branch: string, path: string = '') => {
    const cleanPath = path.startsWith('/') ? path.slice(1) : path;
    return `/${addr}/${name}/tree/${branch}${cleanPath ? `/${cleanPath}` : ''}`;
  },
  
  blob: (addr: string, name: string, branch: string, path: string) => {
    const cleanPath = path.startsWith('/') ? path.slice(1) : path;
    return `/${addr}/${name}/blob/${branch}/${cleanPath}`;
  },
  
  commits: (addr: string, name: string, branch: string) =>
    `/${addr}/${name}/commits/${branch}`,
  
  commit: (addr: string, name: string, hash: string) =>
    `/${addr}/${name}/commit/${hash}`,
};

/**
 * Generate breadcrumb navigation for file/directory paths
 */
export function generateBreadcrumbs(
  addr: string, 
  name: string, 
  branch: string, 
  path: string,
  isFile: boolean = false
) {
  const breadcrumbs = [
    { label: name, href: repoUrls.home(addr, name) }
  ];

  if (!path) return breadcrumbs;

  const pathParts = path.split('/').filter(Boolean);
  
  for (let i = 0; i < pathParts.length; i++) {
    const currentPath = pathParts.slice(0, i + 1).join('/');
    const isLastPart = i === pathParts.length - 1;
    
    breadcrumbs.push({
      label: pathParts[i],
      href: isLastPart && isFile 
        ? repoUrls.blob(addr, name, branch, currentPath)
        : repoUrls.tree(addr, name, branch, currentPath)
    });
  }

  return breadcrumbs;
}