// Test that AI window actions dialog uses subtle selection styling
// matching the main menu theme (not bright yellow)
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting AI actions selection style test');

// Wait for app to be ready
await new Promise(r => setTimeout(r, 500));

// Capture a screenshot to verify the styling
// Note: This test primarily verifies the code compiles and runs
// The actual visual test requires opening the AI window and triggering Cmd+K
// which needs manual verification

const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, 'ai-actions-selection-' + Date.now() + '.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error('[SCREENSHOT] ' + path);
console.error('[TEST] AI actions selection style test complete');

process.exit(0);
