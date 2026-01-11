// Test: chat prompt visual test
// Verifies: chat prompt renders, messages display correctly, streaming works

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
}

function log(result: TestResult): void {
  console.log(JSON.stringify(result));
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

async function runTests(): Promise<void> {
  const testName = 'chat-prompt-visual';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    // Test 1: Open chat with initial messages
    console.error('[TEST] Opening chat prompt with initial messages');

    // We need to run chat in a non-blocking way for testing
    // Use onMessage to handle submissions
    const chatPromise = chat({
      placeholder: 'Type a message...',
      hint: 'Press Enter to send',
      footer: 'Test Chat Session',
      messages: [
        { text: 'Hello! I am an assistant.', position: 'left', name: 'Assistant' },
        { text: 'Hi there! Can you help me?', position: 'right', name: 'User' },
      ],
      onMessage: async (text: string) => {
        console.error(`[TEST] User sent: ${text}`);
        // Echo back with streaming simulation
        const streamId = chat.startStream('left');
        const words = `I received your message: "${text}". Let me think about that...`.split(' ');
        for (const word of words) {
          await sleep(50);
          chat.appendChunk(streamId, word + ' ');
        }
        chat.completeStream(streamId);
      },
    });

    // Wait for UI to render
    await sleep(800);

    // Screenshot 1: Initial state with messages
    const shot1 = await saveScreenshot('chat-initial');
    console.error('[TEST] Screenshot 1: Initial chat with messages');

    // Test 2: Add a new message programmatically
    console.error('[TEST] Adding message programmatically');
    chat.addMessage({
      text: 'This is a programmatically added message.',
      position: 'left',
      name: 'System',
    });
    await sleep(300);

    // Screenshot 2: After adding message
    const shot2 = await saveScreenshot('chat-added-message');
    console.error('[TEST] Screenshot 2: After adding message');

    // Test 3: Test streaming
    console.error('[TEST] Testing streaming response');
    const streamId = chat.startStream('left');
    const streamText = 'This is a streaming response that appears word by word...';
    const words = streamText.split(' ');
    for (const word of words) {
      chat.appendChunk(streamId, word + ' ');
      await sleep(80);
    }
    chat.completeStream(streamId);
    await sleep(300);

    // Screenshot 3: After streaming completes
    const shot3 = await saveScreenshot('chat-streaming-complete');
    console.error('[TEST] Screenshot 3: After streaming completes');

    // Test 4: Add right-aligned (user) message
    console.error('[TEST] Adding user message');
    chat.addMessage({
      text: 'This is a user message on the right side.',
      position: 'right',
      name: 'Test User',
    });
    await sleep(300);

    // Screenshot 4: After user message
    const shot4 = await saveScreenshot('chat-user-message');
    console.error('[TEST] Screenshot 4: After user message');

    // Test 5: Clear and verify
    console.error('[TEST] Clearing messages');
    chat.clear();
    await sleep(300);

    // Screenshot 5: After clear
    const shot5 = await saveScreenshot('chat-cleared');
    console.error('[TEST] Screenshot 5: After clear');

    // Add messages again after clear to verify clear worked
    chat.addMessage({
      text: 'Chat was cleared! This is a fresh start.',
      position: 'left',
    });
    await sleep(300);

    // Final screenshot
    const shot6 = await saveScreenshot('chat-after-clear');
    console.error('[TEST] Screenshot 6: After clear with new message');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot6,
    });

    console.error('[TEST] All chat prompt tests completed successfully');

    // Exit after tests
    process.exit(0);
  } catch (error) {
    log({
      test: testName,
      status: 'fail',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      error: String(error),
    });
    console.error(`[TEST ERROR] ${error}`);
    process.exit(1);
  }
}

// Run tests
runTests();
