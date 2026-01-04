import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Testing notes window with hover state...');

// Wait for the notes window to be ready
await new Promise(r => setTimeout(r, 2000));

// Capture screenshot
try {
  const screenshot = await captureScreenshot();
  console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

  const dir = join(process.cwd(), 'test-screenshots');
  mkdirSync(dir, { recursive: true });
  const filepath = join(dir, `notes-hover-test-${Date.now()}.png`);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
} catch (e) {
  console.error('[ERROR] Screenshot failed:', e);
}

process.exit(0);
