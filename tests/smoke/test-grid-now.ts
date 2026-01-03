import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[GRID-TEST] Starting grid overlay test...');

// First show a div with content
await div(`
<div class="flex flex-col gap-4 p-8 text-white">
  <h1 class="text-2xl font-bold">Debug Grid Overlay Test</h1>
  <p class="text-gray-300">Testing the visual debugging grid overlay.</p>
  <div class="flex gap-4 mt-4">
    <div class="w-24 h-16 bg-blue-500 rounded flex items-center justify-center">Box 1</div>
    <div class="w-24 h-16 bg-green-500 rounded flex items-center justify-center">Box 2</div>
    <div class="w-24 h-16 bg-red-500 rounded flex items-center justify-center">Box 3</div>
  </div>
  <div class="flex gap-4">
    <div class="w-24 h-16 bg-yellow-500 rounded flex items-center justify-center text-black">Box 4</div>
    <div class="w-24 h-16 bg-purple-500 rounded flex items-center justify-center">Box 5</div>
    <div class="w-24 h-16 bg-pink-500 rounded flex items-center justify-center">Box 6</div>
  </div>
</div>
`);

console.error('[GRID-TEST] Div displayed, enabling grid...');

// Enable the debug grid with all features
await showGrid({ 
  gridSize: 16, 
  showBounds: true,
  showAlignmentGuides: true
});

console.error('[GRID-TEST] Grid enabled, waiting for render...');

// Wait for render
await new Promise(r => setTimeout(r, 1000));

console.error('[GRID-TEST] Capturing screenshot...');

// Capture screenshot
const screenshot = await captureScreenshot();
console.error(`[GRID-TEST] Captured: ${screenshot.width}x${screenshot.height}`);

// Save screenshot
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const filepath = join(dir, `grid-overlay-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[GRID-TEST] Screenshot saved: ${filepath}`);

// Now test without grid
await hideGrid();
await new Promise(r => setTimeout(r, 500));

const screenshot2 = await captureScreenshot();
const filepath2 = join(dir, `grid-overlay-off-${Date.now()}.png`);
writeFileSync(filepath2, Buffer.from(screenshot2.data, 'base64'));
console.error(`[GRID-TEST] Screenshot (grid off) saved: ${filepath2}`);

console.error('[GRID-TEST] Test complete!');
process.exit(0);
