// Test: Verify env() prompt visual appearance
// This test triggers an env prompt and captures a screenshot

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Wait a moment for window to be ready
await new Promise(r => setTimeout(r, 500));

// Use a unique test key
const testKey = "TEST_ENV_PROMPT_KEY_" + Date.now();

// Capture screenshot after a delay (prompt will be showing)
const screenshotPromise = (async () => {
  await new Promise(r => setTimeout(r, 1000)); // Wait for env prompt to render
  try {
    const screenshot = await captureScreenshot();
    // Use absolute path to the project directory
    const projectDir = '/Users/johnlindquist/dev/script-kit-gpui';
    const dir = join(projectDir, 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    const path = join(dir, `env-prompt-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    return path;
  } catch (e) {
    return null;
  }
})();

// Start the env prompt - it will wait for user input
// But we exit after the screenshot is captured
screenshotPromise.then((path) => {
  console.error(`[DONE] Screenshot complete: ${path}`);
  process.exit(0);
});

// Trigger the env prompt (this will block until user input or escape)
const value = await env(testKey);
console.error(`[RESULT] Got value: ${value ? "(value provided)" : "(cancelled)"}`);
process.exit(0);
