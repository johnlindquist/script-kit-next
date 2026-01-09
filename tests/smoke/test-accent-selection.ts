// Test for accent-tinted selection colors
// This script shows the main menu to visually verify selection styling

import '../../scripts/kit-sdk';

// Just show the main menu with some items
// The selection should have a subtle gold tint
const choice = await arg({
  placeholder: "Test accent-tinted selection",
  choices: [
    { name: "Minimize Window", description: "Minimize the frontmost window" },
    { name: "Mini Script", description: "A sample mini script" },
    { name: "Google Gemini", description: "AI assistant integration" },
    { name: "Migration Assistant", description: "Help with migrations" },
    { name: "System Information", description: "View system details" },
  ]
});

// Short delay then screenshot
await new Promise(r => setTimeout(r, 300));

const fs = await import('fs');
const path = await import('path');

const screenshot = await captureScreenshot();
const dir = path.join(process.cwd(), 'test-screenshots');
fs.mkdirSync(dir, { recursive: true });

const filePath = path.join(dir, `accent-selection-${Date.now()}.png`);
fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filePath}`);

console.log(`Selected: ${choice}`);
process.exit(0);
