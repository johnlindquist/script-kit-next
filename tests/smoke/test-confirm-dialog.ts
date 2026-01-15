// Test script for ConfirmDialog visual verification
// Tests the migrated Button component in the confirm dialog

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

async function main() {
  console.error('[TEST] Starting ConfirmDialog visual test');

  // Show a confirm dialog
  const resultPromise = confirm({
    message: "Are you sure you want to proceed?",
    confirmText: "Confirm",
    cancelText: "Cancel"
  });

  // Wait for dialog to render
  await new Promise(r => setTimeout(r, 800));

  // Capture screenshot
  try {
    const screenshot = await captureScreenshot();
    const dir = join(process.cwd(), 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    
    const path = join(dir, `confirm-dialog-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${path}`);
  } catch (e) {
    console.error(`[ERROR] Screenshot failed: ${e}`);
  }

  // Auto-cancel the dialog by simulating escape
  // For now, just exit after screenshot
  process.exit(0);
}

main().catch(e => {
  console.error(`[ERROR] Test failed: ${e}`);
  process.exit(1);
});
