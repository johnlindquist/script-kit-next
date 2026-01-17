import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

// This test captures the main menu to verify the compact list item design
// Changes being tested:
// - List items at 24px height (down from 48px)
// - No descriptions shown
// - Section headers remain at 24px

// Wait for the UI to fully render
await new Promise(r => setTimeout(r, 800));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `compact-list-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to: ${path}`);

process.exit(0);
