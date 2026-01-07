import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test that the actions window is positioned at the bottom (aligned with footer)
// The user reported it was too high and needed to "scooch down"

// First, set up a simple list to have something to show actions for
const result = await arg({
  placeholder: "Select a script to test actions positioning",
  choices: [
    { name: "Clipboard History", value: "clipboard" },
    { name: "Quick Terminal", value: "terminal" },
    { name: "Scratch Pad", value: "scratch" },
    { name: "Window Switcher", value: "window" },
  ],
  onInit: async () => {
    // Wait for UI to settle
    await new Promise(r => setTimeout(r, 500));
    
    // Simulate Cmd+K to open actions window
    await keyboard.pressKey(Key.LeftSuper);
    await keyboard.type("k");
    await keyboard.releaseKey(Key.LeftSuper);
    
    // Wait for actions window to open
    await new Promise(r => setTimeout(r, 500));
    
    // Capture screenshot
    const shot = await captureScreenshot();
    const dir = join(process.cwd(), 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    
    const path = join(dir, `actions-position-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(shot.data, 'base64'));
    console.error(`[SCREENSHOT] ${path}`);
    
    // Exit after capturing
    process.exit(0);
  }
});
