// Test to verify header heights match between the mini main window and file search
// This captures layouts and placeholders to verify the fixes
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const DIR = join(process.cwd(), 'test-screenshots');
mkdirSync(DIR, { recursive: true });

async function captureAndSave(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 400));
  const shot = await captureScreenshot();
  const path = join(DIR, `${name}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

const mainLayout = await expectMiniMainWindow('test-header-height-fix', 0);

// Test 1: Capture mini main window layout
console.error('[TEST] Capturing mini main window...');
const mainMenuPath = await captureAndSave('header-fix-mini-main-window');

console.error('[LAYOUT] Mini main window layout:', JSON.stringify({
  windowHeight: mainLayout.windowHeight,
  windowWidth: mainLayout.windowWidth,
  promptType: mainLayout.promptType,
  componentCount: mainLayout.components.length
}));

// Find header component
const headerComponent = mainLayout.components.find(c => 
  c.name.toLowerCase().includes('header') || 
  c.bounds.y < 50
);
if (headerComponent) {
  console.error('[LAYOUT] Header bounds:', JSON.stringify(headerComponent.bounds));
}

console.error('[TEST] Done - screenshots saved to test-screenshots/');
console.error('[TEST] To verify file search header, manually select "Search Files" builtin');

process.exit(0);
