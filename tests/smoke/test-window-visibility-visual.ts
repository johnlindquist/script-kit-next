import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';

/**
 * Visual Test: Window Visibility After Script Exit
 *
 * This test captures screenshots to visually verify window state.
 * Screenshots are saved to .test-screenshots/ for inspection.
 *
 * Expected: After hide() + exit(), main menu should be visible again.
 */

const test = 'window-visibility-visual';
const screenshotDir = join(process.cwd(), '.test-screenshots');

function log(status: string, extra: any = {}) {
  console.error(JSON.stringify({
    test,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  }));
}

// Ensure screenshot directory exists
if (!existsSync(screenshotDir)) {
  mkdirSync(screenshotDir, { recursive: true });
}

log('running');
const start = Date.now();

try {
  // Step 1: Capture initial state (main menu visible)
  log('capturing_initial_state');
  const initialShot = await captureScreenshot();
  const initialPath = join(screenshotDir, `visibility-1-initial-${Date.now()}.png`);
  writeFileSync(initialPath, Buffer.from(initialShot.data, 'base64'));
  log('captured_initial', { path: initialPath });

  // Step 2: Hide the window
  log('hiding_window');
  await hide();
  await new Promise(r => setTimeout(r, 200));

  // Step 3: Try to capture while hidden (may fail or show empty)
  log('capturing_hidden_state');
  try {
    const hiddenShot = await captureScreenshot();
    const hiddenPath = join(screenshotDir, `visibility-2-hidden-${Date.now()}.png`);
    writeFileSync(hiddenPath, Buffer.from(hiddenShot.data, 'base64'));
    log('captured_hidden', { path: hiddenPath });
  } catch (e: any) {
    log('hidden_capture_expected_fail', { error: e.message });
  }

  // Step 4: Show HUD (simulating scriptlet behavior)
  log('showing_hud');
  await hud('Testing window visibility');
  await new Promise(r => setTimeout(r, 300));

  // Step 5: Exit will be called - window should come back
  // Note: Can't capture "after exit" from within the script
  // The verification happens in the logs

  log('pass', {
    result: 'Visual test completed - check screenshots and logs',
    initial_screenshot: initialPath,
    duration_ms: Date.now() - start,
    note: 'After this script exits, main menu should be visible. Check VISIBILITY logs.'
  });

} catch (e: any) {
  log('fail', { error: e.message, duration_ms: Date.now() - start });
}

exit(0);
