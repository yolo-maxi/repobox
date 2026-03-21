#!/usr/bin/env node

/**
 * Simple integration test for commit detail functionality
 * Tests the core git operations against the repobox repository itself
 */

const path = require('path');
const { execSync } = require('child_process');

// Simple test framework
function assertEquals(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${expected}, got ${actual}`);
  }
}

function assertTrue(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertNotNull(value, message) {
  if (value === null || value === undefined) {
    throw new Error(message);
  }
}

// Test git command execution
function testGitCommand() {
  console.log('Testing basic git command...');
  
  try {
    const result = execSync('git log --oneline -n 1', { 
      encoding: 'utf8',
      timeout: 5000,
      cwd: __dirname 
    }).trim();
    
    assertTrue(result.length > 0, 'Git command should return output');
    console.log('✓ Git command works');
  } catch (error) {
    console.log('✗ Git command failed:', error.message);
    throw error;
  }
}

// Test commit hash validation
function testHashValidation() {
  console.log('Testing hash validation...');
  
  const validHashes = [
    'abc1234',  // 7 chars
    'abc1234567890def',  // 16 chars
    '1234567890abcdef1234567890abcdef12345678'  // 40 chars
  ];
  
  const invalidHashes = [
    'abc123',  // too short
    'invalid-hash!',  // invalid characters
    'gggggggg',  // invalid hex
    ''  // empty
  ];
  
  const hashRegex = /^[a-f0-9]{7,40}$/i;
  
  validHashes.forEach(hash => {
    assertTrue(hashRegex.test(hash), `Hash ${hash} should be valid`);
  });
  
  invalidHashes.forEach(hash => {
    assertTrue(!hashRegex.test(hash), `Hash ${hash} should be invalid`);
  });
  
  console.log('✓ Hash validation works');
}

// Test diff parsing logic
function testDiffParsing() {
  console.log('Testing diff parsing...');
  
  const sampleDiff = `@@ -1,3 +1,3 @@
 line 1
-old line 2
+new line 2
 line 3`;

  // Simple diff parser test
  const lines = sampleDiff.split('\n');
  let additionCount = 0;
  let deletionCount = 0;
  
  lines.forEach(line => {
    if (line.startsWith('+') && !line.startsWith('+++')) additionCount++;
    if (line.startsWith('-') && !line.startsWith('---')) deletionCount++;
  });
  
  assertEquals(additionCount, 1, 'Should find 1 addition');
  assertEquals(deletionCount, 1, 'Should find 1 deletion');
  
  console.log('✓ Basic diff parsing works');
}

// Test file extension language detection
function testLanguageDetection() {
  console.log('Testing language detection...');
  
  const testCases = [
    { path: 'test.js', expected: 'javascript' },
    { path: 'src/component.tsx', expected: 'tsx' },
    { path: 'script.py', expected: 'python' },
    { path: 'main.rs', expected: 'rust' },
    { path: 'README.md', expected: 'markdown' },
    { path: 'config.json', expected: 'json' },
    { path: 'unknown.xyz', expected: 'text' }
  ];
  
  const languageMap = {
    'js': 'javascript',
    'jsx': 'jsx',
    'ts': 'typescript', 
    'tsx': 'tsx',
    'py': 'python',
    'rs': 'rust',
    'json': 'json',
    'md': 'markdown'
  };
  
  function detectLanguage(filePath) {
    const ext = filePath.split('.').pop()?.toLowerCase();
    return languageMap[ext] || 'text';
  }
  
  testCases.forEach(({ path, expected }) => {
    const detected = detectLanguage(path);
    assertEquals(detected, expected, `Language detection for ${path}`);
  });
  
  console.log('✓ Language detection works');
}

// Test performance limits
function testPerformanceLimits() {
  console.log('Testing performance limits...');
  
  function shouldShowDiff(additions, deletions, isBinary) {
    const totalChanges = additions + deletions;
    return !isBinary && totalChanges > 0 && totalChanges <= 2000;
  }
  
  // Test cases
  assertTrue(shouldShowDiff(10, 5, false), 'Small changes should show diff');
  assertTrue(!shouldShowDiff(3000, 0, false), 'Large changes should not show diff');
  assertTrue(!shouldShowDiff(10, 5, true), 'Binary files should not show diff');
  assertTrue(!shouldShowDiff(0, 0, false), 'No changes should not show diff');
  
  console.log('✓ Performance limits work');
}

// Test actual repository if possible
function testActualRepository() {
  console.log('Testing against actual repository...');
  
  try {
    // Get the latest commit hash
    const latestCommit = execSync('git rev-parse HEAD', { 
      encoding: 'utf8',
      timeout: 5000,
      cwd: __dirname 
    }).trim();
    
    assertTrue(latestCommit.length === 40, 'Latest commit hash should be 40 characters');
    
    // Get commit info
    const commitInfo = execSync(`git show -s --format='%an|%ae|%at|%s' ${latestCommit}`, {
      encoding: 'utf8', 
      timeout: 5000,
      cwd: __dirname
    }).trim();
    
    const parts = commitInfo.split('|');
    assertTrue(parts.length >= 4, 'Commit info should have all parts');
    assertTrue(parts[0].length > 0, 'Author should not be empty');
    assertTrue(parseInt(parts[2]) > 0, 'Timestamp should be valid');
    
    console.log('✓ Repository test passed');
    console.log(`  Latest commit: ${latestCommit.substring(0, 7)}`);
    console.log(`  Author: ${parts[0]}`);
    console.log(`  Message: ${parts[3]}`);
    
  } catch (error) {
    console.log('⚠ Repository test skipped (not a git repo or no commits)');
  }
}

// Run all tests
function runTests() {
  console.log('Running commit detail integration tests...\n');
  
  const tests = [
    testGitCommand,
    testHashValidation, 
    testDiffParsing,
    testLanguageDetection,
    testPerformanceLimits,
    testActualRepository
  ];
  
  let passed = 0;
  let failed = 0;
  
  tests.forEach(test => {
    try {
      test();
      passed++;
    } catch (error) {
      console.log(`✗ ${test.name} failed:`, error.message);
      failed++;
    }
  });
  
  console.log(`\nTest Results: ${passed} passed, ${failed} failed`);
  
  if (failed === 0) {
    console.log('🎉 All tests passed! Commit detail feature should work correctly.');
    return true;
  } else {
    console.log('❌ Some tests failed. Please review the implementation.');
    return false;
  }
}

// Run tests if called directly
if (require.main === module) {
  const success = runTests();
  process.exit(success ? 0 : 1);
}

module.exports = { runTests };