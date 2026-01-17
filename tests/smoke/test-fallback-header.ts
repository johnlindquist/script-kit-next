import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Type some text to trigger fallback results
await setInput("xyzabc");
await new Promise(r => setTimeout(r, 500));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `fallback-header-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error('[SCREENSHOT] Saved to: ' + path);

process.exit(0);
