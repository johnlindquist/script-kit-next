// Test shortcut recorder keyboard focus and button clicks
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const testName = "shortcut-recorder-focus";
log(testName, "running");
const start = Date.now();

// Set up screenshot capture BEFORE await arg - fires while prompt is displayed
setTimeout(async () => {
  try {
    const shot = await captureScreenshot();
    const dir = join(process.cwd(), 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    const path = join(dir, `shortcut-recorder-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(shot.data, 'base64'));
    console.error(`[SCREENSHOT] ${path}`);

    log(testName, "pass", { duration_ms: Date.now() - start, screenshot: path });
  } catch (e) {
    log(testName, "fail", { error: String(e), duration_ms: Date.now() - start });
  }
  process.exit(0);
}, 1000);

// Show a simple arg prompt with choices to trigger actions
await arg({
  placeholder: "Test Shortcut Recorder - Press Cmd+K then select Assign Shortcut",
  hint: "Select an item and press Cmd+K to open actions",
}, [
  { name: "Test Item 1", description: "First test item", value: "item1" },
  { name: "Test Item 2", description: "Second test item", value: "item2" },
]);
