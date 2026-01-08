// Test to verify header alignment is consistent between main menu and file search
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const DIR = join(process.cwd(), 'test-screenshots');
mkdirSync(DIR, { recursive: true });

async function captureAndSave(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 500));
  const shot = await captureScreenshot();
  const path = join(DIR, `${name}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] Saved: ${path}`);
  return path;
}

// Capture main menu
console.error('[TEST] Capturing main menu with "Script Kit" placeholder...');
const mainPath = await captureAndSave('header-align-main');

// Get layout info for analysis
const layout = await getLayoutInfo();
console.error('[LAYOUT] Window:', layout.windowWidth, 'x', layout.windowHeight);
console.error('[LAYOUT] Prompt type:', layout.promptType);

console.error('[TEST] Done! Compare header-align-main.png with file search view manually.');
console.error('[TEST] Both should have identical placeholder vertical positioning.');

process.exit(0);
