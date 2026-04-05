// Test script to verify Ask AI [Tab] hint is visible in the mini main window header
// @ts-nocheck
import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';

const fs = require('fs');
const path = require('path');

await expectMiniMainWindow('test-ask-ai-hint', 1000);

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = path.join(process.cwd(), 'test-screenshots');
fs.mkdirSync(dir, { recursive: true });
const filePath = path.join(dir, `mini-main-window-ask-ai-hint-${Date.now()}.png`);
fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filePath}`);

process.exit(0);
