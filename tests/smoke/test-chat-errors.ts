// Test: chat() error handling
// Verifies: setError, clearError methods for displaying error states in chat messages

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
// Test 1: setError displays error state on a message
// =============================================================================
async function testSetError(): Promise<void> {
  const testName = 'chat-set-error';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 1: Verifying setError displays error state');

    // Verify setError method exists
    if (typeof chat.setError !== 'function') {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: `chat.setError is not a function, got ${typeof chat.setError}`,
      });
      process.exit(1);
      return;
    }

    const chatPromise = chat({
      placeholder: 'Error test',
      messages: [{ role: 'user', content: 'Please respond' }],
      async onInit() {
        debug('onInit: Starting stream that will error');

        const msgId = chat.startStream('left');

        // Partial response
        chat.appendChunk(msgId, 'I am starting to respond... ');
        await sleep(200);

        // Simulate error
        chat.setError(msgId, 'API Error: Rate limit exceeded');
        debug(`onInit: setError called on message ${msgId}`);
      },
    });

    // Wait for error to render
    await sleep(600);

    // Screenshot to verify error state
    const shot = await saveScreenshot('chat-set-error');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { setErrorExists: true },
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
// Test 2: clearError removes error state
// =============================================================================
async function testClearError(): Promise<void> {
  const testName = 'chat-clear-error';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 2: Verifying clearError removes error state');

    // Verify clearError method exists
    if (typeof chat.clearError !== 'function') {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: `chat.clearError is not a function, got ${typeof chat.clearError}`,
      });
      process.exit(1);
      return;
    }

    let messageId = '';

    const chatPromise = chat({
      placeholder: 'Clear error test',
      messages: [{ role: 'user', content: 'Testing error recovery' }],
      async onInit() {
        debug('onInit: Creating message with error then clearing');

        messageId = chat.startStream('left');

        // Show partial response
        chat.appendChunk(messageId, 'Processing... ');
        await sleep(200);

        // Set error
        chat.setError(messageId, 'Temporary network error');
        debug('Error state set');
        await sleep(400);

        // Clear error (simulating retry)
        chat.clearError(messageId);
        debug('Error state cleared');

        // Continue with response
        chat.appendChunk(messageId, 'Recovered! Continuing with response.');
        chat.completeStream(messageId);
      },
    });

    // Wait for all operations
    await sleep(1000);

    // Screenshot to verify error was cleared
    const shot = await saveScreenshot('chat-clear-error');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { clearErrorExists: true, messageId },
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
// Test 3: Multiple error states in different messages
// =============================================================================
async function testMultipleErrors(): Promise<void> {
  const testName = 'chat-multiple-errors';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 3: Testing multiple messages with different error states');

    const chatPromise = chat({
      placeholder: 'Multiple errors test',
      messages: [
        { role: 'user', content: 'First request' },
        { role: 'user', content: 'Second request' },
      ],
      async onInit() {
        debug('onInit: Creating multiple responses with varying states');

        // First response - error
        const msg1 = chat.startStream('left');
        chat.appendChunk(msg1, 'First response attempt... ');
        await sleep(100);
        chat.setError(msg1, 'Failed: Timeout');

        // Second response - success
        const msg2 = chat.startStream('left');
        chat.appendChunk(msg2, 'Second response succeeded!');
        chat.completeStream(msg2);
        await sleep(100);

        // Third response - error with different message
        const msg3 = chat.startStream('left');
        chat.appendChunk(msg3, 'Third response... ');
        await sleep(100);
        chat.setError(msg3, 'Error: Invalid API key');

        debug('All messages created with varying states');
      },
    });

    // Wait for all messages
    await sleep(800);

    // Screenshot to verify multiple error states
    const shot = await saveScreenshot('chat-multiple-errors');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { messageCount: 3 },
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
// Test 4: Error recovery with retry pattern
// =============================================================================
async function testErrorRecovery(): Promise<void> {
  const testName = 'chat-error-recovery';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 4: Testing error recovery pattern (error -> clear -> retry)');

    let retryCount = 0;
    const maxRetries = 2;

    const chatPromise = chat({
      placeholder: 'Error recovery test',
      messages: [{ role: 'user', content: 'Please respond (with retries)' }],
      async onInit() {
        debug('onInit: Simulating retry logic');

        const msgId = chat.startStream('left');

        // Simulate retries
        while (retryCount < maxRetries) {
          retryCount++;
          debug(`Attempt ${retryCount}/${maxRetries}`);

          chat.appendChunk(msgId, `Attempt ${retryCount}... `);
          await sleep(200);

          if (retryCount < maxRetries) {
            // Fail and retry
            chat.setError(msgId, `Retry ${retryCount}/${maxRetries}: Connection failed`);
            await sleep(300);
            chat.clearError(msgId);
          }
        }

        // Final success
        chat.appendChunk(msgId, 'Success on final attempt!');
        chat.completeStream(msgId);
        debug('Recovery complete');
      },
    });

    // Wait for all retries
    await sleep(1500);

    // Screenshot
    const shot = await saveScreenshot('chat-error-recovery');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { retryCount, maxRetries },
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
// Run the test specified by command line arg
// =============================================================================
const testArg = process.argv[2] || '1';

debug(`Running chat error test ${testArg}`);

switch (testArg) {
  case '1':
  case 'set':
    await testSetError();
    break;
  case '2':
  case 'clear':
    await testClearError();
    break;
  case '3':
  case 'multiple':
    await testMultipleErrors();
    break;
  case '4':
  case 'recovery':
    await testErrorRecovery();
    break;
  default:
    debug(`Unknown test: ${testArg}, running test 1`);
    await testSetError();
}
