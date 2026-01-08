// Visual test to capture BOTH main menu and file search views for comparison
import '../../scripts/kit-sdk';
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

// Step 1: Capture main menu (default state)
console.error('[TEST] Step 1: Main menu view');
await captureAndSave('main-menu');

// Step 2: Get layout info for main menu
console.error('[TEST] Getting layout info for main menu...');
const mainMenuLayout = await getLayoutInfo();
console.error('[LAYOUT] Main menu:', JSON.stringify(mainMenuLayout, null, 2));

// Step 3: Open file search (we need to simulate selecting the builtin)
// Note: This test just captures the main menu for now since we need
// user interaction to open file search

console.error('[TEST] Done - check test-screenshots/ for main menu capture');
console.error('[TEST] To see file search, manually select "Search Files" builtin');

process.exit(0);
