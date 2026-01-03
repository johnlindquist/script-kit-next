import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[GRID-FINAL] Starting final grid test...');

// Enable grid FIRST (before any UI)
console.error('[GRID-FINAL] Enabling grid overlay...');
await showGrid({ 
  gridSize: 16, 
  showBounds: true,
  showAlignmentGuides: true 
});
console.error('[GRID-FINAL] Grid enabled');

// Small delay
await new Promise(r => setTimeout(r, 300));

// Now capture the screenshot while grid is on main script list
console.error('[GRID-FINAL] Capturing main view with grid...');
const shot1 = await captureScreenshot();
console.error(`[GRID-FINAL] Got ${shot1.width}x${shot1.height}`);

const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const file1 = join(dir, `grid-final-main-${Date.now()}.png`);
writeFileSync(file1, Buffer.from(shot1.data, 'base64'));
console.error(`[GRID-FINAL] Saved: ${file1}`);

// Hide grid
await hideGrid();
console.error('[GRID-FINAL] Grid hidden');

await new Promise(r => setTimeout(r, 300));

// Capture without grid
console.error('[GRID-FINAL] Capturing without grid...');
const shot2 = await captureScreenshot();
const file2 = join(dir, `grid-final-nogrid-${Date.now()}.png`);
writeFileSync(file2, Buffer.from(shot2.data, 'base64'));
console.error(`[GRID-FINAL] Saved: ${file2}`);

console.error('[GRID-FINAL] Test complete!');
process.exit(0);
