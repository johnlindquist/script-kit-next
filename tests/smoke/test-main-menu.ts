// Test mini main window selection styling
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';

// Timer to capture screenshot after UI is displayed
setTimeout(async () => {
  try {
    const fs = await import('fs');
    const path = await import('path');
    await expectMiniMainWindow('test-main-menu', 0);
    
    const screenshot = await captureScreenshot();
    const dir = path.join(process.cwd(), 'test-screenshots');
    fs.mkdirSync(dir, { recursive: true });
    
    const filePath = path.join(dir, `mini-main-window-test-${Date.now()}.png`);
    fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${filePath}`);
  } catch (e) {
    console.error(`[ERROR] ${e}`);
  }
  process.exit(0);
}, 1500);

// Just wait indefinitely (timer will exit)
await new Promise(() => {});
