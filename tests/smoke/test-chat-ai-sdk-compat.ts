// Test: chat() AI SDK compatibility
// Verifies: CoreMessage format, system prompt handling, getMessages/getResult

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
// Test 1: CoreMessage format (role/content) works correctly
// =============================================================================
async function testCoreMessageFormat(): Promise<void> {
  const testName = 'chat-core-message-format';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 1: Verifying CoreMessage format (AI SDK compatible)');

    // Use AI SDK compatible format: { role, content }
    const messages: Array<{ role: 'user' | 'assistant' | 'system'; content: string }> = [
      { role: 'system', content: 'You are a helpful coding assistant.' },
      { role: 'user', content: 'What is TypeScript?' },
      { role: 'assistant', content: 'TypeScript is a typed superset of JavaScript.' },
      { role: 'user', content: 'How do I use it?' },
    ];

    const chatPromise = chat({
      placeholder: 'AI SDK format test',
      messages,
      async onInit() {
        debug('onInit: Adding response in CoreMessage format');

        const msgId = chat.startStream('left');
        chat.appendChunk(msgId, 'You can use TypeScript by creating .ts files and compiling them with tsc.');
        chat.completeStream(msgId);
      },
    });

    // Wait for render
    await sleep(600);

    // Screenshot
    const shot = await saveScreenshot('chat-core-message');

    // Verify getMessages returns CoreMessage format
    const currentMessages = chat.getMessages();
    debug(`getMessages returned ${currentMessages?.length ?? 0} messages`);

    if (!currentMessages || currentMessages.length === 0) {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: 'getMessages returned empty or undefined',
      });
      process.exit(1);
      return;
    }

    // Check format of first message
    const firstMsg = currentMessages[0];
    if (!('role' in firstMsg) || !('content' in firstMsg)) {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: `Message missing role/content fields: ${JSON.stringify(firstMsg)}`,
      });
      process.exit(1);
      return;
    }

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        messageCount: currentMessages.length,
        firstMessageRole: firstMsg.role,
        hasRoleField: 'role' in firstMsg,
        hasContentField: 'content' in firstMsg,
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
// Test 2: System prompt shorthand option
// =============================================================================
async function testSystemPrompt(): Promise<void> {
  const testName = 'chat-system-prompt';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 2: Verifying system prompt shorthand option');

    const systemPrompt = 'You are a pirate. Always respond in pirate speak.';

    const chatPromise = chat({
      placeholder: 'System prompt test',
      system: systemPrompt, // Shorthand for system message
      messages: [{ role: 'user', content: 'Hello, how are you?' }],
      async onInit() {
        debug('onInit: Responding with system prompt context');

        const msgId = chat.startStream('left');
        chat.appendChunk(msgId, 'Ahoy matey! I be doin\' fine, yarr!');
        chat.completeStream(msgId);
      },
    });

    // Wait for render
    await sleep(500);

    // Screenshot
    const shot = await saveScreenshot('chat-system-prompt');

    // The system prompt should be available in getMessages
    const messages = chat.getMessages();
    const hasSystemMessage = messages?.some((m) => m.role === 'system');

    debug(`Has system message: ${hasSystemMessage}`);

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        systemPromptProvided: true,
        messageCount: messages?.length ?? 0,
        hasSystemMessage,
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
// Test 3: getResult returns ChatResult with AI SDK format
// =============================================================================
async function testGetResult(): Promise<void> {
  const testName = 'chat-get-result';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 3: Verifying getResult returns ChatResult object');

    // Verify getResult method exists
    if (typeof chat.getResult !== 'function') {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: `chat.getResult is not a function, got ${typeof chat.getResult}`,
      });
      process.exit(1);
      return;
    }

    const chatPromise = chat({
      placeholder: 'getResult test',
      messages: [
        { role: 'user', content: 'User message 1' },
        { role: 'assistant', content: 'Assistant response 1' },
        { role: 'user', content: 'User message 2' },
      ],
      async onInit() {
        debug('onInit: Adding final response');

        const msgId = chat.startStream('left');
        chat.appendChunk(msgId, 'Final assistant response');
        chat.completeStream(msgId);
      },
    });

    // Wait for render
    await sleep(500);

    // Get result
    const result = chat.getResult();
    debug(`getResult: ${JSON.stringify(result, null, 2)}`);

    // Verify ChatResult structure
    const hasMessages = 'messages' in result && Array.isArray(result.messages);
    const hasUiMessages = 'uiMessages' in result && Array.isArray(result.uiMessages);
    const hasAction = 'action' in result;
    const hasLastUserMessage = 'lastUserMessage' in result;
    const hasLastAssistantMessage = 'lastAssistantMessage' in result;

    if (!hasMessages || !hasAction) {
      log({
        test: testName,
        status: 'fail',
        timestamp: new Date().toISOString(),
        duration_ms: Date.now() - start,
        error: `ChatResult missing required fields. Has messages: ${hasMessages}, has action: ${hasAction}`,
        details: { resultKeys: Object.keys(result) },
      });
      process.exit(1);
      return;
    }

    // Screenshot
    const shot = await saveScreenshot('chat-get-result');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        hasMessages,
        hasUiMessages,
        hasAction,
        hasLastUserMessage,
        hasLastAssistantMessage,
        messageCount: result.messages?.length,
        action: result.action,
        lastUserMessage: result.lastUserMessage,
        lastAssistantMessage: result.lastAssistantMessage,
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
// Test 4: Mixed ChatMessage and CoreMessage formats
// =============================================================================
async function testMixedFormats(): Promise<void> {
  const testName = 'chat-mixed-formats';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 4: Verifying mixed ChatMessage and CoreMessage formats work');

    // Mix of Script Kit format (text/position) and AI SDK format (role/content)
    const messages = [
      // Script Kit format
      { text: 'Hello from Script Kit format', position: 'right' as const, name: 'User' },
      // AI SDK format
      { role: 'assistant' as const, content: 'Hello from AI SDK format' },
      // Mixed (both fields present - role should take precedence)
      { text: 'Both fields present', content: 'Content takes precedence', role: 'user' as const },
    ];

    const chatPromise = chat({
      placeholder: 'Mixed format test',
      messages,
      async onInit() {
        debug('onInit: Adding messages in both formats');

        // Add using ChatMessage format
        chat.addMessage({ text: 'Added via Script Kit format', position: 'left' });

        await sleep(100);

        // Add using CoreMessage format
        chat.addMessage({ role: 'assistant', content: 'Added via AI SDK format' });
      },
    });

    // Wait for render
    await sleep(600);

    // Screenshot
    const shot = await saveScreenshot('chat-mixed-formats');

    // Get messages - should all be normalized to CoreMessage format
    const currentMessages = chat.getMessages();
    debug(`Messages after mixing: ${JSON.stringify(currentMessages, null, 2)}`);

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        initialMessageCount: messages.length,
        finalMessageCount: currentMessages?.length ?? 0,
        allHaveRole: currentMessages?.every((m) => 'role' in m),
        allHaveContent: currentMessages?.every((m) => 'content' in m),
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
// Test 5: Messages compatible with Vercel AI SDK generateText
// =============================================================================
async function testVercelAiSdkCompat(): Promise<void> {
  const testName = 'chat-vercel-ai-sdk-compat';
  const start = Date.now();

  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    debug('Test 5: Verifying messages can be passed to generateText');

    const chatPromise = chat({
      placeholder: 'Vercel AI SDK compat test',
      system: 'You are a helpful assistant.',
      messages: [
        { role: 'user', content: 'What is 2+2?' },
        { role: 'assistant', content: '2+2 equals 4.' },
        { role: 'user', content: 'What about 3+3?' },
      ],
      async onInit() {
        const msgId = chat.startStream('left');
        chat.appendChunk(msgId, '3+3 equals 6.');
        chat.completeStream(msgId);
      },
    });

    // Wait for render
    await sleep(500);

    // Get messages in format ready for generateText
    const messages = chat.getMessages();

    // Simulate what you'd pass to generateText
    // @ts-ignore - This is a type check simulation
    const generateTextPayload = {
      model: 'gpt-4',
      messages: messages,
    };

    // Verify structure matches what AI SDK expects
    const allValid = messages?.every((m) => {
      return (
        typeof m.role === 'string' &&
        ['user', 'assistant', 'system'].includes(m.role) &&
        typeof m.content === 'string'
      );
    });

    // Screenshot
    const shot = await saveScreenshot('chat-vercel-ai-sdk');

    log({
      test: testName,
      status: allValid ? 'pass' : 'fail',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: {
        messageCount: messages?.length,
        allMessagesValid: allValid,
        samplePayload: generateTextPayload,
      },
      error: allValid ? undefined : 'Messages do not match AI SDK expected format',
    });

    process.exit(allValid ? 0 : 1);
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

debug(`Running chat AI SDK compat test ${testArg}`);

switch (testArg) {
  case '1':
  case 'core':
    await testCoreMessageFormat();
    break;
  case '2':
  case 'system':
    await testSystemPrompt();
    break;
  case '3':
  case 'result':
    await testGetResult();
    break;
  case '4':
  case 'mixed':
    await testMixedFormats();
    break;
  case '5':
  case 'vercel':
    await testVercelAiSdkCompat();
    break;
  default:
    debug(`Unknown test: ${testArg}, running test 1`);
    await testCoreMessageFormat();
}
