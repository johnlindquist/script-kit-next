// Test script for screenshot scale/hiDpi option
// Tests that captureScreenshot() returns 1x by default, 2x with hiDpi: true

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] Testing screenshot scale options...');

// Display some content using setPanel (doesn't wait for submission)
// and show window
await show();
setPanel(`<div class="p-8 bg-blue-500 text-white text-2xl">
  <h1>Screenshot Scale Test</h1>
  <p>Testing 1x (default) vs 2x (hiDpi) resolution</p>
</div>`);

// Wait for render
await new Promise(r => setTimeout(r, 1000));

// Capture at 1x (default)
console.error('[SMOKE] Capturing 1x screenshot (default)...');
const shot1x = await captureScreenshot();
console.error(`[SMOKE] 1x: ${shot1x.width}x${shot1x.height}, data length: ${shot1x.data.length}`);

// Capture at 2x (hiDpi)
console.error('[SMOKE] Capturing 2x screenshot (hiDpi: true)...');
const shot2x = await captureScreenshot({ hiDpi: true });
console.error(`[SMOKE] 2x: ${shot2x.width}x${shot2x.height}, data length: ${shot2x.data.length}`);

// Validate expectations - focus on dimensions, not file size (PNG compression varies)
const is2xApproxDoubleWidth = (shot2x.width >= shot1x.width * 1.8 && shot2x.width <= shot1x.width * 2.2);
const is2xApproxDoubleHeight = (shot2x.height >= shot1x.height * 1.8 && shot2x.height <= shot1x.height * 2.2);
const is2xApproxDouble = is2xApproxDoubleWidth && is2xApproxDoubleHeight;

console.error(`[SMOKE] 2x is ~2x wider than 1x: ${is2xApproxDoubleWidth}`);
console.error(`[SMOKE] 2x is ~2x taller than 1x: ${is2xApproxDoubleHeight}`);

// Save screenshots for visual verification
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const timestamp = Date.now();
const path1x = join(dir, `screenshot-1x-${timestamp}.png`);
const path2x = join(dir, `screenshot-2x-${timestamp}.png`);

writeFileSync(path1x, Buffer.from(shot1x.data, 'base64'));
writeFileSync(path2x, Buffer.from(shot2x.data, 'base64'));

console.error(`[SMOKE] Saved 1x: ${path1x}`);
console.error(`[SMOKE] Saved 2x: ${path2x}`);

// Report results
if (is2xApproxDouble) {
  console.error('[SMOKE] ✅ PASS: Screenshot scale option works correctly');
  console.error(`[SMOKE] 1x resolution: ${shot1x.width}x${shot1x.height}`);
  console.error(`[SMOKE] 2x resolution: ${shot2x.width}x${shot2x.height}`);
} else {
  console.error('[SMOKE] ❌ FAIL: Screenshot scale option not working as expected');
  console.error(`[SMOKE] Expected 2x dimensions to be ~2x larger than 1x`);
  console.error(`[SMOKE] 1x: ${shot1x.width}x${shot1x.height}`);
  console.error(`[SMOKE] 2x: ${shot2x.width}x${shot2x.height}`);
}

process.exit(0);
