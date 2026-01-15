// Test: Verify main window keeps vibrancy when actions window opens
// This tests that NSVisualEffectView state=1 (active) prevents dimming

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting actions window vibrancy test');

// Show main menu with some items
const result = await arg({
  placeholder: "Select an item to test actions",
  choices: [
    { name: "AI Chat", description: "Test vibrancy with actions", value: "ai" },
    { name: "System Settings", description: "Open settings", value: "settings" },
    { name: "Test Script", description: "A test", value: "test" },
  ],
  // Simulate pressing Cmd+K to open actions (user will need to do this manually)
  hint: "Press Cmd+K to open actions, then check vibrancy",
});

console.error(`[TEST] Selected: ${result}`);
process.exit(0);
