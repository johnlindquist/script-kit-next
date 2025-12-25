// Name: SDK Test - editor(), mini(), micro()
// Description: Tests editor, mini, and micro prompt functions

/**
 * SDK TEST: test-editor.ts
 * 
 * Tests the editor(), mini(), and micro() functions.
 * 
 * Test cases:
 * 1. editor-basic: Basic editor with default content
 * 2. editor-language: Editor with specific language syntax highlighting
 * 3. mini-basic: Mini prompt with string choices
 * 4. micro-basic: Micro prompt with string choices
 * 
 * Expected behavior:
 * - editor() sends JSONL message with type: 'editor'
 * - mini() sends JSONL message with type: 'mini'  
 * - micro() sends JSONL message with type: 'micro'
 * - User input is returned as the value
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

debug('test-editor.ts starting...');
debug(`SDK globals: editor=${typeof editor}, mini=${typeof mini}, micro=${typeof micro}`);

// -----------------------------------------------------------------------------
// Test 1: editor with default content
// -----------------------------------------------------------------------------
const test1 = 'editor-basic';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: editor() with empty content');
  
  const result = await editor();
  
  debug(`Test 1 result: "${result.substring(0, 50)}..."`);
  logTest(test1, 'pass', { result: result.length, duration_ms: Date.now() - start1 });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: editor with language
// -----------------------------------------------------------------------------
const test2 = 'editor-language';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: editor() with TypeScript content');
  
  const initialCode = `// Hello World
function greet(name: string): string {
  return \`Hello, \${name}!\`;
}

console.log(greet('World'));
`;
  
  const result = await editor(initialCode, 'typescript');
  
  debug(`Test 2 result: "${result.substring(0, 50)}..."`);
  logTest(test2, 'pass', { result: result.length, duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: mini prompt
// -----------------------------------------------------------------------------
const test3 = 'mini-basic';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: mini() with string choices');
  
  const result = await mini('Quick action', [
    'Copy',
    'Paste', 
    'Cut',
    'Delete'
  ]);
  
  debug(`Test 3 result: "${result}"`);
  logTest(test3, 'pass', { result, duration_ms: Date.now() - start3 });
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Test 4: micro prompt
// -----------------------------------------------------------------------------
const test4 = 'micro-basic';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: micro() with string choices');
  
  const result = await micro('Yes or No?', [
    'Yes',
    'No'
  ]);
  
  debug(`Test 4 result: "${result}"`);
  logTest(test4, 'pass', { result, duration_ms: Date.now() - start4 });
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: mini with structured choices
// -----------------------------------------------------------------------------
const test5 = 'mini-structured';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: mini() with structured choices');
  
  const result = await mini('Select theme', [
    { name: 'Light Mode', value: 'light', description: 'Bright theme for daytime' },
    { name: 'Dark Mode', value: 'dark', description: 'Dark theme for nighttime' },
    { name: 'System', value: 'system', description: 'Follow system preferences' }
  ]);
  
  debug(`Test 5 result: "${result}"`);
  logTest(test5, 'pass', { result, duration_ms: Date.now() - start5 });
} catch (err) {
  logTest(test5, 'fail', { error: String(err), duration_ms: Date.now() - start5 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-editor.ts completed!');

await div(md(`# editor/mini/micro Tests Complete

All prompt tests have been executed.

## Test Cases Run
1. **editor-basic**: Editor with empty content
2. **editor-language**: Editor with TypeScript syntax highlighting
3. **mini-basic**: Mini prompt with string choices
4. **micro-basic**: Micro prompt with string choices
5. **mini-structured**: Mini prompt with structured choices

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-editor.ts exiting...');
