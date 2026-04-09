// Name: SDK Test - AI Context Parts
// Description: Verifies typed AI context parts serialize correctly for aiStartChat and aiSendMessage

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
  console.log(JSON.stringify({
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra,
  }));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

const sentMessages: unknown[] = [];
const originalStdoutWrite = (process as any).stdout.write.bind((process as any).stdout);
(process as any).stdout.write = (chunk: any, ...args: any[]) => {
  try {
    const parsed = JSON.parse(chunk.toString().trim());
    sentMessages.push(parsed);
  } catch {
    // pass through
  }
  return originalStdoutWrite(chunk, ...args);
};

async function expectSentMessage(
  invoke: () => Promise<unknown>,
  type: string,
  validate?: (msg: any) => void,
): Promise<any> {
  sentMessages.length = 0;
  await invoke();
  const msg = sentMessages.find((m: any) => m.type === type) as any;
  if (!msg) throw new Error(`${type} message not found`);
  validate?.(msg);
  return msg;
}

async function runTests() {
  {
    const test = 'ai-context-parts-start-chat-mixed-parts';
    const start = Date.now();
    logTest(test, 'running');
    try {
      const msg = await expectSentMessage(
        () =>
          aiStartChat('Review these', {
            noResponse: true,
            parts: [
              { kind: 'resourceUri', uri: 'kit://context?profile=minimal', label: 'Current Context' },
              { kind: 'filePath', path: '/tmp/example.rs', label: 'example.rs' },
            ],
          } as any),
        'aiStartChat',
        (m) => {
          if (!Array.isArray(m.parts) || m.parts.length !== 2) {
            throw new Error(`expected 2 parts, got ${JSON.stringify(m.parts)}`);
          }
          if (m.parts[0].kind !== 'resourceUri' || m.parts[1].kind !== 'filePath') {
            throw new Error(`unexpected part ordering: ${JSON.stringify(m.parts)}`);
          }
        },
      );
      debug(`aiStartChat mixed parts payload: ${JSON.stringify(msg.parts)}`);
      logTest(test, 'pass', { result: msg.parts, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  {
    const test = 'ai-context-parts-send-message-mixed-parts';
    const start = Date.now();
    logTest(test, 'running');
    try {
      const msg = await expectSentMessage(
        () =>
          (aiSendMessage as any)(
            'chat-123',
            'Follow up',
            undefined,
            [
              { kind: 'filePath', path: '/tmp/a.md', label: 'a.md' },
              {
                kind: 'resourceUri',
                uri: 'kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0',
                label: 'Selected Text',
              },
            ],
          ),
        'aiSendMessage',
        (m) => {
          if (!Array.isArray(m.parts) || m.parts.length !== 2) {
            throw new Error(`expected 2 parts, got ${JSON.stringify(m.parts)}`);
          }
          if (m.parts[0].kind !== 'filePath' || m.parts[1].kind !== 'resourceUri') {
            throw new Error(`unexpected part ordering: ${JSON.stringify(m.parts)}`);
          }
        },
      );
      debug(`aiSendMessage mixed parts payload: ${JSON.stringify(msg.parts)}`);
      logTest(test, 'pass', { result: msg.parts, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  {
    const test = 'ai-context-parts-omitted-when-not-provided';
    const start = Date.now();
    logTest(test, 'running');
    try {
      const msg = await expectSentMessage(
        () => aiStartChat('Hello'),
        'aiStartChat',
        (m) => {
          if (Array.isArray(m.parts) && m.parts.length > 0) {
            throw new Error(`unexpected parts payload: ${JSON.stringify(m.parts)}`);
          }
        },
      );
      debug(`aiStartChat without parts payload: ${JSON.stringify(msg)}`);
      logTest(test, 'pass', { result: msg, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  debug('AI context parts SDK tests complete');
  process.exit(0);
}

runTests();
