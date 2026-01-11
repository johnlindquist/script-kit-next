// Test for accent-tinted selection colors
// This script shows the main menu to visually verify selection styling

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Set up screenshot capture BEFORE await arg - fires while prompt is displayed
setTimeout(async () => {
  const screenshot = await captureScreenshot();
  const dir = join(process.cwd(), 'test-screenshots');
  mkdirSync(dir, { recursive: true });

  const filePath = join(dir, `accent-selection-${Date.now()}.png`);
  writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filePath}`);
  process.exit(0);
}, 1000);

// Just show the main menu with some items
// The selection should have a subtle gold tint
await arg({
  placeholder: "Test accent-tinted selection",
  choices: [
    { name: "Minimize Window", description: "Minimize the frontmost window" },
    { name: "Mini Script", description: "A sample mini script" },
    { name: "Google Gemini", description: "AI assistant integration" },
    { name: "Migration Assistant", description: "Help with migrations" },
    { name: "System Information", description: "View system details" },
  ]
});
