// Name: Audit Visual Single
// Description: Test a single prompt type with screenshot capture

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), '.test-screenshots', 'audit');
mkdirSync(screenshotDir, { recursive: true });

// Get test type from env or default
const testType = process.env.AUDIT_TEST || 'div-basic';

console.error(`[AUDIT] Running test: ${testType}`);

async function captureAndExit(name: string) {
  await new Promise(r => setTimeout(r, 500));
  const ss = await captureScreenshot();
  const path = join(screenshotDir, `${name}.png`);
  writeFileSync(path, Buffer.from(ss.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  process.exit(0);
}

switch (testType) {
  case 'div-basic':
    await div(`
      <div style="padding: 24px; font-family: system-ui;">
        <h1 style="color: #3b82f6; margin-bottom: 16px; font-size: 28px;">Basic HTML Test</h1>
        <p style="color: #9ca3af; margin-bottom: 12px;">Testing basic HTML rendering with inline styles.</p>
        <ul style="color: #d1d5db; line-height: 1.8;">
          <li>Heading renders correctly</li>
          <li>Paragraph with proper spacing</li>
          <li>Unordered list displays</li>
        </ul>
        <div style="margin-top: 16px; padding: 12px; background: #1f2937; border-radius: 8px;">
          <code style="color: #10b981;">Inline code block</code>
        </div>
      </div>
    `);
    await captureAndExit('test-div-basic');
    break;

  case 'div-tailwind':
    await div({
      html: `
        <div class="text-center py-8">
          <h1 class="text-3xl font-bold text-blue-400 mb-4">Tailwind CSS Test</h1>
          <p class="text-gray-400 mb-6">Testing Tailwind utility classes</p>
          <div class="flex gap-3 justify-center flex-wrap">
            <span class="px-4 py-2 bg-green-500/20 text-green-400 rounded-lg">Success</span>
            <span class="px-4 py-2 bg-yellow-500/20 text-yellow-400 rounded-lg">Warning</span>
            <span class="px-4 py-2 bg-red-500/20 text-red-400 rounded-lg">Error</span>
            <span class="px-4 py-2 bg-blue-500/20 text-blue-400 rounded-lg">Info</span>
          </div>
          <div class="mt-6 p-4 bg-gray-800 rounded-lg text-left">
            <p class="text-sm text-gray-300">containerClasses: "p-6"</p>
          </div>
        </div>
      `,
      containerClasses: 'p-6'
    });
    await captureAndExit('test-div-tailwind');
    break;

  case 'arg-choices': {
    const choice = await arg('Select a programming language', [
      'JavaScript',
      'TypeScript', 
      'Python',
      'Rust',
      'Go'
    ]);
    console.error(`[RESULT] Selected: ${choice}`);
    await captureAndExit('test-arg-choices');
    break;
  }

  case 'arg-structured': {
    const action = await arg('Select an action', [
      { name: 'Run Script', value: 'run', description: 'Execute the current script' },
      { name: 'Edit Script', value: 'edit', description: 'Open script in VS Code' },
      { name: 'Delete Script', value: 'delete', description: 'Remove script from disk' },
      { name: 'Duplicate Script', value: 'duplicate', description: 'Create a copy' }
    ]);
    console.error(`[RESULT] Selected: ${action}`);
    await captureAndExit('test-arg-structured');
    break;
  }

  case 'arg-unicode': {
    const emoji = await arg('Select an item (Unicode test)', [
      '🍎 Apple',
      '🍌 Banana', 
      '🍒 Cherry',
      '日本語 Japanese',
      '中文 Chinese',
      'العربية Arabic'
    ]);
    console.error(`[RESULT] Selected: ${emoji}`);
    await captureAndExit('test-arg-unicode');
    break;
  }

  case 'arg-empty': {
    const text = await arg('Type anything (no choices)', []);
    console.error(`[RESULT] Typed: ${text}`);
    await captureAndExit('test-arg-empty');
    break;
  }

  case 'div-markdown':
    await div(md(`
# Markdown Rendering Test

This tests **bold**, *italic*, and \`inline code\`.

## Code Block

\`\`\`typescript
const greeting = "Hello, World!";
console.log(greeting);
\`\`\`

## List

- First item
- Second item  
- Third item

> This is a blockquote

[Link text](https://example.com)
    `));
    await captureAndExit('test-div-markdown');
    break;

  case 'arg-large': {
    const items = Array.from({ length: 100 }, (_, i) => `Item ${String(i + 1).padStart(3, '0')}`);
    const selected = await arg('Select from 100 items', items);
    console.error(`[RESULT] Selected: ${selected}`);
    await captureAndExit('test-arg-large');
    break;
  }

  default:
    console.error(`[ERROR] Unknown test type: ${testType}`);
    process.exit(1);
}
