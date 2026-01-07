import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test triggers file search via fallback action
// When no script matches, typing and pressing Enter should trigger "Search Files"

console.error('[TEST] Starting file search test');

// Wait for app to be ready
await new Promise(r => setTimeout(r, 500));

// Type a search query that won't match any scripts
await setInput("test-nonexistent-query-12345");
await new Promise(r => setTimeout(r, 300));

// Take a screenshot to see the current state
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `file-search-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to: ${path}`);

console.error('[TEST] File search test completed');
process.exit(0);
