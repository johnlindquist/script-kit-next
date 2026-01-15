import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test opens the AI window and captures a screenshot to verify
// the new chat dropdown button is visible in the header

console.error('[TEST] Starting AI dropdown visual test');

// Wait a moment for the AI window to fully render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot of the AI window
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `ai-window-dropdown-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

console.error('[TEST] AI dropdown test complete');
process.exit(0);
