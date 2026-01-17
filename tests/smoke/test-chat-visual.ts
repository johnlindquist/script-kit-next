// Visual test for chat auto-response
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting visual chat test');

// Start chat - it should auto-respond
const chatPromise = chat({
  placeholder: 'Ask follow-up...',
  hint: 'Visual Chat Test',
  system: `You are a helpful assistant. Just say "Hello! Built-in AI is working!" as your response.`,
  messages: [{ role: 'user', content: `Say hello!` }],
});

// Wait for streaming to complete
await new Promise(r => setTimeout(r, 3000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `chat-auto-response-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
