import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Capturing mini main window input screenshot...');
await expectMiniMainWindow('test-main-input-gpui', 600);

const screenshot = await captureScreenshot();
console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const filepath = join(
  screenshotDir,
  `mini-main-window-input-${Date.now()}.png`
);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

await hide();

process.exit(0);
