import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting xcap screenshot test...');

// Display something (don't await - the div promise waits for submit which never comes)
const divPromise = div(`<div class="p-8 bg-blue-500 text-white text-2xl">Screenshot Test</div>`);

// Wait for render
await new Promise(r => setTimeout(r, 500));

// Capture screenshot  
console.error(`[TEST] About to call captureScreenshot...`);
const screenshot = await captureScreenshot();
console.error(`[TEST] Screenshot captured: ${screenshot.width}x${screenshot.height}`);

// Verify dimensions are reasonable (app window, not full desktop)
// App window is typically 750x500 at 1x or ~1500x1000 at 2x retina
const isAppWindow = screenshot.width < 2000 && screenshot.height < 1500;
console.error(`[TEST] Is app window (not desktop): ${isAppWindow}`);
console.error(`[TEST] Expected: ~750x500 @1x or ~1500x1000 @2x, not 3024x1964 (desktop)`);

// Save it
const dir = join(process.cwd(), '.mocks/test');
mkdirSync(dir, { recursive: true });
const path = join(dir, 'xcap-test.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[TEST] Saved to: ${path}`);

if (isAppWindow && screenshot.width > 0 && screenshot.height > 0) {
  console.error('[TEST] SUCCESS: Screenshot captured app window only');
} else if (screenshot.width >= 3024) {
  console.error('[TEST] FAIL: Screenshot captured desktop, not app window');
} else {
  console.error('[TEST] PARTIAL: Screenshot captured but dimensions unexpected');
}

process.exit(0);
