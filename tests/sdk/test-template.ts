// Name: SDK Test - template()
// Description: Tests template() tab-through snippet editing

/**
 * SDK TEST: test-template.ts
 * 
 * Tests the template() function which provides VSCode-like snippet editing.
 * 
 * Test cases:
 * 1. template-simple: Simple numbered placeholders
 * 2. template-defaults: Placeholders with default values
 * 3. template-multiline: Multi-line code template
 * 
 * Expected behavior:
 * - template() sends JSONL message with type: 'template'
 * - User can tab through placeholders
 * - Filled template is returned
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
  console.error('[TEST] ' + msg);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-template.ts starting...');
debug('SDK globals: template=' + typeof template);

// -----------------------------------------------------------------------------
// Test 1: Simple template with numbered placeholders
// -----------------------------------------------------------------------------
const test1 = 'template-simple';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: template() with simple placeholders');
  
  // Using regular string to avoid template literal issues with $1, $2
  const result = await template('Hello $1, welcome to $2!');
  
  debug('Test 1 result: "' + result + '"');
  
  logTest(test1, 'pass', { result, duration_ms: Date.now() - start1 });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Template with default values
// -----------------------------------------------------------------------------
const test2 = 'template-defaults';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: template() with default values');
  
  // eslint-disable-next-line no-template-curly-in-string
  const result = await template('function ${1:functionName}(${2:params}) {\n  $3\n}');
  
  debug('Test 2 result:\n' + result);
  
  logTest(test2, 'pass', { result, duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Multi-line code template
// -----------------------------------------------------------------------------
const test3 = 'template-multiline';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: template() with multi-line code');
  
  // eslint-disable-next-line no-template-curly-in-string
  const result = await template('interface ${1:Name} {\n  ${2:property}: ${3:string};\n}');
  
  debug('Test 3 result:\n' + result);
  
  logTest(test3, 'pass', { result, duration_ms: Date.now() - start3 });
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-template.ts completed!');

await div(md('# template() Tests Complete\n\nAll `template()` tests have been executed.\n\n## Test Cases Run\n1. **template-simple**: Simple numbered placeholders\n2. **template-defaults**: Placeholders with default values\n3. **template-multiline**: Multi-line code template\n\n---\n\n*Check the JSONL output for detailed results*\n\nPress Escape or click to exit.'));

debug('test-template.ts exiting...');
