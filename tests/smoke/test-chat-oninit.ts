// Test: chat() onInit callback pattern
// Verifies: The pattern used by AI text tools where onInit starts an async stream
// This is CRITICAL for the inline chat AI workflow

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

// =============================================================================
// Test 1: onInit callback is called on chat open
// =============================================================================
async function testOnInitCalled(): Promise<void> {
  const testName = 'chat-oninit-called';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  let onInitCalled = false;
  let onInitTimestamp = 0;

  try {
    debug('Test 1: Verifying onInit is called when chat opens');

    // Start chat with onInit - it should be called immediately
    const chatPromise = chat({
      placeholder: 'onInit Test',
      messages: [{ role: 'user', content: 'Hello, test message' }],
      async onInit() {
        onInitCalled = true;
        onInitTimestamp = Date.now();
        debug(`onInit called at ${onInitTimestamp}`);

        // Simulate async work (like fetching AI response)
        await sleep(100);

        // Add a response message
        const msgId = chat.startStream('left');
        chat.appendChunk(msgId, 'onInit was called successfully!');
        chat.completeStream(msgId);
      },
    });

    // Wait for UI to render and onInit to complete
    await sleep(500);

    // Verify onInit was called
    if (!onInitCalled) {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: 'onInit callback was NOT called',
      });
      return;
    }

    // Screenshot to verify the streamed message appeared
    const shot = await saveScreenshot('chat-oninit-called');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        onInitCalled,
        onInitTimestamp,
        delayFromStart: onInitTimestamp - start,
      },
    });

    // Exit the chat
    process.exit(0);
  } catch (error) {
    log({
      test: testName,
      status: 'fail',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      error: String(error),
    });
    process.exit(1);
  }
}

// =============================================================================
// Test 2: onInit with streaming (AI text tools pattern)
// =============================================================================
async function testOnInitStreaming(): Promise<void> {
  const testName = 'chat-oninit-streaming';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  const chunks: string[] = [];
  let streamCompleted = false;

  try {
    debug('Test 2: Testing onInit with streaming (AI text tools pattern)');

    const systemPrompt = 'You are a helpful assistant.';
    const userPrompt = 'Please explain what testing means.';

    const chatPromise = chat({
      placeholder: 'Ask follow-up...',
      messages: [{ role: 'user', content: userPrompt }],
      system: systemPrompt,
      async onInit() {
        debug('onInit: Starting stream simulation');

        // This simulates what the AI text tools do
        const msgId = chat.startStream('left');

        // Simulate streaming response
        const response = 'Testing is the process of verifying that software works correctly. It involves running code with various inputs and checking outputs.';
        const words = response.split(' ');

        for (const word of words) {
          chunks.push(word);
          chat.appendChunk(msgId, word + ' ');
          await sleep(30);
        }

        chat.completeStream(msgId);
        streamCompleted = true;
        debug(`onInit: Stream completed with ${chunks.length} chunks`);
      },
    });

    // Wait for streaming to complete
    await sleep(2000);

    // Verify streaming worked
    if (chunks.length === 0) {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: 'No chunks were streamed',
      });
      return;
    }

    if (!streamCompleted) {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: 'Stream was not completed',
      });
      return;
    }

    // Screenshot to verify the streamed content
    const shot = await saveScreenshot('chat-oninit-streaming');

    // Get layout info to verify message bubble position
    const layout = await getLayoutInfo();

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        chunkCount: chunks.length,
        streamCompleted,
        layoutComponentCount: layout.components?.length ?? 0,
        promptType: layout.promptType,
      },
    });

    process.exit(0);
  } catch (error) {
    log({
      test: testName,
      status: 'fail',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      error: String(error),
    });
    process.exit(1);
  }
}

// =============================================================================
// Test 3: onInit error handling
// =============================================================================
async function testOnInitError(): Promise<void> {
  const testName = 'chat-oninit-error';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  let errorHandled = false;

  try {
    debug('Test 3: Testing onInit error handling');

    const chatPromise = chat({
      placeholder: 'Error handling test',
      messages: [{ role: 'user', content: 'This will trigger an error' }],
      async onInit() {
        debug('onInit: Starting with error simulation');

        const msgId = chat.startStream('left');

        // Simulate partial stream then error (like network failure)
        chat.appendChunk(msgId, 'Starting response... ');
        await sleep(100);

        // Use setError to display error state
        chat.setError(msgId, 'Network connection failed');
        errorHandled = true;

        debug('onInit: Error state set on message');
      },
    });

    // Wait for error to be displayed
    await sleep(500);

    // Screenshot to verify error state
    const shot = await saveScreenshot('chat-oninit-error');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { errorHandled },
    });

    process.exit(0);
  } catch (error) {
    log({
      test: testName,
      status: 'fail',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      error: String(error),
    });
    process.exit(1);
  }
}

// =============================================================================
// Run the test specified by command line arg or default to test 1
// =============================================================================
const testArg = process.argv[2] || '1';

debug(`Running chat onInit test ${testArg}`);

switch (testArg) {
  case '1':
  case 'called':
    await testOnInitCalled();
    break;
  case '2':
  case 'streaming':
    await testOnInitStreaming();
    break;
  case '3':
  case 'error':
    await testOnInitError();
    break;
  default:
    debug(`Unknown test: ${testArg}, running test 1`);
    await testOnInitCalled();
}
