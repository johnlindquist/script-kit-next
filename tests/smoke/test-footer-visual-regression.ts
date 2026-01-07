// Name: Footer Visual Regression Test
// Description: Captures screenshots of key views to verify unified footer appears consistently

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Configuration
const screenshotDir = join(process.cwd(), 'test-screenshots', 'after');
mkdirSync(screenshotDir, { recursive: true });

console.error('[FOOTER-TEST] Starting footer visual regression tests...');
console.error(`[FOOTER-TEST] Output directory: ${screenshotDir}`);

// Helper to capture screenshots
async function capture(name: string, waitMs: number = 600): Promise<string> {
  await new Promise(r => setTimeout(r, waitMs));
  const screenshot = await captureScreenshot();
  const path = join(screenshotDir, `${name}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${screenshot.width}x${screenshot.height} -> ${path}`);
  return path;
}

// =============================================================================
// 1. Main Menu (default view with footer)
// =============================================================================
console.error('[FOOTER-TEST] 1/4: Main Menu');
await capture('01-main-menu-footer', 800);

// =============================================================================
// 2. Arg Prompt with Choices (footer should be visible)
// =============================================================================
console.error('[FOOTER-TEST] 2/4: Arg Prompt with Choices');
arg('Select a fruit', [
  { name: 'Apple', value: 'apple', description: 'A delicious red fruit' },
  { name: 'Banana', value: 'banana', description: 'Yellow and sweet' },
  { name: 'Cherry', value: 'cherry', description: 'Small and red' },
  { name: 'Date', value: 'date', description: 'Sweet dried fruit' },
]);
await capture('02-arg-choices-footer');

// =============================================================================
// 3. Div Prompt with HTML (footer should be visible)
// =============================================================================
console.error('[FOOTER-TEST] 3/4: Div Prompt');
div(`
  <div class="p-6 space-y-4">
    <h1 class="text-2xl font-bold text-blue-400">Footer Verification</h1>
    <p class="text-gray-300">This div prompt should show the unified footer at the bottom.</p>
    <div class="flex gap-2 flex-wrap">
      <span class="px-3 py-1 bg-green-500/20 text-green-400 rounded">Footer</span>
      <span class="px-3 py-1 bg-blue-500/20 text-blue-400 rounded">Logo</span>
      <span class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded">Actions</span>
    </div>
    <div class="mt-4 p-4 bg-gray-800/50 rounded border border-gray-700">
      <p class="text-gray-400 text-sm">Expected: 40px footer with Script Kit logo, Run button, Actions button (⌘K)</p>
    </div>
  </div>
`);
await capture('03-div-prompt-footer');

// =============================================================================
// 4. Editor Prompt (footer should be visible)
// =============================================================================
console.error('[FOOTER-TEST] 4/4: Editor Prompt');
editor(`// Footer Verification Test
// The editor view should also have the unified footer

function verifyFooter() {
  // Expected footer elements:
  // - Script Kit logo on the left
  // - Primary action button (Run/Submit)
  // - Actions button with ⌘K hint
  return true;
}

// Footer height should be 40px
const FOOTER_HEIGHT = 40;

console.log("Footer test complete!");
`, 'typescript');
await capture('04-editor-prompt-footer', 800);

// =============================================================================
// Summary
// =============================================================================
console.error('[FOOTER-TEST] All screenshots captured!');
console.error('[FOOTER-TEST] Screenshots saved to: ' + screenshotDir);
console.error('[FOOTER-TEST] Verification checklist:');
console.error('[FOOTER-TEST]   [ ] Footer appears at bottom (40px height)');
console.error('[FOOTER-TEST]   [ ] Script Kit logo visible on left');
console.error('[FOOTER-TEST]   [ ] Primary action button visible');
console.error('[FOOTER-TEST]   [ ] Actions button visible with cmd+K hint');

process.exit(0);
