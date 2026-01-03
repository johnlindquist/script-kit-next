// Name: Template Visual Test
// Description: Visual test for template with screenshot capture

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Template Visual Test",
  description: "Visual test for template with footer info"
};

console.error('[TEST] Starting template visual test...');

// Set a timeout to capture screenshot and exit
setTimeout(async () => {
  console.error('[TEST] Capturing screenshot...');
  
  try {
    const screenshot = await captureScreenshot();
    console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);
    
    const dir = join(process.cwd(), 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    
    const filename = `template-footer-${Date.now()}.png`;
    const filepath = join(dir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    
    console.error(`[SCREENSHOT] ${filepath}`);
  } catch (err) {
    console.error('[TEST] Screenshot failed:', err);
  }
  
  process.exit(0);
}, 2000);

console.error('[TEST] Calling template()...');

// This opens the editor with template tabstops
// Expected footer: "Tab 1 of 2 - "name" - Tab to continue, Esc to exit"
const result = await template('Hello ${1:name}, welcome to ${2:place}!');

console.error('[TEST] Template completed with result:', result);
process.exit(0);
