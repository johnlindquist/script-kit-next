// Test footer layout: Logo left, Run Script + Actions right
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Show test content with choices so footer is visible
const html = `
<div class="p-4 flex flex-col gap-2">
  <div class="text-lg font-bold text-yellow-400">Footer Layout Test</div>
  <div class="text-sm text-gray-400">Check footer at bottom:</div>
  <ul class="text-sm text-gray-300 list-disc ml-4">
    <li>Logo on the left (yellow box)</li>
    <li>"Run Script ↵" button</li>
    <li>Divider</li>
    <li>"Actions ⌘K" button</li>
  </ul>
</div>
`;

await div(html);

// Wait for render
await new Promise(r => setTimeout(r, 800));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `footer-layout-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
