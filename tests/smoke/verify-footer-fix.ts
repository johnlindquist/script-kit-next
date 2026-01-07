// Name: Verify Footer Fix
// Description: Quick test to verify arg prompt footer is visible

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), 'test-screenshots', 'verify');
mkdirSync(screenshotDir, { recursive: true });

console.error('[VERIFY] Starting footer verification...');

// Show arg prompt with choices
arg('Select a fruit', [
  { name: 'Apple', value: 'apple', description: 'A delicious red fruit' },
  { name: 'Banana', value: 'banana', description: 'Yellow and sweet' },
  { name: 'Cherry', value: 'cherry', description: 'Small and red' },
  { name: 'Date', value: 'date', description: 'Sweet dried fruit' },
]);

// Wait for render and capture
await new Promise(r => setTimeout(r, 800));
const screenshot = await captureScreenshot();
const path = join(screenshotDir, 'arg-prompt-with-footer.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] arg-prompt: ${screenshot.width}x${screenshot.height} -> ${path}`);

process.exit(0);
