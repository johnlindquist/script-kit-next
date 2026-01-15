import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test checks if the Notes window has vibrancy/transparency
// We'll open the notes window and capture a screenshot

console.error('[TEST] Starting Notes vibrancy test');

// Wait for app to initialize
await new Promise(r => setTimeout(r, 1000));

// The Notes window should already be open (via openNotes command)
// Capture screenshot of whatever window is visible
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `notes-vibrancy-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

console.error('[TEST] Notes vibrancy test complete');
process.exit(0);
