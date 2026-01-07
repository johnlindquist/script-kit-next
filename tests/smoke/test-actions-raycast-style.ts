// Name: Actions Raycast Style Visual Test
// Description: Tests the Raycast-style actions dialog - requires manual Cmd+K press

import '../../scripts/kit-sdk';

// @ts-ignore - Node.js modules available at runtime
const fs = require('fs');

const dir = `${process.cwd()}/test-screenshots`;
fs.mkdirSync(dir, { recursive: true });

console.error('[TEST] Starting Raycast-style actions dialog visual test...');
console.error('[TEST] *** PRESS Cmd+K WHEN THE WINDOW APPEARS ***');

// Test with SDK actions - these should display the actions panel when Cmd+K is pressed
const argPromise = arg({
  placeholder: "PRESS Cmd+K NOW to show actions...",
  choices: [
    { name: "Activity Monitor", description: "Monitor system processes", value: "activity" },
    { name: "Terminal", description: "Open terminal emulator", value: "terminal" },
    { name: "VS Code", description: "Code editor", value: "vscode" },
    { name: "Safari", description: "Web browser", value: "safari" },
    { name: "Finder", description: "File manager", value: "finder" },
  ],
  actions: [
    { name: "Open", shortcut: "enter" },
    { name: "Run in Terminal", shortcut: "cmd+t" },
    { name: "Edit Script", shortcut: "cmd+e" },
    { name: "Reveal in Finder", shortcut: "cmd+shift+f" },
    { name: "Copy Path", shortcut: "cmd+shift+c" },
    { name: "Configure Shortcut", shortcut: "cmd+k" },
    { name: "Delete", shortcut: "cmd+backspace" },
  ]
});

// Wait 3 seconds for user to press Cmd+K
console.error('[TEST] Waiting 3 seconds for Cmd+K press...');
await new Promise(r => setTimeout(r, 3000));

console.error('[TEST] Capturing screenshot (should show actions if Cmd+K pressed)...');
const actionsShot = await captureScreenshot();
const actionsPath = `${dir}/actions-raycast-${Date.now()}.png`;
// @ts-ignore - Buffer available at runtime
fs.writeFileSync(actionsPath, Buffer.from(actionsShot.data, 'base64'));
console.error(`[SCREENSHOT] ${actionsPath}`);
console.error(`[TEST] Size: ${actionsShot.width}x${actionsShot.height}`);

// Output test results
console.log(JSON.stringify({
  test: 'actions-raycast-style',
  status: 'captured',
  screenshot: actionsPath,
  dimensions: {
    width: actionsShot.width,
    height: actionsShot.height,
  },
  note: 'Verify actions popup is visible (requires manual Cmd+K press)',
  timestamp: new Date().toISOString(),
}));

console.error('[TEST] Test complete. Review screenshot to verify:');
console.error('[TEST] - Compact row spacing (tight rows)');
console.error('[TEST] - Pill-style selection (rounded bg, no left accent bar)');
console.error('[TEST] - Individual keycap badges for shortcuts');
console.error('[TEST] - Search input at bottom');

// Exit
await new Promise(r => setTimeout(r, 500));
process.exit(0);
