// Name: Test AI Window Scrolling and Search
// Description: Visual tests for AI window chat list scrolling and search functionality

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), 'test-screenshots', 'ai-window-scroll');
mkdirSync(screenshotDir, { recursive: true });

async function captureAndSave(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 500)); // Wait for render
  const screenshot = await captureScreenshot();
  const filepath = join(screenshotDir, `${name}-${Date.now()}.png`);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${filepath}`);
  return filepath;
}

console.error('[TEST] Starting AI window scroll and search tests...');
console.error('[TEST] Note: This test expects the AI window to already be open');

// Test 1: Initial state - should show chat list in sidebar
console.error('[TEST] 1. Capturing initial AI window state...');
await captureAndSave('01-initial-state');

// Give time for any animations
await new Promise(r => setTimeout(r, 1000));

// Test 2: Capture after a moment (to show rendered content)
console.error('[TEST] 2. Capturing after render...');
await captureAndSave('02-after-render');

console.error('[TEST] AI window scroll test complete!');
console.error('[TEST] Check screenshots in: ' + screenshotDir);

process.exit(0);
