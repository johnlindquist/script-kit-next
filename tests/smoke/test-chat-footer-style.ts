// Test: Chat prompt footer styling with PromptFooter component
// Verifies the footer has yellow accents and Script Kit logo like main menu

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Show a chat prompt (don't await - just display it)
chat({
  placeholder: "Type your message...",
  hint: "Testing footer styling"
});

// Wait for render
await new Promise(r => setTimeout(r, 1500));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `chat-footer-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] Saved to: ${path}`);
console.error("[TEST] Visual verification:");
console.error("  - Footer should have Script Kit logo on left");
console.error("  - Model name should show next to logo with yellow accent");
console.error("  - 'Continue in Chat' button with yellow highlight");
console.error("  - 'Actions ⌘K' button with yellow highlight");

process.exit(0);
