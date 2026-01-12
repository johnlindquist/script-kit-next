// Test: AI Setup Card when no API keys are configured
// This test verifies the setup UX when opening AI chat without any API keys

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Wait for UI to render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const timestamp = Date.now();
const path = join(dir, `ai-setup-card-${timestamp}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] Saved to: ${path}`);
console.error(`[SCREENSHOT] Dimensions: ${screenshot.width}x${screenshot.height}`);

process.exit(0);
