// Name: SDK Test - path()
// Description: Tests path() file/folder browser prompt

/**
 * SDK TEST: test-path.ts
 * 
 * Tests the path() function which provides file/folder browsing.
 * 
 * Test cases:
 * 1. path-basic: Basic path selection
 * 2. path-startpath: Path with startPath option
 * 3. path-hint: Path with hint option
 * 
 * Expected behavior:
 * - path() sends JSONL message with type: 'path'
 * - Options are passed to the message
 * - Selected path is returned
 */

import '../../scripts/kit-sdk';

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-path.ts starting...');
debug(`SDK globals: path=${typeof path}`);

// -----------------------------------------------------------------------------
// Test 1: Basic path selection
// -----------------------------------------------------------------------------
const test1 = 'path-basic';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: path() with no options');
  
  const result = await path();
  
  debug(`Test 1 result: "${result}"`);
  
  logTest(test1, 'pass', { result, duration_ms: Date.now() - start1 });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Path with startPath option
// -----------------------------------------------------------------------------
const test2 = 'path-startpath';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: path() with startPath option');
  
  const result = await path({ startPath: process.env.HOME || '/' });
  
  debug(`Test 2 result: "${result}"`);
  
  logTest(test2, 'pass', { result, duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Path with hint option
// -----------------------------------------------------------------------------
const test3 = 'path-hint';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: path() with hint option');
  
  const result = await path({ hint: 'Select a configuration file' });
  
  debug(`Test 3 result: "${result}"`);
  
  logTest(test3, 'pass', { result, duration_ms: Date.now() - start3 });
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-path.ts completed!');

await div(md(`# path() Tests Complete

All \`path()\` tests have been executed.

## Test Cases Run
1. **path-basic**: Basic path selection
2. **path-startpath**: Path with startPath option
3. **path-hint**: Path with hint option

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-path.ts exiting...');
