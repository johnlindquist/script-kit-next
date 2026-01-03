import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Disable auto-dismiss by keeping the window visible
console.error('[GRID] Starting focused grid test...');

// First, use setAlwaysOnTop to keep window active (if available)
try {
  // @ts-ignore - may not be in types yet
  await setAlwaysOnTop?.(true);
} catch (e) {
  console.error('[GRID] setAlwaysOnTop not available, continuing...');
}

// Show a div 
await div(`<div class="p-8 text-white">
  <h1 class="text-xl mb-4">Grid Test</h1>
  <div class="flex gap-4">
    <div class="w-20 h-12 bg-blue-500">A</div>
    <div class="w-20 h-12 bg-green-500">B</div>
  </div>
</div>`);

console.error('[GRID] Div shown');

// Small delay
await new Promise(r => setTimeout(r, 200));

console.error('[GRID] Calling showGrid...');
await showGrid({ gridSize: 16, showBounds: true });
console.error('[GRID] showGrid called');

// Wait for render
await new Promise(r => setTimeout(r, 800));

console.error('[GRID] Capturing screenshot...');
const shot = await captureScreenshot();
console.error(`[GRID] Got ${shot.width}x${shot.height}`);

const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const file = join(dir, `grid-focused-${Date.now()}.png`);
writeFileSync(file, Buffer.from(shot.data, 'base64'));
console.error(`[GRID] Saved: ${file}`);

console.error('[GRID] Test complete!');
process.exit(0);
