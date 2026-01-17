// Name: SDK Test - Utility Functions
// Description: Tests uuid, compile, and HTTP methods (using native fetch)

/**
 * SDK TEST: test-utils.ts
 * 
 * Tests utility functions that don't require user interaction.
 * 
 * Test cases:
 * 1. utils-uuid: uuid() generation
 * 2. utils-compile: compile() template function
 * 3. http-get: native fetch() GET request (if network available)
 * 4. http-post: native fetch() POST request (if network available)
 * 
 * Expected behavior:
 * - uuid() generates valid v4 UUIDs
 * - compile() creates template functions
 * - Native fetch() returns response data
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
  expected?: string;
  actual?: string;
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

debug('test-utils.ts starting...');
debug(`SDK globals: uuid=${typeof uuid}, compile=${typeof compile}`);

// -----------------------------------------------------------------------------
// Test 1: uuid() generation
// -----------------------------------------------------------------------------
const testUuid = 'utils-uuid';
logTest(testUuid, 'running');
const startUuid = Date.now();

try {
  debug('Test: uuid() generation');
  
  const id1 = uuid();
  const id2 = uuid();
  
  debug(`Generated UUID 1: ${id1}`);
  debug(`Generated UUID 2: ${id2}`);
  
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  const checks = [
    uuidRegex.test(id1),
    uuidRegex.test(id2),
    id1 !== id2,
  ];
  
  debug(`UUIDs are unique: ${id1 !== id2}`);
  debug(`UUID format valid: ${uuidRegex.test(id1)}`);
  
  if (checks.every(Boolean)) {
    logTest(testUuid, 'pass', { result: id1, duration_ms: Date.now() - startUuid });
  } else {
    logTest(testUuid, 'fail', { 
      error: 'uuid() did not generate valid unique UUIDs',
      actual: id1,
      duration_ms: Date.now() - startUuid 
    });
  }
} catch (err) {
  logTest(testUuid, 'fail', { error: String(err), duration_ms: Date.now() - startUuid });
}

// -----------------------------------------------------------------------------
// Test: compile() template function
// -----------------------------------------------------------------------------
const testCompile = 'utils-compile';
logTest(testCompile, 'running');
const startCompile = Date.now();

try {
  debug('Test: compile() template function');
  
  const greet = compile('Hello, {{name}}! You are {{age}} years old.');
  const result1 = greet({ name: 'Alice', age: 30 });
  const expected = 'Hello, Alice! You are 30 years old.';
  
  debug(`Template result: ${result1}`);
  debug(`Expected: ${expected}`);
  
  // Test with missing key
  const result2 = greet({ name: 'Bob' });
  debug(`Template with missing key: ${result2}`);
  
  if (result1 === expected) {
    logTest(testCompile, 'pass', { result: result1, duration_ms: Date.now() - startCompile });
  } else {
    logTest(testCompile, 'fail', { 
      error: 'compile() did not produce expected output',
      expected,
      actual: result1,
      duration_ms: Date.now() - startCompile 
    });
  }
} catch (err) {
  logTest(testCompile, 'fail', { error: String(err), duration_ms: Date.now() - startCompile });
}

// -----------------------------------------------------------------------------
// Test 4: HTTP GET request (using native fetch)
// -----------------------------------------------------------------------------
const test4 = 'http-get';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: HTTP GET request (using native fetch)');
  
  const response = await fetch('https://httpbin.org/get');
  const data = await response.json();
  
  debug(`GET response has data: ${!!data}`);
  
  if (data && typeof data === 'object') {
    logTest(test4, 'pass', { result: 'GET request successful', duration_ms: Date.now() - start4 });
  } else {
    logTest(test4, 'fail', { 
      error: 'GET request did not return expected data',
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  debug(`HTTP GET test skipped (network unavailable): ${err}`);
  logTest(test4, 'skip', { error: 'Network unavailable', duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: HTTP POST request (using native fetch)
// -----------------------------------------------------------------------------
const test5 = 'http-post';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: HTTP POST request (using native fetch)');
  
  const response = await fetch('https://httpbin.org/post', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message: 'hello' })
  });
  const data = await response.json();
  
  debug(`POST response has data: ${!!data}`);
  
  if (data && typeof data === 'object') {
    logTest(test5, 'pass', { result: 'POST request successful', duration_ms: Date.now() - start5 });
  } else {
    logTest(test5, 'fail', { 
      error: 'POST request did not return expected data',
      duration_ms: Date.now() - start5 
    });
  }
} catch (err) {
  debug(`HTTP POST test skipped (network unavailable): ${err}`);
  logTest(test5, 'skip', { error: 'Network unavailable', duration_ms: Date.now() - start5 });
}

// -----------------------------------------------------------------------------
// Test 6: Window control functions (fire-and-forget)
// -----------------------------------------------------------------------------
const test6 = 'utils-window-control';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: Window control functions');
  
  // These are fire-and-forget, just verify they don't throw
  await show();
  debug('show() completed');
  
  // Note: hide() and blur() would affect the test runner, so just verify they exist
  const hasHide = typeof hide === 'function';
  const hasBlur = typeof blur === 'function';
  
  debug(`hide function exists: ${hasHide}`);
  debug(`blur function exists: ${hasBlur}`);
  
  if (hasHide && hasBlur) {
    logTest(test6, 'pass', { result: 'Window control functions available', duration_ms: Date.now() - start6 });
  } else {
    logTest(test6, 'fail', { 
      error: 'Window control functions not available',
      duration_ms: Date.now() - start6 
    });
  }
} catch (err) {
  logTest(test6, 'fail', { error: String(err), duration_ms: Date.now() - start6 });
}

// -----------------------------------------------------------------------------
// Test 7: Content setters (fire-and-forget)
// -----------------------------------------------------------------------------
const test7 = 'utils-content-setters';
logTest(test7, 'running');
const start7 = Date.now();

try {
  debug('Test 7: Content setter functions');
  
  // These are fire-and-forget, just verify they don't throw
  setPanel('<div>Panel content</div>');
  debug('setPanel() called');
  
  setPreview('<div>Preview content</div>');
  debug('setPreview() called');
  
  setPrompt('<div>Prompt content</div>');
  debug('setPrompt() called');
  
  logTest(test7, 'pass', { result: 'Content setters work', duration_ms: Date.now() - start7 });
} catch (err) {
  logTest(test7, 'fail', { error: String(err), duration_ms: Date.now() - start7 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-utils.ts completed!');

await div(md(`# Utility Functions Tests Complete

All utility function tests have been executed.

## Test Cases Run
1. **utils-uuid**: uuid() generation
2. **utils-compile**: compile() template function
3. **http-get**: Native fetch() GET request (network dependent)
4. **http-post**: Native fetch() POST request (network dependent)
5. **utils-window-control**: Window control functions
6. **utils-content-setters**: Content setter functions

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-utils.ts exiting...');
