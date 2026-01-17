import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test chat message vibrancy backgrounds
// The message containers should show system blur through white @ 8% opacity

// Don't await - we want to capture the UI while it's showing
const chatPromise = chat({
  placeholder: 'Ask follow-up...',
  messages: [
    { role: 'user', content: 'Test user message' },
    { role: 'assistant', content: 'This is a test assistant response to verify vibrancy.\n\n## The message background should be semi-transparent\n\n- Using white at 8% opacity\n- Lets system blur show through\n- Like other parts of the app' },
  ],
});

// Wait for render
await new Promise(r => setTimeout(r, 1500));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, 'chat-vibrancy-VERIFY.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved to: ${path}`);

process.exit(0);
