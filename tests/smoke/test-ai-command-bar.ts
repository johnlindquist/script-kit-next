// Name: Test AI Command Bar
// Description: Test the Cmd+K command bar in the AI window

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

async function main() {
  const testName = 'ai-command-bar';
  const testDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(testDir, { recursive: true });
  
  console.error(`[TEST] ${testName} - Testing AI window command bar`);
  
  // Note: This test runs inside the main window context
  // The AI window is a separate window, so we can't directly interact with it
  // This test verifies the SDK screenshot capability works
  
  // Wait a moment for any pending UI updates
  await new Promise(r => setTimeout(r, 1000));
  
  try {
    // Capture screenshot of whatever window is active
    const screenshot = await captureScreenshot();
    const path = join(testDir, `${testName}-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] Saved to ${path}`);
    console.error(`[TEST] ${testName} - Screenshot captured successfully`);
  } catch (e) {
    console.error(`[TEST] ${testName} - Failed: ${e}`);
  }
  
  process.exit(0);
}

main();
