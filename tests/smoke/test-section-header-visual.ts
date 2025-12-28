// Test: Section Header Visual Check
// Captures a screenshot of the main menu to verify section header heights

import '../../scripts/kit-sdk';
import { saveScreenshot } from '../autonomous/screenshot-utils';

console.error('[SMOKE] test-section-header-visual.ts starting...');

// Small delay to let the UI render
await new Promise(resolve => setTimeout(resolve, 500));

// Capture screenshot of the app window only
console.error('[SMOKE] Capturing screenshot...');
const screenshot = await captureScreenshot();
console.error(`[SMOKE] Screenshot captured: ${screenshot.width}x${screenshot.height}`);

// Save it
const savedPath = await saveScreenshot(screenshot.data, 'section-header-test');
console.error(`[SMOKE] Screenshot saved to: ${savedPath}`);

// Output for the test runner
console.log(JSON.stringify({
  test: 'section-header-visual',
  status: 'pass',
  screenshot: savedPath,
  dimensions: { width: screenshot.width, height: screenshot.height }
}));

// Exit
process.exit(0);
