// Name: SDK Test - Widget and Media APIs
// Description: Tests widget, term, webcam, mic, eyeDropper, and find functions

/**
 * SDK TEST: test-widget.ts
 * 
 * Tests the widget(), term(), and media prompt functions.
 * 
 * Test cases:
 * 1. widget-basic: Basic widget creation
 * 2. widget-options: Widget with position/size options
 * 3. widget-events: Widget event handlers exist
 * 4. term-basic: term() function exists
 * 5. media-apis: Media API functions exist
 * 6. find-api: find() function exists
 * 
 * Expected behavior:
 * - widget() creates floating HTML panels
 * - term() opens terminal
 * - Media APIs (webcam, mic, eyeDropper) are available
 * - find() provides file search
 */

import '../../scripts/kit-sdk';

// =============================================================================
// Helpers
// =============================================================================

// Local delay helper (wait() was removed from SDK)
const wait = (ms: number): Promise<void> => new Promise(r => setTimeout(r, ms));

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

debug('test-widget.ts starting...');
debug(`SDK globals: widget=${typeof widget}, term=${typeof term}`);

// -----------------------------------------------------------------------------
// Test 1: Basic widget creation
// -----------------------------------------------------------------------------
const test1 = 'widget-basic';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: Basic widget creation');
  
  const w = await widget(`
    <div style="padding: 20px; background: #1e1e1e; color: white; border-radius: 8px;">
      <h2>Hello Widget!</h2>
      <p>This is a floating HTML widget.</p>
      <button data-action="close">Close</button>
    </div>
  `);
  
  debug('Widget created successfully');
  debug(`Widget has close method: ${typeof w.close === 'function'}`);
  debug(`Widget has onClick method: ${typeof w.onClick === 'function'}`);
  
  // Set up close handler and close immediately
  w.onClick((event) => {
    if (event.dataset?.action === 'close') {
      w.close();
    }
  });
  
  // Close after brief display
  setTimeout(() => w.close(), 500);
  
  if (typeof w.close === 'function' && typeof w.onClick === 'function') {
    logTest(test1, 'pass', { result: 'Widget created with methods', duration_ms: Date.now() - start1 });
  } else {
    logTest(test1, 'fail', { 
      error: 'Widget missing expected methods',
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Widget with options
// -----------------------------------------------------------------------------
const test2 = 'widget-options';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: Widget with position/size options');
  
  const w = await widget(
    `<div style="padding: 16px;">
      <h3>Positioned Widget</h3>
      <p>I have custom position and size!</p>
    </div>`,
    {
      x: 100,
      y: 100,
      width: 300,
      height: 200,
      alwaysOnTop: true,
      transparent: false,
      draggable: true,
      hasShadow: true,
    }
  );
  
  debug('Widget with options created successfully');
  
  // Close after brief display
  setTimeout(() => w.close(), 500);
  
  logTest(test2, 'pass', { result: 'Widget with options created', duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Widget event handlers
// -----------------------------------------------------------------------------
const test3 = 'widget-events';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: Widget event handlers exist');
  
  const w = await widget(`<div style="padding: 20px;"><h3>Event Test</h3></div>`);
  
  const hasOnClick = typeof w.onClick === 'function';
  const hasOnInput = typeof w.onInput === 'function';
  const hasOnMoved = typeof w.onMoved === 'function';
  const hasOnResized = typeof w.onResized === 'function';
  const hasOnClose = typeof w.onClose === 'function';
  const hasSetState = typeof w.setState === 'function';
  
  debug(`onClick: ${hasOnClick}`);
  debug(`onInput: ${hasOnInput}`);
  debug(`onMoved: ${hasOnMoved}`);
  debug(`onResized: ${hasOnResized}`);
  debug(`onClose: ${hasOnClose}`);
  debug(`setState: ${hasSetState}`);
  
  // Close after check
  setTimeout(() => w.close(), 500);
  
  const checks = [hasOnClick, hasOnInput, hasOnMoved, hasOnResized, hasOnClose, hasSetState];
  
  if (checks.every(Boolean)) {
    logTest(test3, 'pass', { result: 'All event handlers present', duration_ms: Date.now() - start3 });
  } else {
    logTest(test3, 'fail', { 
      error: 'Some event handlers missing',
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// Wait for widgets to close
await wait(600);

// -----------------------------------------------------------------------------
// Test 4: term() function exists
// -----------------------------------------------------------------------------
const test4 = 'term-exists';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: term() function exists');
  
  const hasTerm = typeof term === 'function';
  
  debug(`term function exists: ${hasTerm}`);
  
  if (hasTerm) {
    logTest(test4, 'pass', { result: 'term() function available', duration_ms: Date.now() - start4 });
  } else {
    logTest(test4, 'fail', { 
      error: 'term() function not found',
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Test 5: Media API functions exist
// -----------------------------------------------------------------------------
const test5 = 'media-apis';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: Media API functions exist');
  
  const hasWebcam = typeof webcam === 'function';
  const hasMic = typeof mic === 'function';
  const hasEyeDropper = typeof eyeDropper === 'function';
  
  debug(`webcam function exists: ${hasWebcam}`);
  debug(`mic function exists: ${hasMic}`);
  debug(`eyeDropper function exists: ${hasEyeDropper}`);
  
  const checks = [hasWebcam, hasMic, hasEyeDropper];
  
  if (checks.every(Boolean)) {
    logTest(test5, 'pass', { result: 'All media APIs available', duration_ms: Date.now() - start5 });
  } else {
    logTest(test5, 'fail', { 
      error: 'Some media APIs missing',
      duration_ms: Date.now() - start5 
    });
  }
} catch (err) {
  logTest(test5, 'fail', { error: String(err), duration_ms: Date.now() - start5 });
}

// -----------------------------------------------------------------------------
// Test 6: find() function exists
// -----------------------------------------------------------------------------
const test6 = 'find-api';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: find() function exists');
  
  const hasFind = typeof find === 'function';
  
  debug(`find function exists: ${hasFind}`);
  
  if (hasFind) {
    logTest(test6, 'pass', { result: 'find() function available', duration_ms: Date.now() - start6 });
  } else {
    logTest(test6, 'fail', { 
      error: 'find() function not found',
      duration_ms: Date.now() - start6 
    });
  }
} catch (err) {
  logTest(test6, 'fail', { error: String(err), duration_ms: Date.now() - start6 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-widget.ts completed!');

await div(md(`# Widget and Media Tests Complete

All widget and media API tests have been executed.

## Test Cases Run
1. **widget-basic**: Basic widget creation
2. **widget-options**: Widget with position/size options
3. **widget-events**: Widget event handlers
4. **term-exists**: term() function availability
5. **media-apis**: webcam, mic, eyeDropper availability
6. **find-api**: find() function availability

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`));

debug('test-widget.ts exiting...');
