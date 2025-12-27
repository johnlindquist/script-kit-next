// Name: Actions Panel Visual Test
// Description: Opens the app, triggers Cmd+K to show actions panel, captures screenshot

/**
 * VISUAL TEST: test-actions-visual.ts
 * 
 * This test captures a screenshot of the actions panel for visual verification.
 * The actions panel is triggered via Cmd+K keyboard shortcut.
 * 
 * USAGE (Automated with visual-test.sh):
 *   ./scripts/visual-test.sh tests/smoke/test-actions-visual.ts 5
 *   # When the window appears, press Cmd+K to show actions panel
 *   # Screenshot is captured after 5 seconds
 * 
 * USAGE (Manual):
 *   cargo build && \
 *   echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-actions-visual.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 *   # Then press Cmd+K in the app window to show actions panel
 * 
 * EXPECTED BEHAVIOR:
 *   1. App shows arg prompt with test choices (simulating script list)
 *   2. User/tester presses Cmd+K to show the actions panel overlay
 *   3. Screenshot is captured showing the actions panel
 *   4. Test validates screenshot was captured
 * 
 * NOTE: Currently keyboard.tap() is not processed by the app, so the
 * actions panel must be triggered manually during the test window.
 * Once keyboard message handling is implemented in main.rs, this test
 * will become fully automated.
 * 
 * WHAT TO VERIFY IN SCREENSHOT:
 *   - Actions panel appears as a centered overlay
 *   - Panel has proper padding and rounded corners
 *   - Action items are properly spaced and styled
 *   - Keyboard shortcuts are right-aligned
 *   - Search input is visible at the top
 *   - Background dimming effect is visible
 */

import '../../scripts/kit-sdk';

// Constants
const RENDER_DELAY_MS = 2000; // Longer wait to give time for manual Cmd+K press

console.error('[ACTIONS-VISUAL] Starting actions panel visual test...');
console.error('[ACTIONS-VISUAL] *** PRESS Cmd+K to show the actions panel ***');

// Run the visual test
async function runActionsVisualTest() {
  console.error('[ACTIONS-VISUAL] Step 1: Showing arg prompt with choices...');
  
  // Create test choices that mimic the script list
  // This gives context for testing the actions panel
  const choices = [
    { name: 'Test Script 1', value: 'test-1', description: 'A test script for visual testing' },
    { name: 'Test Script 2', value: 'test-2', description: 'Another test script' },
    { name: 'Hello World', value: 'hello', description: 'Classic hello world script' },
    { name: 'Generate Report', value: 'report', description: 'Generates a report' },
    { name: 'Send Email', value: 'email', description: 'Send an email notification' },
  ];
  
  // Start the arg prompt - this creates and shows the window
  // We don't await it because we're testing the actions overlay
  const argPromise = arg('Press Cmd+K to show actions panel:', choices);
  
  // Give the window time to render
  console.error('[ACTIONS-VISUAL] Step 2: Window rendering, waiting for Cmd+K...');
  await wait(300);
  
  // Attempt to trigger Cmd+K programmatically
  // NOTE: Currently keyboard messages aren't processed by the app
  // This is a placeholder for when that feature is implemented
  console.error('[ACTIONS-VISUAL] Step 3: Sending Cmd+K (may require manual trigger)...');
  try {
    await keyboard.tap('command', 'k');
  } catch (e) {
    console.error('[ACTIONS-VISUAL] keyboard.tap not available');
  }
  
  // Wait for user to manually press Cmd+K and for panel to render
  console.error('[ACTIONS-VISUAL] Step 4: Waiting for actions panel...');
  console.error('[ACTIONS-VISUAL] *** If actions panel not visible, press Cmd+K NOW ***');
  await wait(RENDER_DELAY_MS);
  
  // Capture the screenshot
  console.error('[ACTIONS-VISUAL] Step 5: Capturing screenshot...');
  try {
    const screenshot = await captureScreenshot();
    console.error(`[ACTIONS-VISUAL] Screenshot captured: ${screenshot.width}x${screenshot.height}`);
    
    // Determine if this is likely the app window or the whole screen
    // App window is typically around 750x500, full screen is much larger
    const isAppWindow = screenshot.width < 2000 && screenshot.height < 1500;
    
    // Log the result
    console.error('[ACTIONS-VISUAL] SUCCESS: Screenshot captured');
    console.error(`[ACTIONS-VISUAL] Screenshot size: ${Math.round(screenshot.data.length / 1024)}KB`);
    console.error(`[ACTIONS-VISUAL] Capture type: ${isAppWindow ? 'App window' : 'Full display (manual verification needed)'}`);
    
    // Output JSON result for programmatic parsing
    console.log(JSON.stringify({
      test: 'actions-visual',
      status: 'pass',
      screenshot: {
        width: screenshot.width,
        height: screenshot.height,
        sizeKB: Math.round(screenshot.data.length / 1024),
        captureType: isAppWindow ? 'app_window' : 'full_display',
      },
      note: 'Verify actions panel is visible in screenshot. If not, re-run with manual Cmd+K.',
      timestamp: new Date().toISOString(),
    }));
  } catch (err) {
    console.error(`[ACTIONS-VISUAL] FAIL: Screenshot capture failed: ${err}`);
    console.log(JSON.stringify({
      test: 'actions-visual',
      status: 'fail',
      error: String(err),
      timestamp: new Date().toISOString(),
    }));
  }
  
  // Clean up - close any open dialogs
  console.error('[ACTIONS-VISUAL] Step 6: Cleaning up...');
  
  // Race between the arg promise completing (via escape or selection) and timeout
  // This prevents hanging if the user doesn't interact
  await Promise.race([
    argPromise.catch(() => {}), // Ignore errors from cancellation
    wait(1000), // Timeout after 1 second
  ]);
  
  console.error('[ACTIONS-VISUAL] Test complete.');
  console.error('[ACTIONS-VISUAL] Review screenshot to verify actions panel styling.');
}

// Run the test
runActionsVisualTest().catch((err) => {
  console.error(`[ACTIONS-VISUAL] FATAL: ${err}`);
  console.log(JSON.stringify({
    test: 'actions-visual',
    status: 'fail',
    error: String(err),
    timestamp: new Date().toISOString(),
  }));
});
