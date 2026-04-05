// Visual test to capture BOTH the mini main window and file search views
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const DIR = join(process.cwd(), 'test-screenshots');
mkdirSync(DIR, { recursive: true });

async function captureAndSave(name: string) {
  await new Promise(r => setTimeout(r, 500)); // Wait for render
  const shot = await captureScreenshot();
  const path = join(DIR, `${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

const mainMenuLayout = await expectMiniMainWindow(
  'test-input-height-comparison',
  0
);

// Step 1: Capture mini main window
console.error('[TEST] Step 1: Mini main window view');
await captureAndSave('mini-main-window');

// Step 2: Use captured layout info for the mini main window
console.error('[TEST] Using layout info for mini main window...');
console.error('[LAYOUT] Mini main window:', JSON.stringify(mainMenuLayout, null, 2));

// Step 3: Open file search (we need to simulate selecting the builtin)
// Note: This test just captures the mini main window for now since we need
// user interaction to open file search

console.error('[TEST] Done - check test-screenshots/ for mini main window capture');
console.error('[TEST] To see file search, manually select "Search Files" builtin');

process.exit(0);
