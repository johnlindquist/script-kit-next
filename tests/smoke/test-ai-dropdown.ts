import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test opens Agent Chat and captures a screenshot to verify
// the new chat dropdown button is visible in the header

console.error('[TEST] Starting AI dropdown visual test');

// Wait a moment for Agent Chat to fully render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot of Agent Chat
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `ai-window-dropdown-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

console.error('[TEST] Agent Chat dropdown test complete');
process.exit(0);
