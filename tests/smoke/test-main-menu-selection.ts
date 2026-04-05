// Test: Verify first selectable item is selected when the mini main window opens
// This captures the mini main window state immediately on load
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

await expectMiniMainWindow('test-main-menu-selection', 300);

const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `mini-main-window-selection-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
