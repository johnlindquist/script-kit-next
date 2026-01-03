// Test script for showGrid with showDimensions option
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Grid Dimensions Test",
  description: "Tests the showDimensions option in the grid overlay",
};

console.error('[TEST] Starting grid dimensions test...');

// First, show the grid with showDimensions enabled
await showGrid({
  gridSize: 16,
  showBounds: true,
  showDimensions: true,  // The new feature we're testing
  showAlignmentGuides: false,
});

// Display some content to have something in the UI
await div(`
  <div class="p-4 space-y-4">
    <h1 class="text-xl font-bold">Grid Dimensions Test</h1>
    <p>The debug grid should show component dimensions in labels like "Header (500x45)"</p>
    <div class="flex gap-2">
      <button class="px-4 py-2 bg-blue-500 text-white rounded">Button 1</button>
      <button class="px-4 py-2 bg-green-500 text-white rounded">Button 2</button>
    </div>
  </div>
`);

// Wait for render
await new Promise(resolve => setTimeout(resolve, 1000));

// Capture screenshot to verify
const screenshot = await captureScreenshot();
console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

// Save to .test-screenshots/
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const filepath = join(dir, `grid-dimensions-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

// Hide the grid and exit
await hideGrid();

console.error('[TEST] Grid dimensions test complete');
process.exit(0);
