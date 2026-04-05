import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Capture the main script list header immediately
// The mini main window should already be visible before this script runs
// This script does NOT call any UI functions (arg, div, etc.) so it won't change the view
await expectMiniMainWindow('test-main-header', 0);

const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `mini-main-window-header-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
