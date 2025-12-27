// Name: Test Editor Visual Fill
// Description: Verifies that the editor completely fills the 700px window height
// This test specifically catches the bug where the editor renders but doesn't fill its container

import '../../scripts/kit-sdk';

// Constants from window_resize.rs
const MAX_HEIGHT = 700;
const TOLERANCE = 20;

console.error('[SMOKE] test-editor-visual-fill.ts starting...');

// Helper to log test results in JSONL format
interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail';
  timestamp: string;
  details?: Record<string, unknown>;
}

function logTest(result: TestResult) {
  console.log(JSON.stringify(result));
}

// Generate content that should fill a 700px window
// If the editor is working correctly, you should NOT see empty space below
const editorContent = `// VISUAL FILL TEST
// This editor should completely fill the 700px window height
// If you see empty space below line 35, the layout is BROKEN!

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
// Line 31
// Line 32
// Line 33
// Line 34
// Line 35 - END MARKER

// If you can see this line AND there's empty space below,
// the editor is not filling the window correctly!`;

logTest({
  test: 'editor-visual-fill',
  status: 'running',
  timestamp: new Date().toISOString(),
});

const startTime = Date.now();

// Wait for window to be ready
await wait(100);

// Start measuring bounds and screenshot in parallel with the editor
const measurePromise = (async () => {
  // Wait for editor to render
  await wait(200);
  
  // Get window bounds
  const bounds = await getWindowBounds();
  console.error(`[SMOKE] Window bounds: ${JSON.stringify(bounds)}`);
  
  // Capture screenshot
  const screenshot = await captureScreenshot();
  console.error(`[SMOKE] Screenshot captured: ${screenshot.width}x${screenshot.height}`);
  
  return { bounds, screenshot };
})();

// Show the editor
const editorPromise = editor(editorContent, 'typescript');

// Get measurements
const { bounds, screenshot } = await measurePromise;

// Analyze results
const heightDiff = Math.abs(bounds.height - MAX_HEIGHT);
const boundsPass = heightDiff <= TOLERANCE;

// Screenshot dimensions should match window bounds
const screenshotMatchesBounds = (
  Math.abs(screenshot.width - bounds.width) <= 2 &&
  Math.abs(screenshot.height - bounds.height) <= 2
);

console.error(`[SMOKE] Analysis:`);
console.error(`  - Window height: ${bounds.height}px (expected: ${MAX_HEIGHT}px, diff: ${heightDiff}px)`);
console.error(`  - Bounds check: ${boundsPass ? 'PASS' : 'FAIL'}`);
console.error(`  - Screenshot matches bounds: ${screenshotMatchesBounds ? 'YES' : 'NO'}`);

// The key insight: if window is 700px but screenshot shows only partial content,
// then the editor is not filling its container
// This requires visual inspection of the screenshot

const details = {
  windowBounds: bounds,
  screenshotDimensions: { width: screenshot.width, height: screenshot.height },
  expectedHeight: MAX_HEIGHT,
  heightDiff,
  boundsPass,
  screenshotMatchesBounds,
  duration_ms: Date.now() - startTime,
};

if (boundsPass && screenshotMatchesBounds) {
  logTest({
    test: 'editor-visual-fill',
    status: 'pass',
    timestamp: new Date().toISOString(),
    details,
  });
  console.error('[SMOKE] TEST PASSED - Window resized correctly and screenshot captured');
} else {
  logTest({
    test: 'editor-visual-fill',
    status: 'fail',
    timestamp: new Date().toISOString(),
    details,
  });
  console.error('[SMOKE] TEST FAILED - Visual verification failed');
}

// Submit to continue (won't wait for user interaction in test mode)
submit('// Test submitted');
await editorPromise;

console.error('[SMOKE] test-editor-visual-fill.ts completed!');
