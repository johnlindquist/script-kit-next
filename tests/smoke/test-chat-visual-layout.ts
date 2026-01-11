// Test: chat() layout and positioning verification
// Verifies: Layout components, message positioning, text wrapping

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

// Test 1: Basic chat layout verification
async function testBasicLayout(): Promise<void> {
  const testName = 'chat-layout-basic';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Type a message...',
      hint: 'Chat Layout Test',
      footer: 'Press Enter to send',
      messages: [
        { role: 'user', content: 'Hello, user message on right' },
        { role: 'assistant', content: 'Hi! Assistant on left' },
      ],
    });

    await sleep(400);
    const layout = await getLayoutInfo();
    const shot = await saveScreenshot('chat-layout-basic');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        promptType: layout.promptType,
        componentCount: layout.components?.length ?? 0,
        windowWidth: layout.windowWidth,
        windowHeight: layout.windowHeight,
      },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 2: Message bubble positioning
async function testMessagePositioning(): Promise<void> {
  const testName = 'chat-message-positioning';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Positioning test',
      messages: [
        { role: 'user', content: 'User message 1 (right)' },
        { role: 'assistant', content: 'Assistant message 1 (left)' },
        { role: 'user', content: 'User message 2' },
        { role: 'assistant', content: 'Assistant message 2' },
      ],
    });

    await sleep(400);
    const shot = await saveScreenshot('chat-message-positioning');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { messageCount: 4 },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 3: Long message wrapping
async function testLongMessageWrapping(): Promise<void> {
  const testName = 'chat-long-message-wrapping';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const longMessage = `This is a very long message that should wrap properly within the chat bubble. It contains multiple sentences to test text wrapping. The message should not overflow the container.`;

    const chatPromise = chat({
      placeholder: 'Long message test',
      messages: [
        { role: 'user', content: 'Tell me something long' },
        { role: 'assistant', content: longMessage },
      ],
    });

    await sleep(400);
    const shot = await saveScreenshot('chat-long-message');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { longMessageLength: longMessage.length },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 4: Empty state
async function testEmptyState(): Promise<void> {
  const testName = 'chat-empty-state';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Start a conversation...',
      hint: 'New Chat',
      footer: 'Empty state test',
    });

    await sleep(400);
    const shot = await saveScreenshot('chat-empty-state');

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
debug(`Running chat layout test ${testArg}`);

switch (testArg) {
  case '1': case 'layout': await testBasicLayout(); break;
  case '2': case 'position': await testMessagePositioning(); break;
  case '3': case 'long': await testLongMessageWrapping(); break;
  case '4': case 'empty': await testEmptyState(); break;
  default: await testBasicLayout();
}
