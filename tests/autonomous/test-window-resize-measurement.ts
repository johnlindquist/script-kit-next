// Window Resize Measurement Tests
// Tests window dimensions after various prompts to verify resize behavior
//
// Uses getWindowBounds() SDK function to measure actual window dimensions
// and validates against expected layout constants from window_resize.rs

import '../../scripts/kit-sdk';
import { saveScreenshot, analyzeContentFill, generateReport, ensureScreenshotDir } from './screenshot-utils';

// =============================================================================
// Layout Constants (from src/window_resize.rs)
// =============================================================================

const LAYOUT = {
  MIN_HEIGHT: 120,      // Compact mode (header only, no list)
  MAX_HEIGHT: 700,      // Editor/div/term prompts
  HEADER_HEIGHT: 100,   // Logo + input field area
  LIST_ITEM_HEIGHT: 52, // Height of each list item
  FOOTER_HEIGHT: 44,    // Footer/actions bar
  WINDOW_WIDTH: 750,    // Fixed window width
  MAX_VISIBLE_ITEMS: 10,// Max items before scrolling
  LIST_PADDING: 8,      // Padding at bottom of list area
} as const;

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

async function runTest(name: string, fn: () => Promise<void>) {
  logTest(name, 'running');
  const start = Date.now();
  try {
    await fn();
    logTest(name, 'pass', { duration_ms: Date.now() - start });
  } catch (err) {
    logTest(name, 'fail', { error: String(err), duration_ms: Date.now() - start });
  }
}

// Tolerance for height comparisons (pixels)
const HEIGHT_TOLERANCE = 20;

function assertHeightInRange(actual: number, expected: number, description: string) {
  const diff = Math.abs(actual - expected);
  if (diff > HEIGHT_TOLERANCE) {
    throw new Error(
      `${description}: expected height ~${expected}px (+-${HEIGHT_TOLERANCE}), got ${actual}px (diff: ${diff}px)`
    );
  }
  debug(`${description}: height=${actual}px (expected=${expected}px, diff=${diff}px) - OK`);
}

function assertWidth(actual: number, expected: number, description: string) {
  // Width should be exact
  if (actual !== expected) {
    throw new Error(
      `${description}: expected width ${expected}px, got ${actual}px`
    );
  }
  debug(`${description}: width=${actual}px - OK`);
}

/**
 * Calculate expected height for a list with N items
 * Formula from window_resize.rs: header + list_height + footer + padding
 * where list_height = min(item_count, MAX_VISIBLE_ITEMS) * LIST_ITEM_HEIGHT
 */
function calculateExpectedListHeight(itemCount: number): number {
  if (itemCount === 0) {
    return LAYOUT.MIN_HEIGHT;
  }
  
  const visibleItems = Math.min(itemCount, LAYOUT.MAX_VISIBLE_ITEMS);
  const listHeight = visibleItems * LAYOUT.LIST_ITEM_HEIGHT;
  const totalHeight = LAYOUT.HEADER_HEIGHT + listHeight + LAYOUT.FOOTER_HEIGHT + LAYOUT.LIST_PADDING;
  
  // Clamp to min/max
  return Math.max(LAYOUT.MIN_HEIGHT, Math.min(LAYOUT.MAX_HEIGHT, totalHeight));
}

// =============================================================================
// Tests
// =============================================================================

debug('test-window-resize-measurement.ts starting...');

// Ensure screenshot directory exists before tests
await ensureScreenshotDir();

// -----------------------------------------------------------------------------
// Test 1: Verify window width is constant
// -----------------------------------------------------------------------------

await runTest('window-width-constant', async () => {
  const bounds = await getWindowBounds();
  await wait(100); // Allow settle time
  
  assertWidth(bounds.width, LAYOUT.WINDOW_WIDTH, 'Initial window width');
  debug(`Window position: x=${bounds.x}, y=${bounds.y}`);
});

// -----------------------------------------------------------------------------
// Test 2: arg() with choices - should size based on list items
// -----------------------------------------------------------------------------

await runTest('arg-list-height-3-items', async () => {
  // Show arg prompt with 3 choices
  const choices = ['Apple', 'Banana', 'Cherry'];
  
  // We need to trigger the prompt and then measure before submitting
  // Use a setTimeout to measure after prompt renders
  const measurePromise = (async () => {
    await wait(150); // Wait for prompt to render and resize
    const bounds = await getWindowBounds();
    return bounds;
  })();
  
  // Start the arg prompt (will wait for user input)
  const argPromise = arg('Select a fruit', choices);
  
  // Get the measurement
  const bounds = await measurePromise;
  
  // Calculate expected height for 3 items
  const expectedHeight = calculateExpectedListHeight(3);
  assertHeightInRange(bounds.height, expectedHeight, 'arg() with 3 choices');
  assertWidth(bounds.width, LAYOUT.WINDOW_WIDTH, 'arg() width');
  
  debug(`Expected height for 3 items: ${expectedHeight}px`);
  debug(`Actual bounds: ${JSON.stringify(bounds)}`);
  
  // Submit to continue
  submit('Apple');
  await argPromise;
});

// -----------------------------------------------------------------------------
// Test 3: arg() with many choices - should cap at MAX_VISIBLE_ITEMS
// -----------------------------------------------------------------------------

await runTest('arg-list-height-many-items', async () => {
  // Show arg prompt with 20 choices (more than MAX_VISIBLE_ITEMS)
  const choices = Array.from({ length: 20 }, (_, i) => `Item ${i + 1}`);
  
  const measurePromise = (async () => {
    await wait(150);
    const bounds = await getWindowBounds();
    return bounds;
  })();
  
  const argPromise = arg('Select an item', choices);
  const bounds = await measurePromise;
  
  // Should cap at MAX_VISIBLE_ITEMS (10)
  const expectedHeight = calculateExpectedListHeight(20); // Will be capped internally
  assertHeightInRange(bounds.height, expectedHeight, 'arg() with 20 choices');
  
  debug(`Expected height for 20 items (capped to 10): ${expectedHeight}px`);
  debug(`Actual bounds: ${JSON.stringify(bounds)}`);
  
  submit('Item 1');
  await argPromise;
});

// -----------------------------------------------------------------------------
// Test 4: editor() prompt - should be MAX_HEIGHT
// -----------------------------------------------------------------------------

await runTest('editor-max-height', async () => {
  const measurePromise = (async () => {
    await wait(150);
    const bounds = await getWindowBounds();
    const screenshot = await captureScreenshot();
    return { bounds, screenshot };
  })();
  
  const editorPromise = editor('console.log("hello world")', 'javascript');
  const { bounds, screenshot } = await measurePromise;
  
  assertHeightInRange(bounds.height, LAYOUT.MAX_HEIGHT, 'editor() prompt height');
  assertWidth(bounds.width, LAYOUT.WINDOW_WIDTH, 'editor() width');
  
  debug(`Editor bounds: ${JSON.stringify(bounds)}`);
  
  // Capture screenshot for visual verification
  const screenshotPath = await saveScreenshot(screenshot.data, 'editor-max-height');
  
  // Analyze if editor fills the window
  const analysis = await analyzeContentFill(screenshotPath, LAYOUT.MAX_HEIGHT);
  if (!analysis.pass) {
    debug(`VISUAL CHECK FAILED: ${analysis.message}`);
    debug(`Screenshot saved to: ${screenshotPath}`);
    // Don't throw yet - log for debugging but let bounds check determine pass/fail
  }
  
  debug(`Screenshot captured: ${screenshotPath}`);
  debug(`Visual analysis: ${analysis.pass ? 'PASS' : 'FAIL'} - ${analysis.message}`);
  
  submit('console.log("submitted")');
  await editorPromise;
});

// -----------------------------------------------------------------------------
// Test 5: div() prompt - should be MAX_HEIGHT
// -----------------------------------------------------------------------------

await runTest('div-max-height', async () => {
  const measurePromise = (async () => {
    await wait(150);
    const bounds = await getWindowBounds();
    const screenshot = await captureScreenshot();
    return { bounds, screenshot };
  })();
  
  const divPromise = div('<h1>Hello World</h1><p>This is a div prompt test.</p>');
  const { bounds, screenshot } = await measurePromise;
  
  assertHeightInRange(bounds.height, LAYOUT.MAX_HEIGHT, 'div() prompt height');
  assertWidth(bounds.width, LAYOUT.WINDOW_WIDTH, 'div() width');
  
  debug(`Div bounds: ${JSON.stringify(bounds)}`);
  
  // Capture screenshot for visual verification
  const screenshotPath = await saveScreenshot(screenshot.data, 'div-max-height');
  
  // Analyze if div fills the window
  const analysis = await analyzeContentFill(screenshotPath, LAYOUT.MAX_HEIGHT);
  if (!analysis.pass) {
    debug(`VISUAL CHECK FAILED: ${analysis.message}`);
    debug(`Screenshot saved to: ${screenshotPath}`);
    // Don't throw yet - log for debugging but let bounds check determine pass/fail
  }
  
  debug(`Screenshot captured: ${screenshotPath}`);
  debug(`Visual analysis: ${analysis.pass ? 'PASS' : 'FAIL'} - ${analysis.message}`);
  
  submit(null);
  await divPromise;
});

// -----------------------------------------------------------------------------
// Test 6: Height transition - arg with list -> editor
// -----------------------------------------------------------------------------

await runTest('height-transition-arg-to-editor', async () => {
  // First, show arg with 3 items
  const argMeasurePromise = (async () => {
    await wait(150);
    return await getWindowBounds();
  })();
  
  const argPromise = arg('Select fruit', ['Apple', 'Banana', 'Cherry']);
  const argBounds = await argMeasurePromise;
  
  const expectedArgHeight = calculateExpectedListHeight(3);
  assertHeightInRange(argBounds.height, expectedArgHeight, 'arg() height before transition');
  
  submit('Apple');
  await argPromise;
  
  // Now show editor - should expand to MAX_HEIGHT
  await wait(50); // Small gap between prompts
  
  const editorMeasurePromise = (async () => {
    await wait(150);
    return await getWindowBounds();
  })();
  
  const editorPromise = editor('// Code here', 'typescript');
  const editorBounds = await editorMeasurePromise;
  
  assertHeightInRange(editorBounds.height, LAYOUT.MAX_HEIGHT, 'editor() height after transition');
  
  // Verify the height actually changed
  const heightDiff = editorBounds.height - argBounds.height;
  debug(`Height transition: ${argBounds.height}px -> ${editorBounds.height}px (diff: ${heightDiff}px)`);
  
  if (heightDiff < 50) {
    throw new Error(`Expected significant height increase, but only got ${heightDiff}px`);
  }
  
  submit('// Submitted');
  await editorPromise;
});

// -----------------------------------------------------------------------------
// Test 7: Height transition - editor -> arg with few items
// -----------------------------------------------------------------------------

await runTest('height-transition-editor-to-arg', async () => {
  // First, show editor at MAX_HEIGHT
  const editorMeasurePromise = (async () => {
    await wait(150);
    return await getWindowBounds();
  })();
  
  const editorPromise = editor('let x = 1;', 'javascript');
  const editorBounds = await editorMeasurePromise;
  
  assertHeightInRange(editorBounds.height, LAYOUT.MAX_HEIGHT, 'editor() height');
  
  submit('let x = 2;');
  await editorPromise;
  
  // Now show arg with 2 items - should shrink
  await wait(50);
  
  const argMeasurePromise = (async () => {
    await wait(150);
    return await getWindowBounds();
  })();
  
  const argPromise = arg('Pick one', ['Yes', 'No']);
  const argBounds = await argMeasurePromise;
  
  const expectedArgHeight = calculateExpectedListHeight(2);
  assertHeightInRange(argBounds.height, expectedArgHeight, 'arg() height after shrink');
  
  // Verify the height actually decreased
  const heightDiff = editorBounds.height - argBounds.height;
  debug(`Height shrink: ${editorBounds.height}px -> ${argBounds.height}px (diff: ${heightDiff}px)`);
  
  if (heightDiff < 50) {
    throw new Error(`Expected significant height decrease, but only got ${heightDiff}px`);
  }
  
  submit('Yes');
  await argPromise;
});

// -----------------------------------------------------------------------------
// Test 8: arg() with no choices - should be MIN_HEIGHT
// -----------------------------------------------------------------------------

await runTest('arg-no-choices-min-height', async () => {
  const measurePromise = (async () => {
    await wait(150);
    return await getWindowBounds();
  })();
  
  // arg with empty choices (text input mode)
  const argPromise = arg('Enter your name', []);
  const bounds = await measurePromise;
  
  // With no choices, should be compact mode (MIN_HEIGHT)
  assertHeightInRange(bounds.height, LAYOUT.MIN_HEIGHT, 'arg() with no choices');
  
  debug(`No-choices bounds: ${JSON.stringify(bounds)}`);
  
  submit('TestUser');
  await argPromise;
});

// -----------------------------------------------------------------------------
// Test 9: Summary - log all layout constants for verification
// -----------------------------------------------------------------------------

await runTest('log-layout-constants', async () => {
  debug('--- Layout Constants from window_resize.rs ---');
  debug(`MIN_HEIGHT: ${LAYOUT.MIN_HEIGHT}px`);
  debug(`MAX_HEIGHT: ${LAYOUT.MAX_HEIGHT}px`);
  debug(`HEADER_HEIGHT: ${LAYOUT.HEADER_HEIGHT}px`);
  debug(`LIST_ITEM_HEIGHT: ${LAYOUT.LIST_ITEM_HEIGHT}px`);
  debug(`FOOTER_HEIGHT: ${LAYOUT.FOOTER_HEIGHT}px`);
  debug(`WINDOW_WIDTH: ${LAYOUT.WINDOW_WIDTH}px`);
  debug(`MAX_VISIBLE_ITEMS: ${LAYOUT.MAX_VISIBLE_ITEMS}`);
  debug(`LIST_PADDING: ${LAYOUT.LIST_PADDING}px`);
  debug('--- Calculated Expected Heights ---');
  debug(`0 items: ${calculateExpectedListHeight(0)}px`);
  debug(`1 item: ${calculateExpectedListHeight(1)}px`);
  debug(`3 items: ${calculateExpectedListHeight(3)}px`);
  debug(`5 items: ${calculateExpectedListHeight(5)}px`);
  debug(`10 items: ${calculateExpectedListHeight(10)}px`);
  debug(`20 items: ${calculateExpectedListHeight(20)}px`);
});

// -----------------------------------------------------------------------------
// Test 10: Visual verification - editor fills window
// -----------------------------------------------------------------------------

await runTest('editor-visual-fill', async () => {
  await ensureScreenshotDir();
  
  const measurePromise = (async () => {
    await wait(200); // Extra time for render
    const bounds = await getWindowBounds();
    const screenshot = await captureScreenshot();
    return { bounds, screenshot };
  })();
  
  const editorPromise = editor(`// Visual test content
// This editor should completely fill the 700px window
// If you see empty space below, the layout is broken!
// Line 1
// Line 2
// Line 3
// Line 4
// Line 5
// Line 6
// Line 7
// Line 8
// Line 9
// Line 10
// Line 11
// Line 12
// Line 13
// Line 14
// Line 15
// Line 16
// Line 17
// Line 18
// Line 19
// Line 20
// Line 21
// Line 22
// Line 23
// Line 24
// Line 25
// Line 26
// Line 27
// Line 28
// Line 29
// Line 30
// End of content`, 'typescript');
  
  const { bounds, screenshot } = await measurePromise;
  
  // Save screenshot
  const screenshotPath = await saveScreenshot(screenshot.data, 'editor-visual-fill');
  debug(`Screenshot: ${screenshotPath}`);
  
  // Check bounds first
  assertHeightInRange(bounds.height, LAYOUT.MAX_HEIGHT, 'editor() height');
  
  // Visual verification
  const analysis = await analyzeContentFill(screenshotPath, LAYOUT.MAX_HEIGHT);
  const report = generateReport('editor-visual-fill', screenshotPath, analysis);
  debug(report);
  
  if (!analysis.pass) {
    throw new Error(`Visual verification failed: ${analysis.message}. Screenshot: ${screenshotPath}`);
  }
  
  submit('// Submitted');
  await editorPromise;
});

// -----------------------------------------------------------------------------
// Test 11: Visual verification - div fills window
// -----------------------------------------------------------------------------

await runTest('div-visual-fill', async () => {
  await ensureScreenshotDir();
  
  const measurePromise = (async () => {
    await wait(200); // Extra time for render
    const bounds = await getWindowBounds();
    const screenshot = await captureScreenshot();
    return { bounds, screenshot };
  })();
  
  const divPromise = div(`
    <div class="p-4">
      <h1 class="text-2xl font-bold mb-4">Visual Fill Test</h1>
      <p class="mb-2">This div should completely fill the 700px window.</p>
      <p class="mb-2">If you see empty space at the bottom, the layout is broken!</p>
      <ul class="list-disc ml-6">
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
        <li>Item 4</li>
        <li>Item 5</li>
        <li>Item 6</li>
        <li>Item 7</li>
        <li>Item 8</li>
        <li>Item 9</li>
        <li>Item 10</li>
      </ul>
      <p class="mt-4">End of content.</p>
    </div>
  `);
  
  const { bounds, screenshot } = await measurePromise;
  
  // Save screenshot
  const screenshotPath = await saveScreenshot(screenshot.data, 'div-visual-fill');
  debug(`Screenshot: ${screenshotPath}`);
  
  // Check bounds first
  assertHeightInRange(bounds.height, LAYOUT.MAX_HEIGHT, 'div() height');
  
  // Visual verification
  const analysis = await analyzeContentFill(screenshotPath, LAYOUT.MAX_HEIGHT);
  const report = generateReport('div-visual-fill', screenshotPath, analysis);
  debug(report);
  
  if (!analysis.pass) {
    throw new Error(`Visual verification failed: ${analysis.message}. Screenshot: ${screenshotPath}`);
  }
  
  submit(null);
  await divPromise;
});

debug('test-window-resize-measurement.ts completed!');
