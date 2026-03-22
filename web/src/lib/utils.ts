export function formatTimeAgo(dateString: string | null): string {
  if (!dateString) return 'Never';
  
  const now = new Date();
  const date = new Date(dateString);
  const diffMs = now.getTime() - date.getTime();
  
  const seconds = Math.floor(diffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  const months = Math.floor(days / 30);
  const years = Math.floor(days / 365);
  
  if (years > 0) return `${years} year${years > 1 ? 's' : ''} ago`;
  if (months > 0) return `${months} month${months > 1 ? 's' : ''} ago`;
  if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
  if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  return 'Just now';
}

const ALIAS_ADJ_1 = ['deep', 'wild', 'bright', 'silent', 'swift', 'lunar', 'solar', 'frost', 'ember', 'neon', 'misty', 'stone', 'velvet', 'cosmic', 'golden', 'azure'];
const ALIAS_ADJ_2 = ['blue', 'green', 'coral', 'silver', 'crimson', 'violet', 'amber', 'teal', 'indigo', 'scarlet', 'cobalt', 'pearl', 'obsidian', 'jade', 'sunset', 'aqua'];
const ALIAS_ANIMAL = ['kraken', 'otter', 'falcon', 'fox', 'wolf', 'lynx', 'orca', 'raven', 'viper', 'tiger', 'panda', 'eagle', 'whale', 'manta', 'gecko', 'badger'];

export function formatAddress(address: string): string {
  if (!address) return '';

  // Already human-readable name (ENS / alias / username)
  if (!/^0x[a-fA-F0-9]{40}$/.test(address)) return address;

  const hex = address.slice(2).toLowerCase();
  const a = parseInt(hex.slice(0, 2), 16) % ALIAS_ADJ_1.length;
  const b = parseInt(hex.slice(2, 4), 16) % ALIAS_ADJ_2.length;
  const c = parseInt(hex.slice(4, 6), 16) % ALIAS_ANIMAL.length;

  return `${ALIAS_ADJ_1[a]}-${ALIAS_ADJ_2[b]}-${ALIAS_ANIMAL[c]}`;
}

export function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function getFileIcon(fileName: string, isDirectory: boolean): string {
  if (isDirectory) return '📁';
  
  const ext = fileName.split('.').pop()?.toLowerCase();
  
  switch (ext) {
    case 'md':
    case 'txt':
      return '📄';
    case 'js':
    case 'ts':
    case 'jsx':
    case 'tsx':
      return '📜';
    case 'json':
      return '📋';
    case 'png':
    case 'jpg':
    case 'jpeg':
    case 'gif':
    case 'svg':
      return '🖼️';
    case 'css':
    case 'scss':
    case 'sass':
      return '🎨';
    case 'html':
    case 'htm':
      return '🌐';
    default:
      return '📄';
  }
}

export function copyToClipboard(text: string): Promise<void> {
  return navigator.clipboard.writeText(text);
}