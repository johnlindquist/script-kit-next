import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test triggers file search via the TriggerBuiltin stdin command
// We send the command via the SDK's internal mechanisms

console.error('[TEST] Starting file search builtin test');

// Wait for app to be ready
await new Promise(r => setTimeout(r, 1000));

// Take a screenshot to see the current state (should be file search view)
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `file-search-builtin-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to: ${path}`);

console.error('[TEST] File search builtin test completed');
process.exit(0);
