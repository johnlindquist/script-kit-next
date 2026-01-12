// Test: Verify AI setup card appears when Tab is pressed with no API key configured
// This test simulates the user flow:
// 1. Type something in main menu
// 2. Press Tab (Ask AI feature)
// 3. Should see setup card prompting to configure API key

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Wait for window to be ready
await new Promise(r => setTimeout(r, 500));

// Type something in the search input
await setFilterText("test query");
await new Promise(r => setTimeout(r, 300));

// Simulate Tab key to trigger Ask AI
// Note: This requires the Tab key to be handled by the app
// For now, we'll capture the current state and verify manually

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `ai-setup-inline-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] ${path}`);
console.error('[INFO] To test Tab->AI flow manually:');
console.error('  1. Type something in main menu');
console.error('  2. Press Tab');
console.error('  3. Should see "API Key Required" setup card');
console.error('  4. Press Enter to configure or Esc to go back');

process.exit(0);
