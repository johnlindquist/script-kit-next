// Name: Visual Baseline Capture
// Description: Captures baseline screenshots of all major views for regression testing

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

// =============================================================================
// Configuration
// =============================================================================

const screenshotDir = join(process.cwd(), 'test-screenshots', 'baseline');
mkdirSync(screenshotDir, { recursive: true });

console.error('[BASELINE] Starting visual baseline capture...');
console.error(`[BASELINE] Output directory: ${screenshotDir}`);

// =============================================================================
// Helper Functions
// =============================================================================

async function capture(name: string, waitMs: number = 500): Promise<string> {
  await new Promise(r => setTimeout(r, waitMs));
  const screenshot = await captureScreenshot();
  const path = join(screenshotDir, `baseline-${name}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${screenshot.width}x${screenshot.height} -> ${path}`);
  return path;
}

const capturedScreenshots: string[] = [];

// =============================================================================
// 1. Main Menu (default view on launch)
// =============================================================================

console.error('[BASELINE] 1/9: Main Menu');
// The main menu is already showing when app launches
// Just capture it directly
await capture('main-menu', 800);
capturedScreenshots.push('baseline-main-menu.png');

// =============================================================================
// 2. Arg Prompt with Choices
// =============================================================================

console.error('[BASELINE] 2/9: Arg Prompt with Choices');
arg('Select an option', [
  { name: 'Option One', value: '1', description: 'First choice in the list' },
  { name: 'Option Two', value: '2', description: 'Second choice in the list' },
  { name: 'Option Three', value: '3', description: 'Third choice in the list' },
  { name: 'Option Four', value: '4', description: 'Fourth choice in the list' },
  { name: 'Option Five', value: '5', description: 'Fifth choice in the list' }
]);
await capture('arg-prompt');
capturedScreenshots.push('baseline-arg-prompt.png');

// =============================================================================
// 3. Div Prompt with HTML Content
// =============================================================================

console.error('[BASELINE] 3/9: Div Prompt with HTML');
div(`
  <div class="p-6 space-y-4">
    <h1 class="text-2xl font-bold text-blue-400">Visual Baseline Test</h1>
    <p class="text-gray-300">Testing div prompt with HTML content and Tailwind styling.</p>
    <div class="flex gap-2 flex-wrap">
      <span class="px-3 py-1 bg-green-500/20 text-green-400 rounded">Tag One</span>
      <span class="px-3 py-1 bg-blue-500/20 text-blue-400 rounded">Tag Two</span>
      <span class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded">Tag Three</span>
    </div>
    <div class="mt-4 p-4 bg-gray-800/50 rounded border border-gray-700">
      <code class="text-green-300">const baseline = "captured";</code>
    </div>
  </div>
`);
await capture('div-prompt');
capturedScreenshots.push('baseline-div-prompt.png');

// =============================================================================
// 4. Editor Prompt
// =============================================================================

console.error('[BASELINE] 4/9: Editor Prompt');
editor(`// Visual Baseline: Editor Prompt
// This captures the code editor view with syntax highlighting

function greet(name: string): string {
  return \`Hello, \${name}!\`;
}

// The editor should fill the window (700px height)
const result = greet("World");
console.log(result);

// Test syntax highlighting for different elements:
const number = 42;
const boolean = true;
const array = [1, 2, 3];
const object = { key: "value" };

export { greet, result };
`, 'typescript');
await capture('editor-prompt', 800);
capturedScreenshots.push('baseline-editor-prompt.png');

// =============================================================================
// 5. Term Prompt (Terminal)
// =============================================================================

console.error('[BASELINE] 5/9: Term Prompt');
term('echo "=== TERMINAL BASELINE ===" && echo "" && echo "Testing terminal output display" && echo "" && for i in 1 2 3 4 5; do echo "Line $i: Terminal content test"; done && echo "" && echo "Terminal baseline capture complete" && sleep 2');
await capture('term-prompt', 1500);
capturedScreenshots.push('baseline-term-prompt.png');

// =============================================================================
// 6. Form Prompt
// =============================================================================

console.error('[BASELINE] 6/9: Form Prompt');
const formHtml = `
<div class="p-4 space-y-4">
  <h2 class="text-lg font-bold mb-4 text-white">Sample Form</h2>
  
  <div class="space-y-2">
    <label for="name" class="block text-sm font-medium text-gray-300">Name</label>
    <input type="text" name="name" id="name" placeholder="Enter your name" 
           class="w-full px-4 py-2 bg-gray-800 border border-gray-600 rounded text-white" />
  </div>
  
  <div class="space-y-2">
    <label for="email" class="block text-sm font-medium text-gray-300">Email</label>
    <input type="email" name="email" id="email" placeholder="you@example.com" 
           class="w-full px-4 py-2 bg-gray-800 border border-gray-600 rounded text-white" />
  </div>
  
  <div class="space-y-2">
    <label for="message" class="block text-sm font-medium text-gray-300">Message</label>
    <textarea name="message" id="message" rows="3" placeholder="Your message..." 
              class="w-full px-4 py-2 bg-gray-800 border border-gray-600 rounded text-white"></textarea>
  </div>
  
  <div class="flex items-center space-x-2">
    <input type="checkbox" name="subscribe" id="subscribe" 
           class="h-4 w-4 rounded border-gray-600 bg-gray-800" />
    <label for="subscribe" class="text-sm text-gray-300">Subscribe to updates</label>
  </div>
</div>
`;
form(formHtml);
await capture('form-prompt');
capturedScreenshots.push('baseline-form-prompt.png');

// =============================================================================
// 7. Path Prompt
// =============================================================================

console.error('[BASELINE] 7/9: Path Prompt');
path({ startPath: process.env.HOME || '/', hint: 'Select a file or folder' });
await capture('path-prompt', 800);
capturedScreenshots.push('baseline-path-prompt.png');

// =============================================================================
// 8. Arg Prompt with Text Input (Empty Choices)
// =============================================================================

console.error('[BASELINE] 8/9: Arg Text Input (no choices)');
arg({ placeholder: 'Type something here...', hint: 'Text input mode with no choices' });
await capture('arg-text-input');
capturedScreenshots.push('baseline-arg-text-input.png');

// =============================================================================
// 9. Div with Markdown
// =============================================================================

console.error('[BASELINE] 9/9: Div with Markdown');
div(md(`
# Markdown Rendering Test

This tests **bold**, *italic*, and \`inline code\`.

## Code Block

\`\`\`typescript
const greeting = "Hello, World!";
console.log(greeting);
\`\`\`

## List

- Item one
- Item two
- Item three

> A blockquote for testing

---

*Baseline capture complete!*
`));
await capture('div-markdown');
capturedScreenshots.push('baseline-div-markdown.png');

// =============================================================================
// Summary
// =============================================================================

console.error('[BASELINE] === CAPTURE COMPLETE ===');
console.error('[BASELINE] Screenshots captured:');
capturedScreenshots.forEach(s => console.error(`  - ${s}`));
console.error(`[BASELINE] Total: ${capturedScreenshots.length} screenshots`);
console.error(`[BASELINE] Directory: ${screenshotDir}`);

// Show completion summary
div(md(`
# Visual Baseline Capture Complete

**${capturedScreenshots.length} screenshots captured** to \`test-screenshots/baseline/\`

## Captured Views

1. \`baseline-main-menu.png\` - Main menu on launch
2. \`baseline-arg-prompt.png\` - Arg with choices
3. \`baseline-div-prompt.png\` - Div with HTML
4. \`baseline-editor-prompt.png\` - Code editor
5. \`baseline-term-prompt.png\` - Terminal
6. \`baseline-form-prompt.png\` - Form input
7. \`baseline-path-prompt.png\` - File browser
8. \`baseline-arg-text-input.png\` - Text input mode
9. \`baseline-div-markdown.png\` - Markdown rendering

---

These baseline screenshots serve as reference for visual regression testing.

Press Enter or Escape to exit.
`));

await new Promise(r => setTimeout(r, 1000));
await capture('summary');

console.error('[BASELINE] Exiting...');
process.exit(0);
