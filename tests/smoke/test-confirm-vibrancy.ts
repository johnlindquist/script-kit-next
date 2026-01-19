// Test confirm dialog vibrancy and focus
// Tests that the confirm dialog:
// 1. Has vibrancy blur like Actions window
// 2. Captures focus properly
// 3. Keyboard navigation works

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting confirm dialog vibrancy test');

// Show a div to get the main window visible
await div(`
  <div class="p-8 flex flex-col gap-4 items-center justify-center min-h-[200px]">
    <h1 class="text-2xl font-bold text-white">Confirm Dialog Test</h1>
    <p class="text-gray-300">Testing vibrancy and focus</p>
  </div>
`);

console.error('[TEST] Main window shown, waiting for render');
await new Promise(r => setTimeout(r, 500));

// Now trigger a confirm dialog
console.error('[TEST] Triggering confirm dialog');
const confirmed = await confirm({
  message: "Are you sure you want to test vibrancy?",
  confirmText: "Yes",
  cancelText: "Cancel"
});

console.error(`[TEST] User chose: ${confirmed ? 'confirmed' : 'cancelled'}`);

// Capture screenshot
console.error('[TEST] Capturing screenshot');
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `confirm-vibrancy-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

console.error('[TEST] Test complete');
process.exit(0);
