import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[GRID-DBG] Starting debug grid test...');

// Show a simple div first
await div(`<div class="p-8 text-white"><h1>Grid Debug Test</h1></div>`);
console.error('[GRID-DBG] Div shown');

// Small delay
await new Promise(r => setTimeout(r, 300));

// Manually write the showGrid message to stdout to trace it
console.error('[GRID-DBG] About to call showGrid...');

// Test if showGrid is defined
console.error('[GRID-DBG] typeof showGrid:', typeof showGrid);

try {
  console.error('[GRID-DBG] Calling showGrid now...');
  await showGrid({ gridSize: 16, showBounds: true });
  console.error('[GRID-DBG] showGrid returned (no error)');
} catch (e: any) {
  console.error('[GRID-DBG] showGrid threw error:', e.message);
}

// Wait for render
await new Promise(r => setTimeout(r, 500));

console.error('[GRID-DBG] Capturing screenshot...');
try {
  const shot = await captureScreenshot();
  console.error(`[GRID-DBG] Screenshot captured: ${shot.width}x${shot.height}`);
  
  const dir = join(process.cwd(), '.test-screenshots');
  mkdirSync(dir, { recursive: true });
  const file = join(dir, `grid-debug-${Date.now()}.png`);
  writeFileSync(file, Buffer.from(shot.data, 'base64'));
  console.error(`[GRID-DBG] Saved: ${file}`);
} catch (e: any) {
  console.error('[GRID-DBG] Screenshot failed:', e.message);
}

console.error('[GRID-DBG] Test complete, exiting...');
process.exit(0);
