// Name: Notes Raycast Parity Visual Test
// Description: Captures Notes window to verify Raycast-style hover UI design
//
// The Notes window should show when hovered:
// - Title in titlebar (centered or left-aligned)
// - Top-right action icons (⌘K shortcut, list icon, + icon)  
// - Editor area with placeholder or content
// - Footer with character count (centered) and T icon (right)

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Notes Raycast Parity Visual Test');
console.error('[TEST] Note: captureScreenshot() will capture the Notes window if it exists');

// Wait for Notes window to fully render (it was opened via stdin before this script)
await new Promise(resolve => setTimeout(resolve, 1500));

console.error('[TEST] Capturing screenshot...');
const screenshot = await captureScreenshot();
console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const filepath = join(dir, `notes-raycast-parity-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

console.error('[TEST] Expected Raycast parity elements:');
console.error('  - Titlebar with title text');
console.error('  - Top-right icons: ⌘K, list, +');
console.error('  - Editor with content/placeholder');
console.error('  - Footer: character count + T icon');

process.exit(0);
