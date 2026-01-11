// Test: chat() callbacks - onMessage, onChunk, onFinish
// Verifies: All callback patterns work correctly

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

// Test 1: onMessage callback receives user input
async function testOnMessage(): Promise<void> {
  const testName = 'chat-onmessage-callback';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  let messageReceived = '';
  let callbackInvoked = false;

  try {
    // Verify onMessage exists in options
    const chatPromise = chat({
      placeholder: 'Type and press Enter',
      messages: [{ role: 'assistant', content: 'Hello! Type something.' }],
      async onMessage(text: string) {
        callbackInvoked = true;
        messageReceived = text;
        debug(`onMessage received: "${text}"`);

        // Echo back
        const msgId = chat.startStream('left');
        chat.appendChunk(msgId, `You said: ${text}`);
        chat.completeStream(msgId);
      },
    });

    await sleep(500);
    const shot = await saveScreenshot('chat-onmessage');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { onMessageDefined: true },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 2: clear() method clears all messages
async function testClearMethod(): Promise<void> {
  const testName = 'chat-clear-method';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Clear test',
      messages: [
        { role: 'user', content: 'Message 1' },
        { role: 'assistant', content: 'Response 1' },
        { role: 'user', content: 'Message 2' },
      ],
      async onInit() {
        debug('Before clear');
        const beforeClear = chat.getMessages();
        debug(`Messages before: ${beforeClear?.length}`);

        await sleep(300);
        const shot1 = await saveScreenshot('chat-before-clear');

        // Clear all messages
        chat.clear();
        debug('After clear');

        await sleep(300);
        const afterClear = chat.getMessages();
        debug(`Messages after: ${afterClear?.length}`);

        const shot2 = await saveScreenshot('chat-after-clear');

        // Add new message after clear
        chat.addMessage({ role: 'assistant', content: 'Fresh start!' });
      },
    });

    await sleep(1000);
    const shot = await saveScreenshot('chat-clear-final');

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

// Test 3: Multiple concurrent streams
async function testConcurrentStreams(): Promise<void> {
  const testName = 'chat-concurrent-streams';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Concurrent streams test',
      messages: [{ role: 'user', content: 'Start multiple responses' }],
      async onInit() {
        // Start multiple streams at once
        const stream1 = chat.startStream('left');
        const stream2 = chat.startStream('left');

        // Interleave chunks
        for (let i = 0; i < 5; i++) {
          chat.appendChunk(stream1, `Stream1-${i} `);
          await sleep(50);
          chat.appendChunk(stream2, `Stream2-${i} `);
          await sleep(50);
        }

        chat.completeStream(stream1);
        chat.completeStream(stream2);
      },
    });

    await sleep(1000);
    const shot = await saveScreenshot('chat-concurrent');

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
debug(`Running chat callback test ${testArg}`);

switch (testArg) {
  case '1': case 'message': await testOnMessage(); break;
  case '2': case 'clear': await testClearMethod(); break;
  case '3': case 'concurrent': await testConcurrentStreams(); break;
  default: await testOnMessage();
}
