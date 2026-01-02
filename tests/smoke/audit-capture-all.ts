// Name: Audit Capture All
// Description: Captures screenshots of different prompt types for visual verification

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), '.test-screenshots', 'audit');
mkdirSync(screenshotDir, { recursive: true });

async function capture(name: string) {
  await new Promise(r => setTimeout(r, 400));
  const ss = await captureScreenshot();
  const path = join(screenshotDir, `${name}.png`);
  writeFileSync(path, Buffer.from(ss.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

// Test 1: div with basic HTML
console.error('[TEST 1] div-basic-html');
div(`
  <div style="padding: 20px; font-family: system-ui;">
    <h1 style="color: #3b82f6; margin-bottom: 12px;">Test 1: Basic HTML</h1>
    <p style="color: #9ca3af;">Testing basic HTML rendering</p>
    <ul style="margin-top: 8px; color: #d1d5db;">
      <li>List item one</li>
      <li>List item two</li>
      <li>List item three</li>
    </ul>
  </div>
`);
await capture('01-div-basic-html');

// Test 2: div with Tailwind classes
console.error('[TEST 2] div-tailwind');
div({
  html: `
    <div class="text-center">
      <h1 class="text-2xl font-bold text-blue-400">Test 2: Tailwind CSS</h1>
      <p class="mt-2 text-gray-400">Using Tailwind utility classes</p>
      <div class="mt-4 flex gap-2 justify-center">
        <span class="px-3 py-1 bg-green-500/20 text-green-400 rounded">Tag 1</span>
        <span class="px-3 py-1 bg-blue-500/20 text-blue-400 rounded">Tag 2</span>
        <span class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded">Tag 3</span>
      </div>
    </div>
  `,
  containerClasses: 'p-6'
});
await capture('02-div-tailwind');

// Test 3: arg with string choices
console.error('[TEST 3] arg-string-choices');
arg('Test 3: Select a fruit', [
  'Apple',
  'Banana',
  'Cherry',
  'Date',
  'Elderberry'
]);
await capture('03-arg-string-choices');

// Test 4: arg with structured choices
console.error('[TEST 4] arg-structured-choices');
arg('Test 4: Select an action', [
  { name: 'Run Script', value: 'run', description: 'Execute the current script' },
  { name: 'Edit Script', value: 'edit', description: 'Open script in editor' },
  { name: 'Delete Script', value: 'delete', description: 'Remove script from disk' },
  { name: 'Share Script', value: 'share', description: 'Copy shareable link' }
]);
await capture('04-arg-structured-choices');

// Test 5: Unicode in choices
console.error('[TEST 5] unicode-choices');
arg('Test 5: Unicode Support', [
  '🍎 Apple (emoji)',
  '日本語 Japanese',
  'العربية Arabic (RTL)',
  '中文 Chinese',
  'Ελληνικά Greek'
]);
await capture('05-unicode-choices');

// Test 6: Large list performance
console.error('[TEST 6] large-list');
const largeChoices = Array.from({ length: 50 }, (_, i) => `Item ${String(i + 1).padStart(3, '0')}`);
arg('Test 6: Large List (50 items)', largeChoices);
await capture('06-large-list');

// Test 7: Empty choices (text input mode)
console.error('[TEST 7] empty-choices');
arg('Test 7: Type anything (no choices)', []);
await capture('07-empty-choices');

// Test 8: div with markdown
console.error('[TEST 8] div-markdown');
div(md(`
# Test 8: Markdown Rendering

This tests **bold**, *italic*, and \`inline code\`.

## Features
- List item with bullet
- Another item
- Third item

> A blockquote for testing

\`\`\`typescript
const hello = "world";
console.log(hello);
\`\`\`
`));
await capture('08-div-markdown');

// Summary
console.error('[AUDIT] All 8 visual tests captured');
console.error('[AUDIT] Screenshots saved to:', screenshotDir);

div(md(`
# Visual Audit Complete

**8 screenshots captured** in \`.test-screenshots/audit/\`

1. div-basic-html
2. div-tailwind  
3. arg-string-choices
4. arg-structured-choices
5. unicode-choices
6. large-list
7. empty-choices
8. div-markdown

Press Enter to exit and view screenshots.
`));

await new Promise(r => setTimeout(r, 500));
await capture('09-summary');

process.exit(0);
