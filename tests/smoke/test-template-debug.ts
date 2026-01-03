// Name: Template Debug Test
// Description: Visual debug test for template tabstop navigation

/**
 * This test helps debug template/snippet Tab navigation.
 * 
 * Expected behavior:
 * 1. Editor opens with "Hello name, welcome to place!"
 * 2. "name" should be SELECTED (highlighted)
 * 3. Pressing Tab should move selection to "place"
 * 4. Pressing Shift+Tab should move back to "name"
 * 
 * Run with:
 * cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-template-debug.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 */

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Template Debug Test",
  description: "Visual debug for template tabstop navigation"
};

console.error('[DEBUG] Starting template debug test...');
console.error('[DEBUG] template function available:', typeof template);

// Give the app a moment to fully initialize
await new Promise(r => setTimeout(r, 500));

console.error('[DEBUG] Calling template() with multi-tabstop template...');
console.error('[DEBUG] Template: "Hello ${1:name}, welcome to ${2:place}!"');
console.error('[DEBUG] Expected: Editor shows "Hello name, welcome to place!" with "name" selected');
console.error('[DEBUG] Expected: Tab moves to "place", Shift+Tab moves back to "name"');

// Call template - this should open editor with tabstops
const result = await template('Hello ${1:name}, welcome to ${2:place}!', { 
  language: 'plaintext' 
});

console.error('[DEBUG] Template result:', result);

// Capture screenshot for visual verification
try {
  const screenshot = await captureScreenshot();
  console.error(`[DEBUG] Screenshot captured: ${screenshot.width}x${screenshot.height}`);
  
  const dir = join(process.cwd(), 'test-screenshots');
  mkdirSync(dir, { recursive: true });
  const filepath = join(dir, `template-debug-${Date.now()}.png`);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
} catch (e) {
  console.error('[DEBUG] Screenshot failed:', e);
}

console.error('[DEBUG] Test complete');
