// Name: Clipboard Actions Footer Test
// Description: Captures screenshot of clipboard history footer to verify Actions button visibility

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] Clipboard actions footer test starting...');

// Wait for clipboard history view to render
await new Promise(resolve => setTimeout(resolve, 1000));

// Capture screenshot
console.error('[SMOKE] Capturing screenshot...');
const screenshot = await captureScreenshot();
console.error(`[SMOKE] Screenshot: ${screenshot.width}x${screenshot.height}`);

// Save to ./.test-screenshots/
const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const filename = `clipboard-actions-footer-${Date.now()}.png`;
const filepath = join(screenshotDir, filename);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] Saved to: ${filepath}`);
console.error('[SMOKE] Test complete - verify Actions ⌘K appears in the footer');

process.exit(0);
