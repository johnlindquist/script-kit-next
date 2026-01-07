import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test triggers file search with a query that should find results

console.error('[TEST] Starting file search with results test');

// Wait for app to be ready
await new Promise(r => setTimeout(r, 500));

// Set input to search for something common (e.g., "main" should find files)
await setInput("main.rs");
await new Promise(r => setTimeout(r, 1500)); // Give time for mdfind search

// Take a screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `file-search-results-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to: ${path}`);

console.error('[TEST] File search with results test completed');
process.exit(0);
