import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test header styling - shortcut badge should have grey background (no border)
// Logo should be 21px with yellow background at 85% opacity
// Yellow text for "Ask AI", "Run", "Actions"
// Grey text for "⌘↵", "↵", "⌘K"

// Use arg() to show the main script list view with header
// Don't await - just show it, then capture
arg({
  placeholder: "Header styling test - check the header elements",
  choices: [
    { name: "Apple", value: "apple", description: "A red fruit" },
    { name: "Banana", value: "banana", description: "A yellow fruit" },
    { name: "Cherry", value: "cherry", description: "A small red fruit" },
    { name: "Date", value: "date", description: "A sweet dried fruit" },
    { name: "Elderberry", value: "elderberry", description: "A dark purple berry" },
  ]
});

// Wait for render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `header-styling-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
