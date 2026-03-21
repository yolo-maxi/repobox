import { execSync } from 'child_process';
import { getRepoPath, getRepo } from './database';
import { secp256k1 } from '@noble/curves/secp256k1.js';
import { keccak256 } from 'js-sha3';

export interface GitCommit {
  hash: string;
  author: string;
  email: string;
  timestamp: number;
  message: string;
  signerAddress?: string;
  signatureValid?: boolean;
  ownerAddress?: string;
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

/**
 * Extract signer address from a git commit signature
 */
export function extractSignerAddress(address: string, name: string, commitHash: string): { signerAddress?: string, signatureValid: boolean } {
  try {
    const repoPath = getRepoPath(address, name);
    
    // Get the raw commit object to extract the signature
    const commitData = gitCommand(repoPath, `cat-file commit ${commitHash}`);
    
    // Look for the REPOBOX SIGNATURE block
    const sigMatch = commitData.match(/-----BEGIN REPOBOX SIGNATURE-----\s*\n([a-fA-F0-9]+)\s*\n-----END REPOBOX SIGNATURE-----/);
    if (!sigMatch) {
      return { signatureValid: false };
    }
    
    const signatureHex = sigMatch[1].trim();
    
    // Convert hex to bytes - should be 65 bytes (130 hex chars)
    if (signatureHex.length !== 130) {
      return { signatureValid: false };
    }
    
    const signatureBytes = Buffer.from(signatureHex, 'hex');
    
    // Get the commit content without the signature for verification
    // Remove the gpgsig block to get the original signed content
    const sigBlockStart = commitData.indexOf('gpgsig -----BEGIN REPOBOX SIGNATURE-----');
    const sigBlockEnd = commitData.indexOf('-----END REPOBOX SIGNATURE-----') + '-----END REPOBOX SIGNATURE-----'.length;
    
    if (sigBlockStart === -1 || sigBlockEnd === -1) {
      return { signatureValid: false };
    }
    
    // Reconstruct the original commit data without the signature
    const beforeSig = commitData.substring(0, sigBlockStart).trimEnd();
    const afterSig = commitData.substring(sigBlockEnd + 1); // +1 to skip the newline
    const originalCommitData = beforeSig + '\n\n' + afterSig.trim();
    
    // Recover the address from the signature
    const recoveredAddress = recoverAddressFromSignature(originalCommitData, signatureBytes);
    
    if (recoveredAddress) {
      return {
        signerAddress: recoveredAddress,
        signatureValid: true
      };
    }
    
    return { signatureValid: false };
  } catch (error) {
    console.warn(`Failed to extract signer for commit ${commitHash}:`, error);
    return { signatureValid: false };
  }
}

/**
 * Recover the EVM address from signature using ECDSA recovery
 */
function recoverAddressFromSignature(data: string, signatureBytes: Buffer): string | null {
  try {
    if (signatureBytes.length !== 65) {
      return null;
    }
    
    // Split signature into r, s, and recovery id
    const r = signatureBytes.slice(0, 32);
    const s = signatureBytes.slice(32, 64);
    const recoveryId = signatureBytes[64];
    
    // Hash the data with Keccak256
    const dataBytes = Buffer.from(data, 'utf8');
    const hash = Buffer.from(keccak256.arrayBuffer(dataBytes));
    
    // Create signature with recovery bit
    const signature = new secp256k1.Signature(
      BigInt('0x' + r.toString('hex')),
      BigInt('0x' + s.toString('hex'))
    ).addRecoveryBit(recoveryId);
    
    // Recover the public key
    const publicKey = signature.recoverPublicKey(hash);
    const publicKeyBytes = publicKey.toBytes(false); // uncompressed format
    
    // Derive EVM address from public key
    // Remove the 0x04 prefix and hash the remaining 64 bytes
    const pubKeyForHashing = publicKeyBytes.slice(1);
    const addressHash = Buffer.from(keccak256.arrayBuffer(pubKeyForHashing));
    
    // Take the last 20 bytes and convert to checksummed address
    const addressBytes = addressHash.slice(-20);
    const address = '0x' + addressBytes.toString('hex');
    
    // Return checksummed address
    return toChecksumAddress(address);
  } catch (error) {
    console.warn('Failed to recover address:', error);
    return null;
  }
}

/**
 * Convert address to EIP-55 checksum format
 */
function toChecksumAddress(address: string): string {
  const addr = address.toLowerCase().replace('0x', '');
  const hash = keccak256(addr);
  let checksumAddr = '0x';
  
  for (let i = 0; i < addr.length; i++) {
    if (parseInt(hash[i], 16) >= 8) {
      checksumAddr += addr[i].toUpperCase();
    } else {
      checksumAddr += addr[i];
    }
  }
  
  return checksumAddr;
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
    
    // Get repository owner address
    const repo = getRepo(address, name);
    const ownerAddress = repo?.owner_address || address;
    
    return output.split('\n').map(line => {
      const [hash, author, email, timestamp, message] = line.split('|');
      
      // Extract signer information
      const signerInfo = extractSignerAddress(address, name, hash);
      
      return {
        hash,
        author,
        email,
        timestamp: parseInt(timestamp),
        message,
        ownerAddress,
        ...signerInfo
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