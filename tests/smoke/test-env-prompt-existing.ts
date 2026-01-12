// Test: Verify env() prompt shows update mode with modification date
// Tests the "already configured" state with timestamp and delete option

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
    const path = join(dir, `env-prompt-existing-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    return path;
  } catch (e) {
    return null;
  }
})();

screenshotPromise.then(() => {
  process.exit(0);
});

// Use a key that should already exist (Vercel API key from AI setup)
// This should show the "update mode" with modification date
const value = await env("SCRIPT_KIT_VERCEL_API_KEY");
process.exit(0);
