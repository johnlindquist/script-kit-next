// Name: Test Actions Opacity
// Description: Visually verify actions dialog opacity/transparency

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), '.test-screenshots', 'actions');
mkdirSync(screenshotDir, { recursive: true });

async function capture(name: string) {
  await new Promise(r => setTimeout(r, 400));
  const ss = await captureScreenshot();
  const path = join(screenshotDir, `${name}.png`);
  writeFileSync(path, Buffer.from(ss.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

// Show main menu with some script-like choices
console.error('[TEST] Displaying main menu for actions test');

// This test requires manual interaction to open the actions dialog:
// 1. The script shows a list
// 2. User presses Cmd+K to open actions
// 3. Screenshot would capture the actions dialog

// For now, just capture the main window state
const choices = [
  { name: 'ACP Chat', value: 'acp-chat', description: 'Open the primary ACP Chat surface' },
  { name: 'Bluetooth Settings', value: 'bluetooth', description: 'Open Bluetooth settings' },
  { name: 'Check Permissions', value: 'permissions', description: 'Check all required macOS permissions' },
  { name: 'Clear Suggested', value: 'clear', description: 'Clear all suggested/recently used items' },
  { name: 'Clipboard History', value: 'clipboard', description: 'View and manage your clipboard history' },
];

// Show the arg prompt - user needs to press Cmd+K to see the actions dialog
arg({
  placeholder: 'Test Actions Opacity - Press Cmd+K for actions',
  choices,
  onInit: async () => {
    console.error('[TEST] Main menu displayed, waiting for Cmd+K');
  }
});

// Capture before user interacts
await capture('actions-main-menu');

console.error('[TEST] Press Cmd+K to open actions dialog, then press Enter to capture');
