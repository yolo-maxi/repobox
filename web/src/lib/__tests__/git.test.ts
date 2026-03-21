import { 
  gitCommand, 
  getCommitDetail, 
  parseDiffOutput,
  mapGitStatus,
  calculateFileStats,
  calculateDiffStats 
} from '../git';

// Mock execSync since we can't run actual git commands in tests
jest.mock('child_process', () => ({
  execSync: jest.fn()
}));

import { execSync } from 'child_process';
const mockExecSync = execSync as jest.MockedFunction<typeof execSync>;

describe('Git Utilities', () => {
  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('gitCommand', () => {
    it('should execute git command with proper timeout', () => {
      mockExecSync.mockReturnValue(Buffer.from('test output'));
      
      const result = gitCommand('/test/repo', 'log --oneline');
      
      expect(mockExecSync).toHaveBeenCalledWith(
        'git --git-dir="/test/repo" log --oneline',
        {
          encoding: 'utf8',
          timeout: 15000,
          maxBuffer: 10 * 1024 * 1024
        }
      );
      expect(result).toBe('test output');
    });

    it('should handle empty repository error gracefully', () => {
      const error = new Error('Command failed');
      (error as any).status = 128;
      mockExecSync.mockImplementation(() => {
        throw error;
      });
      
      const result = gitCommand('/test/repo', 'log --oneline');
      
      expect(result).toBe('');
    });

    it('should throw timeout error with descriptive message', () => {
      const error = new Error('Timeout');
      (error as any).code = 'ETIMEDOUT';
      mockExecSync.mockImplementation(() => {
        throw error;
      });
      
      expect(() => gitCommand('/test/repo', 'log --oneline'))
        .toThrow('Git operation timed out - commit may be too large');
    });
  });

  describe('parseDiffOutput', () => {
    it('should parse simple diff correctly', () => {
      const diffOutput = `diff --git a/test.js b/test.js
index 1234567..abcdefg 100644
--- a/test.js
+++ b/test.js
@@ -1,3 +1,3 @@
 line 1
-old line 2
+new line 2
 line 3`;
      
      const hunks = parseDiffOutput(diffOutput);
      
      expect(hunks).toHaveLength(1);
      expect(hunks[0].oldStart).toBe(1);
      expect(hunks[0].newStart).toBe(1);
      expect(hunks[0].lines).toHaveLength(4);
      
      const lines = hunks[0].lines;
      expect(lines[0].type).toBe('context');
      expect(lines[1].type).toBe('deletion');
      expect(lines[2].type).toBe('addition');
      expect(lines[3].type).toBe('context');
    });

    it('should handle binary files', () => {
      const diffOutput = `diff --git a/image.png b/image.png
Binary files a/image.png and b/image.png differ`;
      
      const hunks = parseDiffOutput(diffOutput);
      
      expect(hunks).toHaveLength(0);
    });

    it('should handle empty diff output', () => {
      const hunks = parseDiffOutput('');
      expect(hunks).toHaveLength(0);
    });
  });

  describe('mapGitStatus', () => {
    it('should map git status codes correctly', () => {
      expect(mapGitStatus('A')).toBe('added');
      expect(mapGitStatus('M')).toBe('modified');
      expect(mapGitStatus('D')).toBe('deleted');
      expect(mapGitStatus('R100')).toBe('renamed');
      expect(mapGitStatus('C')).toBe('modified');
    });
  });

  describe('calculateFileStats', () => {
    it('should count additions and deletions correctly', () => {
      const hunks = [
        {
          oldStart: 1,
          oldCount: 3,
          newStart: 1,
          newCount: 3,
          lines: [
            { type: 'context' as const, content: 'line 1' },
            { type: 'deletion' as const, content: 'old line' },
            { type: 'addition' as const, content: 'new line' },
            { type: 'addition' as const, content: 'another new line' },
            { type: 'context' as const, content: 'line 3' }
          ]
        }
      ];
      
      const stats = calculateFileStats(hunks);
      
      expect(stats.additions).toBe(2);
      expect(stats.deletions).toBe(1);
    });
  });

  describe('calculateDiffStats', () => {
    it('should aggregate file stats correctly', () => {
      const fileChanges = [
        { 
          path: 'file1.js',
          status: 'modified' as const,
          additions: 5,
          deletions: 2,
          hunks: []
        },
        {
          path: 'file2.js', 
          status: 'added' as const,
          additions: 10,
          deletions: 0,
          hunks: []
        }
      ];
      
      const stats = calculateDiffStats(fileChanges);
      
      expect(stats.additions).toBe(15);
      expect(stats.deletions).toBe(2);
      expect(stats.filesChanged).toBe(2);
    });
  });

  describe('getCommitDetail', () => {
    it('should return null for invalid hash format', () => {
      const result = getCommitDetail('0x123', 'repo', 'invalid-hash!');
      expect(result).toBeNull();
    });

    it('should handle git command failures gracefully', () => {
      mockExecSync.mockImplementation(() => {
        throw new Error('Git command failed');
      });
      
      const result = getCommitDetail('0x123', 'repo', 'abc1234');
      expect(result).toBeNull();
    });
  });
});

// Helper function exports for testing
export { parseDiffOutput, mapGitStatus, calculateFileStats, calculateDiffStats };