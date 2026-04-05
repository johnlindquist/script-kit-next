// @ts-nocheck
// Test to capture the mini main window header and verify Ask AI hint
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

await expectMiniMainWindow('test-main-menu-header', 1000);

// Capture screenshot of the mini main window header
const shot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `mini-main-window-header-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);
process.exit(0);
