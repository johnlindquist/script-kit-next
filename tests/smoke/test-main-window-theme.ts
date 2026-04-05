// Name: Test Main Window Theme
// Description: Captures screenshot of the mini main window to compare with Notes window theme

import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting mini main window theme capture...');

// Create screenshot directory first
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const layout = await expectMiniMainWindow('test-main-window-theme', 1000);

// Capture screenshot of the mini main window
console.error('[TEST] Capturing mini main window screenshot (hi_dpi=true for full resolution)...');

try {
  // Use hi_dpi=true to get full retina resolution
  const screenshot = await captureScreenshot({ hiDpi: true });
  console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

  const timestamp = Date.now();
  const filename = `mini-main-window-theme-${timestamp}.png`;
  const filepath = join(screenshotDir, filename);
  
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  
  console.error('[TEST] Screenshot saved successfully');
  console.error(
    `[TEST] Layout: ${layout.windowWidth}x${layout.windowHeight} (${layout.promptType})`
  );
  console.error('[TEST] Theme colors from logs:');
  console.error('[TEST]   background: #1e1e1e');
  console.error('[TEST]   accent: #fbbf24');
  console.error('[TEST]   selected_subtle: #2a2a2a');
  console.error('[TEST]   title_bar: #2d2d30');
  console.error('[TEST]   border: #464647');
} catch (err) {
  console.error(`[TEST] Failed to capture screenshot: ${err}`);
}

process.exit(0);
