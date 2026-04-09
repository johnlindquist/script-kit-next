// Name: SDK Test - ACP SDK
// Description: Verifies ACP SDK message shapes for aiStartChat, aiAppendMessage, aiSendMessage, and aiOn

import '../../scripts/kit-sdk';

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra,
  };
  console.log(JSON.stringify(result));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

// Capture messages sent to stdout
const sentMessages: unknown[] = [];
const originalStdoutWrite = (process as any).stdout.write.bind((process as any).stdout);
(process as any).stdout.write = (chunk: any, ...args: any[]) => {
  try {
    const parsed = JSON.parse(chunk.toString().trim());
    sentMessages.push(parsed);
  } catch {
    // Not JSON, pass through
  }
  return originalStdoutWrite(chunk, ...args);
};

async function runTests() {
  // =============================================================================
  // Test 1: aiStartChat carries parts
  // =============================================================================
  {
    const test = 'acp-start-chat-includes-parts';
    const start = Date.now();
    logTest(test, 'running');
    try {
      sentMessages.length = 0;
      await aiStartChat('Review this context', {
        noResponse: true,
        parts: [
          { kind: 'resourceUri', uri: 'kit://context?profile=minimal', label: 'Current Context' },
        ],
      } as any);

      const msg = sentMessages.find((m: any) => m.type === 'aiStartChat') as any;
      if (!msg) throw new Error('aiStartChat message not found');
      if (!Array.isArray(msg.parts) || msg.parts.length !== 1) {
        throw new Error(`expected 1 part, got ${JSON.stringify(msg.parts)}`);
      }
      if (msg.parts[0].kind !== 'resourceUri') {
        throw new Error(`wrong part kind: ${msg.parts[0].kind}`);
      }

      debug(`aiStartChat parts payload: ${JSON.stringify(msg.parts)}`);
      logTest(test, 'pass', { result: msg.parts, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  // =============================================================================
  // Test 2: aiAppendMessage shape
  // =============================================================================
  {
    const test = 'acp-append-message-shape';
    const start = Date.now();
    logTest(test, 'running');
    try {
      sentMessages.length = 0;
      await aiAppendMessage('chat-123', 'Seed assistant reply', 'assistant');

      const msg = sentMessages.find((m: any) => m.type === 'aiAppendMessage') as any;
      if (!msg) throw new Error('aiAppendMessage message not found');
      if (msg.chatId !== 'chat-123' || msg.role !== 'assistant') {
        throw new Error(`bad aiAppendMessage payload: ${JSON.stringify(msg)}`);
      }

      debug(`aiAppendMessage payload: ${JSON.stringify(msg)}`);
      logTest(test, 'pass', { result: msg, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  // =============================================================================
  // Test 3: aiSendMessage carries parts
  // =============================================================================
  {
    const test = 'acp-send-message-includes-parts';
    const start = Date.now();
    logTest(test, 'running');
    try {
      sentMessages.length = 0;
      await (aiSendMessage as any)(
        'chat-123',
        'Now inspect this file',
        undefined,
        [{ kind: 'filePath', path: '/tmp/example.rs', label: 'example.rs' }],
      );

      const msg = sentMessages.find((m: any) => m.type === 'aiSendMessage') as any;
      if (!msg) throw new Error('aiSendMessage message not found');
      if (!Array.isArray(msg.parts) || msg.parts[0]?.kind !== 'filePath') {
        throw new Error(`bad aiSendMessage parts: ${JSON.stringify(msg.parts)}`);
      }

      debug(`aiSendMessage parts payload: ${JSON.stringify(msg.parts)}`);
      logTest(test, 'pass', { result: msg.parts, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  // =============================================================================
  // Test 4: aiOn sends subscribe message
  // =============================================================================
  {
    const test = 'acp-aiOn-sends-subscribe';
    const start = Date.now();
    logTest(test, 'running');
    try {
      sentMessages.length = 0;
      await aiOn('streamChunk' as any, () => {}, 'chat-123');

      const msg = sentMessages.find((m: any) => m.type === 'aiSubscribe') as any;
      if (!msg) throw new Error('aiSubscribe message not found');
      if (msg.chatId !== 'chat-123') throw new Error(`wrong chatId: ${msg.chatId}`);
      if (!Array.isArray(msg.events) || msg.events[0] !== 'streamChunk') {
        throw new Error(`wrong events: ${JSON.stringify(msg.events)}`);
      }

      debug(`aiOn subscribe payload: ${JSON.stringify(msg)}`);
      logTest(test, 'pass', { result: msg, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  debug('ACP SDK tests complete');
  process.exit(0);
}

runTests();
