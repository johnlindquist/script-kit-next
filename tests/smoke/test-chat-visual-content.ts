// Test: chat() content rendering and streaming
// Verifies: Markdown, streaming animation, many messages

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  duration_ms?: number;
  error?: string;
  screenshot?: string;
  details?: Record<string, unknown>;
}

function log(result: TestResult): void {
  console.log(JSON.stringify(result));
}

function debug(msg: string): void {
  console.error(`[TEST] ${msg}`);
}

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function saveScreenshot(name: string): Promise<string> {
  const shot = await captureScreenshot();
  const dir = join(process.cwd(), '.test-screenshots');
  mkdirSync(dir, { recursive: true });
  const path = join(dir, `${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

// Test 1: Markdown rendering
async function testMarkdownRendering(): Promise<void> {
  const testName = 'chat-markdown-rendering';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const markdownContent = `Here's markdown:

## Code Example
\`\`\`typescript
function hello(name: string) {
  return \`Hello, \${name}!\`;
}
\`\`\`

**Bold** and *italic* text. Inline \`code\` too.

- Bullet 1
- Bullet 2`;

    const chatPromise = chat({
      placeholder: 'Markdown test',
      messages: [
        { role: 'user', content: 'Show markdown' },
        { role: 'assistant', content: markdownContent },
      ],
    });

    await sleep(500);
    const shot = await saveScreenshot('chat-markdown');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { hasCodeBlock: true, hasBold: true },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 2: Streaming animation visual
async function testStreamingVisual(): Promise<void> {
  const testName = 'chat-streaming-visual';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Streaming test',
      messages: [{ role: 'user', content: 'Generate response' }],
      async onInit() {
        const msgId = chat.startStream('left');
        const response = 'This is a streaming response that builds up over time word by word.';
        const words = response.split(' ');

        for (const word of words) {
          chat.appendChunk(msgId, word + ' ');
          await sleep(80);
        }
        chat.completeStream(msgId);
      },
    });

    await sleep(1500);
    const shot = await saveScreenshot('chat-streaming-complete');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 3: Many messages (scroll)
async function testManyMessages(): Promise<void> {
  const testName = 'chat-many-messages';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const messages: Array<{ role: 'user' | 'assistant'; content: string }> = [];
    for (let i = 1; i <= 15; i++) {
      messages.push({ role: 'user', content: `User message ${i}` });
      messages.push({ role: 'assistant', content: `Response ${i}` });
    }

    const chatPromise = chat({ placeholder: 'Many messages', messages });

    await sleep(500);
    const shot = await saveScreenshot('chat-many-messages');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { messageCount: messages.length },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 4: Full conversation with all options
async function testFullConversation(): Promise<void> {
  const testName = 'chat-full-conversation';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Ask anything...',
      hint: 'AI Assistant',
      footer: 'Model: Claude 3.5',
      system: 'You are helpful.',
      messages: [
        { role: 'user', content: 'Write TypeScript' },
      ],
      async onInit() {
        const msgId = chat.startStream('left');
        const code = `\`\`\`typescript
interface User { name: string; }
const greet = (u: User) => \`Hi \${u.name}\`;
\`\`\``;
        for (const char of code) {
          chat.appendChunk(msgId, char);
          if (char === '\n') await sleep(15);
        }
        chat.completeStream(msgId);
      },
    });

    await sleep(2000);
    const shot = await saveScreenshot('chat-full-conversation');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Run test
const testArg = process.argv[2] || '1';
debug(`Running chat content test ${testArg}`);

switch (testArg) {
  case '1': case 'markdown': await testMarkdownRendering(); break;
  case '2': case 'streaming': await testStreamingVisual(); break;
  case '3': case 'many': await testManyMessages(); break;
  case '4': case 'full': await testFullConversation(); break;
  default: await testMarkdownRendering();
}
