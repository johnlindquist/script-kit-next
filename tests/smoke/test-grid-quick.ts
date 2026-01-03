import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[GRID] Starting quick grid test...');

// Show a simple div
await div(`<div class="p-8 text-white">
  <h1 class="text-xl mb-4">Grid Test</h1>
  <div class="flex gap-4">
    <div class="w-20 h-12 bg-blue-500">A</div>
    <div class="w-20 h-12 bg-green-500">B</div>
  </div>
</div>`);

console.error('[GRID] Div shown, enabling grid...');

// Enable grid overlay
await showGrid({ gridSize: 16, showBounds: true });

console.error('[GRID] Grid enabled, waiting...');
await new Promise(r => setTimeout(r, 800));

console.error('[GRID] Capturing...');
const shot = await captureScreenshot();
console.error(`[GRID] Got ${shot.width}x${shot.height}`);

const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const file = join(dir, `grid-quick-${Date.now()}.png`);
writeFileSync(file, Buffer.from(shot.data, 'base64'));
console.error(`[GRID] Saved: ${file}`);

console.error('[GRID] Done!');
process.exit(0);
