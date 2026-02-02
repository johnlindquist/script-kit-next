// Name: Notes Markdown Highlight Visual Test
// Description: Captures Notes window with markdown content to verify highlighting

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Notes Markdown Highlight Visual Test');
console.error('[TEST] Expect headings bold, emphasis styles, link color, list markers, code span');

// Wait for Notes window to render (opened via stdin before this script)
await new Promise(resolve => setTimeout(resolve, 1500));

const screenshot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const filepath = join(dir, `notes-markdown-highlight-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

process.exit(0);
