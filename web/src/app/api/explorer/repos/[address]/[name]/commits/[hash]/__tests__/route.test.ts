import { NextRequest } from 'next/server';
import { GET } from '../route';

// Mock the dependencies
jest.mock('@/lib/database', () => ({
  runQueryOne: jest.fn()
}));

jest.mock('@/lib/git', () => ({
  getCommitDetail: jest.fn()
}));

import { runQueryOne } from '@/lib/database';
import { getCommitDetail } from '@/lib/git';

const mockRunQueryOne = runQueryOne as jest.MockedFunction<typeof runQueryOne>;
const mockGetCommitDetail = getCommitDetail as jest.MockedFunction<typeof getCommitDetail>;

describe('/api/explorer/repos/[address]/[name]/commits/[hash]', () => {
  const mockRequest = {
    url: 'http://localhost:3000/api/test'
  } as NextRequest;

  const mockParams = Promise.resolve({
    address: '0x1234567890123456789012345678901234567890',
    name: 'test-repo',
    hash: 'abc1234567890def'
  });

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should return 400 for missing parameters', async () => {
    const invalidParams = Promise.resolve({
      address: '',
      name: 'test-repo',
      hash: 'abc1234'
    });

    const response = await GET(mockRequest, { params: invalidParams });
    const data = await response.json();

    expect(response.status).toBe(400);
    expect(data.error).toBe('Address, name, and hash are required');
  });

  it('should return 400 for invalid hash format', async () => {
    const invalidHashParams = Promise.resolve({
      address: '0x1234567890123456789012345678901234567890',
      name: 'test-repo', 
      hash: 'invalid-hash!'
    });

    const response = await GET(mockRequest, { params: invalidHashParams });
    const data = await response.json();

    expect(response.status).toBe(400);
    expect(data.error).toBe('Invalid commit hash format');
  });

  it('should return 404 for non-existent repository', async () => {
    mockRunQueryOne.mockResolvedValue(null);

    const response = await GET(mockRequest, { params: mockParams });
    const data = await response.json();

    expect(response.status).toBe(404);
    expect(data.error).toBe('Repository not found');
    expect(mockRunQueryOne).toHaveBeenCalledWith(
      'SELECT * FROM repos WHERE address = ? AND name = ?',
      ['0x1234567890123456789012345678901234567890', 'test-repo']
    );
  });

  it('should return 404 for non-existent commit', async () => {
    mockRunQueryOne.mockResolvedValue({
      address: '0x1234567890123456789012345678901234567890',
      name: 'test-repo'
    });
    mockGetCommitDetail.mockReturnValue(null);

    const response = await GET(mockRequest, { params: mockParams });
    const data = await response.json();

    expect(response.status).toBe(404);
    expect(data.error).toBe('Commit not found');
    expect(mockGetCommitDetail).toHaveBeenCalledWith(
      '0x1234567890123456789012345678901234567890',
      'test-repo',
      'abc1234567890def'
    );
  });

  it('should return commit details for valid request', async () => {
    const mockRepo = {
      address: '0x1234567890123456789012345678901234567890',
      name: 'test-repo'
    };

    const mockCommit = {
      hash: 'abc1234567890def1234567890abcdef12345678',
      shortHash: 'abc1234',
      author: 'Test Author',
      email: 'test@example.com',
      timestamp: 1640995200,
      message: 'Test commit message',
      parentHash: 'def5678901234abc5678901234def56789012345',
      childHash: null,
      fileChanges: [],
      stats: { additions: 0, deletions: 0, filesChanged: 0 }
    };

    mockRunQueryOne.mockResolvedValue(mockRepo);
    mockGetCommitDetail.mockReturnValue(mockCommit);

    const response = await GET(mockRequest, { params: mockParams });
    const data = await response.json();

    expect(response.status).toBe(200);
    expect(data).toEqual(mockCommit);
  });

  it('should handle git errors gracefully', async () => {
    mockRunQueryOne.mockResolvedValue({
      address: '0x1234567890123456789012345678901234567890',
      name: 'test-repo'
    });
    mockGetCommitDetail.mockImplementation(() => {
      throw new Error('Git operation failed');
    });

    const response = await GET(mockRequest, { params: mockParams });
    const data = await response.json();

    expect(response.status).toBe(500);
    expect(data.error).toBe('Failed to fetch commit details');
  });

  it('should validate short hash format', async () => {
    const shortHashParams = Promise.resolve({
      address: '0x1234567890123456789012345678901234567890',
      name: 'test-repo',
      hash: 'abc1234'  // 7 characters should be valid
    });

    mockRunQueryOne.mockResolvedValue({
      address: '0x1234567890123456789012345678901234567890',
      name: 'test-repo'
    });
    
    mockGetCommitDetail.mockReturnValue({
      hash: 'abc1234567890def1234567890abcdef12345678',
      shortHash: 'abc1234',
      author: 'Test Author',
      email: 'test@example.com',
      timestamp: 1640995200,
      message: 'Test commit',
      parentHash: null,
      childHash: null,
      fileChanges: [],
      stats: { additions: 0, deletions: 0, filesChanged: 0 }
    });

    const response = await GET(mockRequest, { params: shortHashParams });

    expect(response.status).toBe(200);
    expect(mockGetCommitDetail).toHaveBeenCalledWith(
      '0x1234567890123456789012345678901234567890',
      'test-repo',
      'abc1234'
    );
  });
});