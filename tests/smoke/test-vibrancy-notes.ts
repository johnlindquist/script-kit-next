// Test vibrancy effect in Notes window
// @ts-nocheck
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Give the Notes window time to fully render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot of the current window
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, 'notes-vibrancy-' + Date.now() + '.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error('[SCREENSHOT] ' + path);
console.error('[VIBRANCY_TEST] Screenshot saved - check for blur/vibrancy effect');

process.exit(0);
