"use client";

import { useState } from "react";

interface TestCase {
  id: string;
  name: string;
  mode: 'generate' | 'explain';
  input: string;
  expectedPatterns: string[];
  shouldNotContain?: string[];
}

interface TestResult {
  id: string;
  name: string;
  passed: boolean;
  score: number;
  output: string;
  matchedPatterns: string[];
  missedPatterns: string[];
  unexpectedContent?: string[];
  error?: string;
}

export const TEST_CASES: TestCase[] = [
  // English → Config tests
  {
    id: 'gen-basic-team',
    name: 'Basic team with agents',
    mode: 'generate',
    input: 'Two founders with full access. One AI agent that can only push to feature branches.',
    expectedPatterns: [
      'groups:',
      'founders:',
      'evm:0x',
      'permissions:',
      'push.*>feature',
      'own.*>\\*'
    ]
  },
  {
    id: 'gen-read-only',
    name: 'Read-only access',
    mode: 'generate',
    input: 'Owners can do everything. Viewers can only read the repository.',
    expectedPatterns: [
      'groups:',
      'owners:',
      'viewers:',
      'read.*>\\*',
      'own.*>\\*'
    ]
  },
  {
    id: 'gen-branch-verb',
    name: 'Branch creation permissions',
    mode: 'generate', 
    input: 'Developers can create and push to feature branches. Maintainers can merge anything.',
    expectedPatterns: [
      'branch.*>feature',
      'push.*>feature',
      'merge.*>\\*'
    ]
  },
  
  // Config → English tests  
  {
    id: 'exp-own-expansion',
    name: 'Own verb explanation',
    mode: 'explain',
    input: `groups:\n  founders:\n    - evm:0xAAA...111\npermissions:\n  default: deny\n  rules:\n    - founders own >*`,
    expectedPatterns: [
      'founders.*full access',
      'all branches',
      'read.*repository'
    ]
  },
  {
    id: 'exp-read-only',
    name: 'Read-only explanation',
    mode: 'explain',
    input: `groups:\n  viewers:\n    - evm:0xBBB...222\npermissions:\n  default: deny\n  rules:\n    - viewers read >*`,
    expectedPatterns: [
      'viewers.*read',
      'cannot.*push',
      'cannot.*edit'
    ]
  },
  {
    id: 'exp-file-restrictions',
    name: 'File path restrictions',
    mode: 'explain',
    input: `groups:\n  devs:\n    - evm:0xCCC...333\npermissions:\n  rules:\n    - devs edit ./src/**\n    - devs push >feature/**`,
    expectedPatterns: [
      'devs.*edit.*src',
      'push.*feature',
      'branch.*feature'
    ]
  }
];

export function TestRunner({ 
  onRunTest 
}: { 
  onRunTest: (input: string, mode: 'generate' | 'explain') => Promise<string>;
}) {
  const [testResults, setTestResults] = useState<TestResult[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [currentTest, setCurrentTest] = useState<string>("");

  const executeTest = async (testCase: TestCase): Promise<TestResult> => {
    try {
      setCurrentTest(testCase.name);
      
      const output = await onRunTest(testCase.input, testCase.mode);
      
      const matchedPatterns: string[] = [];
      const missedPatterns: string[] = [];
      
      testCase.expectedPatterns.forEach(pattern => {
        const regex = new RegExp(pattern, 'i');
        if (regex.test(output)) {
          matchedPatterns.push(pattern);
        } else {
          missedPatterns.push(pattern);
        }
      });
      
      const unexpectedContent: string[] = [];
      if (testCase.shouldNotContain) {
        testCase.shouldNotContain.forEach(forbidden => {
          const regex = new RegExp(forbidden, 'i');
          if (regex.test(output)) {
            unexpectedContent.push(forbidden);
          }
        });
      }
      
      const score = matchedPatterns.length / testCase.expectedPatterns.length;
      const passed = score >= 0.8 && unexpectedContent.length === 0;
      
      return {
        id: testCase.id,
        name: testCase.name,
        passed,
        score,
        output,
        matchedPatterns,
        missedPatterns,
        unexpectedContent: unexpectedContent.length > 0 ? unexpectedContent : undefined
      };
    } catch (error: any) {
      return {
        id: testCase.id,
        name: testCase.name,
        passed: false,
        score: 0,
        output: "",
        matchedPatterns: [],
        missedPatterns: testCase.expectedPatterns,
        error: error.message
      };
    }
  };

  const runAllTests = async () => {
    setIsRunning(true);
    setTestResults([]);
    
    const results: TestResult[] = [];
    
    for (const testCase of TEST_CASES) {
      const result = await executeTest(testCase);
      results.push(result);
      setTestResults([...results]);
      
      // Brief pause between tests to avoid rate limiting
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    setIsRunning(false);
    setCurrentTest("");
  };

  const runSingleTest = async (testCase: TestCase) => {
    setIsRunning(true);
    setCurrentTest(testCase.name);
    
    const result = await executeTest(testCase);
    
    setTestResults(prev => {
      const filtered = prev.filter(r => r.id !== testCase.id);
      return [...filtered, result];
    });
    
    setIsRunning(false);
    setCurrentTest("");
  };

  const passedTests = testResults.filter(r => r.passed).length;
  const totalTests = testResults.length;
  const overallScore = totalTests > 0 ? Math.round((passedTests / totalTests) * 100) : 0;

  return (
    <div className="test-runner">
      <div className="test-header">
        <h3>Test Suite</h3>
        <div className="test-controls">
          <button 
            className="test-run-all-btn"
            onClick={runAllTests}
            disabled={isRunning}
          >
            {isRunning ? "Running..." : "Run All Tests"}
          </button>
          {totalTests > 0 && (
            <div className="test-score">
              {passedTests}/{totalTests} passed ({overallScore}%)
            </div>
          )}
        </div>
      </div>

      {isRunning && currentTest && (
        <div className="test-status">
          Running: {currentTest}...
        </div>
      )}

      <div className="test-cases">
        {TEST_CASES.map(testCase => {
          const result = testResults.find(r => r.id === testCase.id);
          
          return (
            <div 
              key={testCase.id} 
              className={`test-case ${result ? (result.passed ? 'passed' : 'failed') : ''}`}
            >
              <div className="test-case-header">
                <div className="test-case-info">
                  <span className="test-case-name">{testCase.name}</span>
                  <span className="test-case-mode">{testCase.mode}</span>
                </div>
                <div className="test-case-actions">
                  {result && (
                    <span className="test-score">
                      {Math.round(result.score * 100)}%
                    </span>
                  )}
                  <button
                    className="test-run-single-btn"
                    onClick={() => runSingleTest(testCase)}
                    disabled={isRunning}
                  >
                    Run
                  </button>
                </div>
              </div>

              {result && (
                <div className="test-result">
                  {result.error ? (
                    <div className="test-error">Error: {result.error}</div>
                  ) : (
                    <>
                      <div className="test-patterns">
                        {result.matchedPatterns.length > 0 && (
                          <div className="matched-patterns">
                            ✅ Matched: {result.matchedPatterns.join(', ')}
                          </div>
                        )}
                        {result.missedPatterns.length > 0 && (
                          <div className="missed-patterns">
                            ❌ Missed: {result.missedPatterns.join(', ')}
                          </div>
                        )}
                        {result.unexpectedContent && result.unexpectedContent.length > 0 && (
                          <div className="unexpected-content">
                            ⚠️ Unexpected: {result.unexpectedContent.join(', ')}
                          </div>
                        )}
                      </div>
                      <details className="test-output">
                        <summary>Output ({result.output.length} chars)</summary>
                        <pre>{result.output}</pre>
                      </details>
                    </>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}