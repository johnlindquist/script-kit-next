// Test: Verify env() prompt with title (simulating API key setup)
// This captures a screenshot showing the full-window centered design

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Wait for window to be ready
await new Promise(r => setTimeout(r, 500));

// Capture screenshot after delay
const screenshotPromise = (async () => {
  await new Promise(r => setTimeout(r, 1000));
  try {
    const screenshot = await captureScreenshot();
    const projectDir = '/Users/johnlindquist/dev/script-kit-gpui';
    const dir = join(projectDir, 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    const path = join(dir, `env-prompt-titled-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    return path;
  } catch (e) {
    return null;
  }
})();

screenshotPromise.then(() => {
  process.exit(0);
});

// Use a key that would trigger the API setup flow
const value = await env("SCRIPT_KIT_ANTHROPIC_API_KEY");
process.exit(0);
