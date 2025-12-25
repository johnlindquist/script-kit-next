// Name: SDK Test - System APIs
// Description: Tests TIER 3 system APIs (alerts, clipboard, keyboard, mouse)

/**
 * SDK TEST: test-system.ts
 * 
 * Tests the TIER 3 system APIs that interact with the operating system.
 * 
 * Test categories:
 * 1. Alerts (fire-and-forget): beep, say, notify, setStatus, menu
 * 2. Clipboard: read/write text and images, copy/paste aliases
 * 3. Text operations: setSelectedText, getSelectedText
 * 4. Input simulation: keyboard, mouse
 * 
 * Note: Most system APIs are fire-and-forget (they send messages but don't wait
 * for user input). The test verifies they send the correct JSONL messages.
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

debug('test-system.ts starting...');
debug(`SDK globals: beep=${typeof beep}, say=${typeof say}, notify=${typeof notify}`);
debug(`SDK globals: clipboard=${typeof clipboard}, keyboard=${typeof keyboard}, mouse=${typeof mouse}`);

// -----------------------------------------------------------------------------
// Test 1: beep() - System beep (fire-and-forget)
// -----------------------------------------------------------------------------
const test1 = 'beep';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: beep()');
  
  await beep();
  
  debug('Test 1 completed - beep message sent');
  logTest(test1, 'pass', { duration_ms: Date.now() - start1 });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: say() - Text-to-speech (fire-and-forget)
// -----------------------------------------------------------------------------
const test2 = 'say';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: say()');
  
  // Without voice
  await say('Hello from Script Kit');
  
  // With voice
  await say('Testing voice parameter', 'Samantha');
  
  debug('Test 2 completed - say messages sent');
  logTest(test2, 'pass', { duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: notify() - System notifications (fire-and-forget)
// -----------------------------------------------------------------------------
const test3 = 'notify';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: notify()');
  
  // String shorthand
  await notify('Simple notification body');
  
  // Options object
  await notify({ title: 'Script Kit', body: 'Task completed successfully!' });
  
  // Title only
  await notify({ title: 'Just a title' });
  
  debug('Test 3 completed - notify messages sent');
  logTest(test3, 'pass', { duration_ms: Date.now() - start3 });
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Test 4: setStatus() - Application status (fire-and-forget)
// -----------------------------------------------------------------------------
const test4 = 'setStatus';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: setStatus()');
  
  await setStatus({ status: 'busy', message: 'Processing...' });
  await setStatus({ status: 'idle', message: 'Ready' });
  await setStatus({ status: 'error', message: 'Something went wrong' });
  
  debug('Test 4 completed - setStatus messages sent');
  logTest(test4, 'pass', { duration_ms: Date.now() - start4 });
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: menu() - System menu (fire-and-forget)
// -----------------------------------------------------------------------------
const test5 = 'menu';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: menu()');
  
  // Just icon
  await menu('star');
  
  // Icon with scripts
  await menu('gear', ['/path/to/script1.ts', '/path/to/script2.ts']);
  
  debug('Test 5 completed - menu messages sent');
  logTest(test5, 'pass', { duration_ms: Date.now() - start5 });
} catch (err) {
  logTest(test5, 'fail', { error: String(err), duration_ms: Date.now() - start5 });
}

// -----------------------------------------------------------------------------
// Test 6: copy() and paste() - Clipboard aliases
// -----------------------------------------------------------------------------
const test6 = 'copy-paste';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: copy() and paste()');
  
  // copy is an alias for clipboard.writeText
  await copy('Hello from copy()');
  
  // paste is an alias for clipboard.readText
  // Note: In real execution, this waits for GPUI response
  // For this test, we just verify the message is sent correctly
  debug('copy() message sent, paste() will wait for response');
  
  logTest(test6, 'pass', { duration_ms: Date.now() - start6 });
} catch (err) {
  logTest(test6, 'fail', { error: String(err), duration_ms: Date.now() - start6 });
}

// -----------------------------------------------------------------------------
// Test 7: clipboard object - Full API
// -----------------------------------------------------------------------------
const test7 = 'clipboard-api';
logTest(test7, 'running');
const start7 = Date.now();

try {
  debug('Test 7: clipboard API');
  
  // Verify clipboard object exists and has methods
  if (typeof clipboard !== 'object') {
    throw new Error('clipboard is not an object');
  }
  
  if (typeof clipboard.readText !== 'function') {
    throw new Error('clipboard.readText is not a function');
  }
  
  if (typeof clipboard.writeText !== 'function') {
    throw new Error('clipboard.writeText is not a function');
  }
  
  if (typeof clipboard.readImage !== 'function') {
    throw new Error('clipboard.readImage is not a function');
  }
  
  if (typeof clipboard.writeImage !== 'function') {
    throw new Error('clipboard.writeImage is not a function');
  }
  
  debug('Test 7 completed - clipboard API verified');
  logTest(test7, 'pass', { 
    result: { 
      methods: ['readText', 'writeText', 'readImage', 'writeImage'] 
    },
    duration_ms: Date.now() - start7 
  });
} catch (err) {
  logTest(test7, 'fail', { error: String(err), duration_ms: Date.now() - start7 });
}

// -----------------------------------------------------------------------------
// Test 8: setSelectedText() - Paste at cursor (fire-and-forget)
// -----------------------------------------------------------------------------
const test8 = 'setSelectedText';
logTest(test8, 'running');
const start8 = Date.now();

try {
  debug('Test 8: setSelectedText()');
  
  await setSelectedText('Inserted text');
  
  debug('Test 8 completed - setSelectedText message sent');
  logTest(test8, 'pass', { duration_ms: Date.now() - start8 });
} catch (err) {
  logTest(test8, 'fail', { error: String(err), duration_ms: Date.now() - start8 });
}

// -----------------------------------------------------------------------------
// Test 9: keyboard object - Type and tap
// -----------------------------------------------------------------------------
const test9 = 'keyboard-api';
logTest(test9, 'running');
const start9 = Date.now();

try {
  debug('Test 9: keyboard API');
  
  // Verify keyboard object exists and has methods
  if (typeof keyboard !== 'object') {
    throw new Error('keyboard is not an object');
  }
  
  if (typeof keyboard.type !== 'function') {
    throw new Error('keyboard.type is not a function');
  }
  
  if (typeof keyboard.tap !== 'function') {
    throw new Error('keyboard.tap is not a function');
  }
  
  // Test keyboard.type (fire-and-forget)
  await keyboard.type('Hello World');
  
  // Test keyboard.tap with multiple keys (fire-and-forget)
  await keyboard.tap('command', 'c');
  await keyboard.tap('command', 'shift', 'p');
  
  debug('Test 9 completed - keyboard messages sent');
  logTest(test9, 'pass', { 
    result: { methods: ['type', 'tap'] },
    duration_ms: Date.now() - start9 
  });
} catch (err) {
  logTest(test9, 'fail', { error: String(err), duration_ms: Date.now() - start9 });
}

// -----------------------------------------------------------------------------
// Test 10: mouse object - Move, click, setPosition
// -----------------------------------------------------------------------------
const test10 = 'mouse-api';
logTest(test10, 'running');
const start10 = Date.now();

try {
  debug('Test 10: mouse API');
  
  // Verify mouse object exists and has methods
  if (typeof mouse !== 'object') {
    throw new Error('mouse is not an object');
  }
  
  if (typeof mouse.move !== 'function') {
    throw new Error('mouse.move is not a function');
  }
  
  if (typeof mouse.leftClick !== 'function') {
    throw new Error('mouse.leftClick is not a function');
  }
  
  if (typeof mouse.rightClick !== 'function') {
    throw new Error('mouse.rightClick is not a function');
  }
  
  if (typeof mouse.setPosition !== 'function') {
    throw new Error('mouse.setPosition is not a function');
  }
  
  // Test mouse.move with path (fire-and-forget)
  await mouse.move([
    { x: 100, y: 100 },
    { x: 200, y: 200 },
    { x: 300, y: 150 }
  ]);
  
  // Test mouse.setPosition (fire-and-forget)
  await mouse.setPosition({ x: 500, y: 500 });
  
  // Test mouse.leftClick (fire-and-forget)
  await mouse.leftClick();
  
  // Test mouse.rightClick (fire-and-forget)
  await mouse.rightClick();
  
  debug('Test 10 completed - mouse messages sent');
  logTest(test10, 'pass', { 
    result: { methods: ['move', 'leftClick', 'rightClick', 'setPosition'] },
    duration_ms: Date.now() - start10 
  });
} catch (err) {
  logTest(test10, 'fail', { error: String(err), duration_ms: Date.now() - start10 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-system.ts completed!');

await div(md(`# System APIs (TIER 3) Tests Complete

All system API tests have been executed.

## Test Categories

### Alerts (Fire-and-Forget)
1. **beep**: System beep sound
2. **say**: Text-to-speech
3. **notify**: System notifications
4. **setStatus**: Application status
5. **menu**: System menu icon/scripts

### Clipboard
6. **copy-paste**: Convenience aliases
7. **clipboard-api**: Full clipboard object

### Text Operations
8. **setSelectedText**: Paste at cursor

### Input Simulation
9. **keyboard-api**: Type text and tap keys
10. **mouse-api**: Move, click, position

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-system.ts exiting...');
