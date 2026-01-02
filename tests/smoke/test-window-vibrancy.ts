// Description: Capture main window vibrancy for visual verification

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Main window vibrancy capture');

const editorPromise = editor(
  `# Vibrancy Check

Background blur should be visible behind this window.`,
  'markdown'
);

setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    const cwd = process.cwd();
    const dir = join(cwd, 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    const filepath = join(dir, 'vibrancy-main.png');
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[TEST] cwd: ${cwd}`);
    console.error(`[SCREENSHOT] ${filepath}`);
    console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);
  } catch (err) {
    console.error('[TEST] Screenshot capture failed:', err);
  } finally {
    try {
      await editorPromise;
    } catch (_err) {
      // Ignore if the prompt was already closed
    }
    process.exit(0);
  }
}, 900);
