// Test script to verify the chat prompt footer displays correctly
// The footer should show:
// - Left: Model indicator (dot + model name)
// - Right: "Continue in Chat ⌘↵" and "Actions ⌘K" buttons via PromptFooter

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Open chat with onMessage handler - it won't resolve but we can capture UI
const chatPromise = chat({
  placeholder: "Test chat prompt...",
  model: "gpt-4o-mini",
  onMessage: async (text) => {
    // This won't be called since we capture and exit before user input
    console.error(`[CHAT] onMessage: ${text}`);
  }
});

// Wait for UI to render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const shot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `chat-footer-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to ${path}`);

process.exit(0);
