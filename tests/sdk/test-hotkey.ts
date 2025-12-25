// Name: SDK Test - hotkey()
// Description: Tests hotkey() keyboard shortcut capture

/**
 * SDK TEST: test-hotkey.ts
 * 
 * Tests the hotkey() function which captures keyboard shortcuts.
 * 
 * Test cases:
 * 1. hotkey-basic: Basic hotkey capture
 * 2. hotkey-placeholder: Hotkey with placeholder text
 * 
 * Expected behavior:
 * - hotkey() sends JSONL message with type: 'hotkey'
 * - Returns HotkeyInfo with key, modifiers, shortcut, keyCode
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

debug('test-hotkey.ts starting...');
debug(`SDK globals: hotkey=${typeof hotkey}`);

// -----------------------------------------------------------------------------
// Test 1: Basic hotkey capture
// -----------------------------------------------------------------------------
const test1 = 'hotkey-basic';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: hotkey() with no arguments');
  
  const result = await hotkey();
  
  debug(`Test 1 result: key="${result.key}", shortcut="${result.shortcut}"`);
  debug(`  Modifiers: cmd=${result.command}, shift=${result.shift}, opt=${result.option}, ctrl=${result.control}`);
  
  // Verify the result has the expected structure
  const hasRequiredFields = 
    typeof result.key === 'string' &&
    typeof result.command === 'boolean' &&
    typeof result.shift === 'boolean' &&
    typeof result.option === 'boolean' &&
    typeof result.control === 'boolean' &&
    typeof result.shortcut === 'string' &&
    typeof result.keyCode === 'string';
  
  if (hasRequiredFields) {
    logTest(test1, 'pass', { result, duration_ms: Date.now() - start1 });
  } else {
    logTest(test1, 'fail', { 
      result, 
      error: 'Missing required fields in HotkeyInfo',
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Hotkey with placeholder
// -----------------------------------------------------------------------------
const test2 = 'hotkey-placeholder';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: hotkey() with placeholder');
  
  const result = await hotkey('Press a keyboard shortcut');
  
  debug(`Test 2 result: shortcut="${result.shortcut}"`);
  
  logTest(test2, 'pass', { result, duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-hotkey.ts completed!');

await div(md(`# hotkey() Tests Complete

All \`hotkey()\` tests have been executed.

## Test Cases Run
1. **hotkey-basic**: Basic hotkey capture
2. **hotkey-placeholder**: Hotkey with placeholder text

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-hotkey.ts exiting...');
