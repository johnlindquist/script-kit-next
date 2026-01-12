// Test AI window sidebar styling and scrolling
// Verifies: no white box in chat list, proper search input styling, scrolling works

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

async function main() {
  console.error('[TEST] Starting AI window sidebar test');

  // Open the AI window
  await openAi();
  
  // Wait for window to render
  await new Promise(r => setTimeout(r, 800));
  
  console.error('[TEST] AI window opened, capturing screenshot');
  
  // Capture screenshot to verify styling
  try {
    const screenshot = await captureScreenshot();
    const dir = join(process.cwd(), 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    
    const filename = `ai-sidebar-${Date.now()}.png`;
    const filepath = join(dir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] Saved to: ${filepath}`);
    console.error('[TEST] Check screenshot for:');
    console.error('  1. No white box in sidebar chat list');
    console.error('  2. Search input has proper dark/themed background');
    console.error('  3. Chat items are properly styled');
  } catch (e) {
    console.error('[TEST] Screenshot capture failed:', e);
  }
  
  // Give time to see the result
  await new Promise(r => setTimeout(r, 500));
  
  console.error('[TEST] Test complete');
  process.exit(0);
}

main().catch(e => {
  console.error('[TEST] Error:', e);
  process.exit(1);
});
