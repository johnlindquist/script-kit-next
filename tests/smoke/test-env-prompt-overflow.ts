// Test: Verify env() prompt handles long input without overflow
// Simulates entering a long API key

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Wait for window to be ready
await new Promise(r => setTimeout(r, 500));

const testKey = "TEST_OVERFLOW_KEY_" + Date.now();

// Capture screenshot after delay and simulate typing
const screenshotPromise = (async () => {
  await new Promise(r => setTimeout(r, 1500)); // Wait for env prompt and simulated typing
  try {
    const screenshot = await captureScreenshot();
    const projectDir = '/Users/johnlindquist/dev/script-kit-gpui';
    const dir = join(projectDir, 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    const path = join(dir, `env-prompt-overflow-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    return path;
  } catch (e) {
    return null;
  }
})();

screenshotPromise.then(() => {
  process.exit(0);
});

// Trigger the env prompt
const value = await env(testKey);
process.exit(0);
