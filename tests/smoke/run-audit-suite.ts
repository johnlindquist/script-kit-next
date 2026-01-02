// Name: Run Audit Suite
// Description: Tests each audit script and captures screenshots

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), '.test-screenshots', 'audit');
mkdirSync(screenshotDir, { recursive: true });

console.error('[AUDIT-SUITE] Starting audit test suite');
console.error('[AUDIT-SUITE] Screenshot dir:', screenshotDir);

// Test 1: Basic div() rendering
console.error('[TEST] div-basic: Testing basic HTML rendering');
await div(`
  <div style="padding: 20px;">
    <h1 style="color: #3b82f6;">Audit Test: div() Basic</h1>
    <p>Testing basic HTML rendering with inline styles.</p>
    <ul>
      <li>Heading renders</li>
      <li>Paragraph renders</li>
      <li>List renders</li>
    </ul>
  </div>
`);

// Capture screenshot
await new Promise(r => setTimeout(r, 300));
try {
  const ss1 = await captureScreenshot();
  writeFileSync(join(screenshotDir, 'div-basic.png'), Buffer.from(ss1.data, 'base64'));
  console.error('[SCREENSHOT] div-basic.png captured');
} catch (e) {
  console.error('[ERROR] Failed to capture div-basic:', e);
}

// Test 2: arg() with choices
console.error('[TEST] arg-choices: Testing arg with string choices');
const choice = await arg('Select a test fruit', [
  'Apple',
  'Banana', 
  'Cherry',
  'Date',
  'Elderberry'
]);
console.error(`[RESULT] arg-choices: Selected "${choice}"`);

// Test 3: arg() with structured choices
console.error('[TEST] arg-structured: Testing arg with structured choices');
const structured = await arg('Select an action', [
  { name: 'Run Script', value: 'run', description: 'Execute the script' },
  { name: 'Edit Script', value: 'edit', description: 'Open in editor' },
  { name: 'Delete Script', value: 'delete', description: 'Remove from disk' }
]);
console.error(`[RESULT] arg-structured: Selected "${structured}"`);

// Test 4: div() with markdown
console.error('[TEST] div-markdown: Testing markdown rendering');
await div(md(`
# Markdown Test

This tests **bold**, *italic*, and \`code\`.

- List item 1
- List item 2
- List item 3

> A blockquote for testing

\`\`\`javascript
const hello = "world";
console.log(hello);
\`\`\`
`));

// Test 5: div() with containerClasses (Tailwind)
console.error('[TEST] div-tailwind: Testing containerClasses');
await div({
  html: `
    <div class="text-center">
      <h2 class="text-2xl font-bold text-blue-500">Tailwind Test</h2>
      <p class="mt-4 text-gray-400">Testing containerClasses with Tailwind</p>
      <div class="mt-4 p-4 bg-green-500/20 rounded-lg">
        <span class="text-green-400">Success indicator</span>
      </div>
    </div>
  `,
  containerClasses: 'p-8 bg-gray-900'
});

// Test 6: Empty choices (edge case)
console.error('[TEST] arg-empty: Testing arg with empty choices');
const emptyResult = await arg('Type anything (no choices)', []);
console.error(`[RESULT] arg-empty: Typed "${emptyResult}"`);

// Test 7: Unicode in choices
console.error('[TEST] unicode: Testing unicode in choices');
const unicodeChoice = await arg('Select emoji', [
  '🍎 Apple',
  '🍌 Banana',
  '🍒 Cherry',
  '日本語 Japanese',
  'العربية Arabic'
]);
console.error(`[RESULT] unicode: Selected "${unicodeChoice}"`);

// Test 8: Large choices list
console.error('[TEST] large-list: Testing 100 item list');
const largeChoices = Array.from({ length: 100 }, (_, i) => `Item ${i + 1}`);
const start = Date.now();
const largeChoice = await arg('Select from 100 items', largeChoices);
const duration = Date.now() - start;
console.error(`[RESULT] large-list: Selected "${largeChoice}" in ${duration}ms`);

// Final summary
console.error('[AUDIT-SUITE] All tests completed');
await div(md(`
# Audit Suite Complete

All interactive tests have been run. Check the console output for results.

## Tests Run:
1. ✅ div-basic - HTML rendering
2. ✅ arg-choices - String choices
3. ✅ arg-structured - Structured choices  
4. ✅ div-markdown - Markdown rendering
5. ✅ div-tailwind - Tailwind classes
6. ✅ arg-empty - Empty choices
7. ✅ unicode - Unicode support
8. ✅ large-list - 100 items

Press Enter to exit.
`));

process.exit(0);
