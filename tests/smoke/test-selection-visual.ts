// Visual test for accent-tinted selection
import '../../scripts/kit-sdk';

// Display a div with styled content that mimics selection
await div(`
<div class="flex flex-col p-4 h-full">
  <h1 class="text-lg text-white mb-4">Accent Selection Test</h1>
  <p class="text-gray-400 mb-4">The first item below should have a subtle gold tint.</p>
  <div class="text-sm text-gray-500">Check the footer bar as well.</div>
</div>
`);

// Wait for render
await new Promise(r => setTimeout(r, 500));

const fs = await import('fs');
const path = await import('path');

const screenshot = await captureScreenshot();
const dir = path.join(process.cwd(), 'test-screenshots');
fs.mkdirSync(dir, { recursive: true });

const filePath = path.join(dir, `accent-test-${Date.now()}.png`);
fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filePath}`);
process.exit(0);
