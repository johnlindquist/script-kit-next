// Visual verification test for file search input alignment fix
// Captures both the mini main window and file search to compare
// @ts-nocheck
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = join(process.cwd(), 'test-screenshots', 'alignment-fix');
mkdirSync(dir, { recursive: true });

// Helper to capture screenshot
async function capture(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 600)); 
  const shot = await captureScreenshot();
  const path = join(dir, `${name}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

await expectMiniMainWindow('test-verify-alignment-fix', 0);

// Capture mini main window first
console.error('[TEST] Capturing mini main window...');
await capture('01-mini-main-window');

// Exit and let coordinator run separate test for file search
console.error('[TEST] Done - mini main window captured');
process.exit(0);
