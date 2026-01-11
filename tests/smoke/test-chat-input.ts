// Test: Verify ChatPrompt input and copy icon
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Add a test message to see the copy icon
const testMessages = [
  { role: 'user' as const, content: 'Hello, this is a test message' },
  { role: 'assistant' as const, content: 'This is a response to test the copy icon visibility' }
];

// Open chat with messages to see the copy icon
chat({
  placeholder: "Ask anything...",
  messages: testMessages,
});

// Wait for render and capture screenshot
await new Promise(r => setTimeout(r, 1000));

const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `chat-input-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
