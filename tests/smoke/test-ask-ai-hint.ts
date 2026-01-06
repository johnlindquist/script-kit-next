// Test script to verify Ask AI [Tab] hint is visible in main menu header
// @ts-nocheck
import '../../scripts/kit-sdk';

const fs = require('fs');
const path = require('path');

// Wait for the main menu to render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = path.join(process.cwd(), 'test-screenshots');
fs.mkdirSync(dir, { recursive: true });
const filePath = path.join(dir, `ask-ai-hint-${Date.now()}.png`);
fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filePath}`);

process.exit(0);
