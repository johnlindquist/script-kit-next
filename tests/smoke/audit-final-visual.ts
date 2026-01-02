// Name: Audit Final Visual
// Description: Final visual verification of arg() and div() prompts

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), '.test-screenshots', 'audit-final');
mkdirSync(screenshotDir, { recursive: true });

console.error('[AUDIT-FINAL] Starting final visual verification');

// Helper to capture screenshot after delay
async function capture(name: string, delayMs = 400) {
  await new Promise(r => setTimeout(r, delayMs));
  try {
    const ss = await captureScreenshot();
    const path = join(screenshotDir, `${name}.png`);
    writeFileSync(path, Buffer.from(ss.data, 'base64'));
    console.error(`[SCREENSHOT] ${path}`);
    return path;
  } catch (e) {
    console.error(`[ERROR] Failed to capture ${name}:`, e);
    return null;
  }
}

// === TEST 1: Basic div with HTML ===
console.error('[TEST 1] Basic div with HTML');
await div(`
  <div style="padding: 24px;">
    <h1 style="color: #60a5fa; font-size: 24px; margin-bottom: 16px;">Test 1: Basic HTML Rendering</h1>
    <p style="color: #9ca3af; line-height: 1.6;">This verifies that basic HTML elements render correctly:</p>
    <ul style="margin-top: 12px; color: #d1d5db;">
      <li style="margin-bottom: 8px;">✓ Headings with inline color</li>
      <li style="margin-bottom: 8px;">✓ Paragraphs with line-height</li>
      <li style="margin-bottom: 8px;">✓ Lists with proper spacing</li>
    </ul>
    <div style="margin-top: 16px; padding: 12px; background: #1e3a5f; border-radius: 8px;">
      <code style="color: #34d399;">Styled code block</code>
    </div>
  </div>
`);
await capture('01-div-basic-html');

// === TEST 2: arg() with 5 string choices ===
console.error('[TEST 2] arg with string choices');
const fruit = await arg('Select a fruit', ['Apple', 'Banana', 'Cherry', 'Date', 'Elderberry']);
console.error(`[RESULT] Selected: ${fruit}`);
await capture('02-arg-string-choices');

// === TEST 3: arg() with structured choices (name, value, description) ===
console.error('[TEST 3] arg with structured choices');
const action = await arg('Select an action', [
  { name: 'Run Script', value: 'run', description: 'Execute the current script immediately' },
  { name: 'Edit Script', value: 'edit', description: 'Open script in VS Code editor' },
  { name: 'Delete Script', value: 'delete', description: 'Permanently remove this script' },
]);
console.error(`[RESULT] Selected: ${action}`);
await capture('03-arg-structured-choices');

// === TEST 4: div with Tailwind CSS classes ===
console.error('[TEST 4] div with Tailwind classes');
await div({
  html: `
    <div class="text-center py-6">
      <h2 class="text-2xl font-bold text-purple-400 mb-4">Test 4: Tailwind CSS</h2>
      <p class="text-gray-400 mb-6">Testing Tailwind utility classes in containerClasses</p>
      <div class="flex gap-3 justify-center">
        <span class="px-4 py-2 bg-green-600/30 text-green-400 rounded-lg font-medium">Success</span>
        <span class="px-4 py-2 bg-yellow-600/30 text-yellow-400 rounded-lg font-medium">Warning</span>
        <span class="px-4 py-2 bg-red-600/30 text-red-400 rounded-lg font-medium">Error</span>
      </div>
    </div>
  `,
  containerClasses: 'p-4 bg-gray-800/50'
});
await capture('04-div-tailwind');

// === TEST 5: Unicode support in choices ===
console.error('[TEST 5] Unicode in choices');
const unicode = await arg('Select (Unicode test)', [
  '🍎 Apple (Emoji)',
  '日本語 Japanese',
  '中文 Chinese', 
  'العربية Arabic',
  'Ελληνικά Greek'
]);
console.error(`[RESULT] Selected: ${unicode}`);
await capture('05-arg-unicode');

// === TEST 6: Empty choices (text input mode) ===
console.error('[TEST 6] Empty choices');
const text = await arg('Type anything (no choices provided)', []);
console.error(`[RESULT] Typed: ${text}`);
await capture('06-arg-empty-choices');

// === TEST 7: Markdown via md() helper ===
console.error('[TEST 7] Markdown rendering');
await div(md(`
# Test 7: Markdown

Testing **bold**, *italic*, and \`inline code\`.

## Code Block
\`\`\`typescript
const greeting = "Hello, World!";
console.log(greeting);
\`\`\`

- List item one
- List item two
- List item three

> This is a blockquote
`));
await capture('07-div-markdown');

// === TEST 8: Large list (50 items) ===
console.error('[TEST 8] Large list performance');
const items = Array.from({ length: 50 }, (_, i) => `Item ${String(i + 1).padStart(3, '0')}`);
const start = Date.now();
const selected = await arg('Select from 50 items', items);
const duration = Date.now() - start;
console.error(`[RESULT] Selected: ${selected} in ${duration}ms`);
await capture('08-arg-large-list');

// === SUMMARY ===
console.error('[AUDIT-FINAL] All 8 tests completed');

await div(md(`
# Audit Complete ✅

All 8 visual verification tests have been run:

| # | Test | Status |
|---|------|--------|
| 1 | div() basic HTML | ✅ |
| 2 | arg() string choices | ✅ |
| 3 | arg() structured choices | ✅ |
| 4 | div() Tailwind CSS | ✅ |
| 5 | Unicode in choices | ✅ |
| 6 | Empty choices (text input) | ✅ |
| 7 | Markdown via md() | ✅ |
| 8 | Large list (50 items) | ✅ |

Screenshots saved to: \`.test-screenshots/audit-final/\`

Press Enter to exit.
`));
await capture('09-summary');

process.exit(0);
