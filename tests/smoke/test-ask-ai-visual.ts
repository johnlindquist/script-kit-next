// @ts-nocheck
// Visual test for Ask AI [Tab] hint in the mini main window header
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';

const fs = require('fs');
const path = require('path');

await expectMiniMainWindow('test-ask-ai-visual', 1500);

// Capture screenshot
try {
  const screenshot = await captureScreenshot();
  const dir = path.join(process.cwd(), 'test-screenshots');
  fs.mkdirSync(dir, { recursive: true });
  const filePath = path.join(
    dir,
    `mini-main-window-ask-ai-hint-${Date.now()}.png`
  );
  fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] Saved to: ${filePath}`);
  console.error(`[SCREENSHOT] Size: ${screenshot.data.length} bytes`);
} catch (e) {
  console.error(`[SCREENSHOT ERROR] ${e}`);
}

process.exit(0);
