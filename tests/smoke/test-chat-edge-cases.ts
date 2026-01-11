// Test: chat() edge cases
// Verifies: Empty messages, unicode/emoji, long words, special characters

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

// Test 1: Unicode and emoji handling
async function testUnicodeEmoji(): Promise<void> {
  const testName = 'chat-unicode-emoji';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Unicode test 🎉',
      messages: [
        { role: 'user', content: 'Hello! 👋 How are you? 🤔' },
        { role: 'assistant', content: '我很好！ 안녕하세요! مرحبا 🎊✨🚀' },
        { role: 'user', content: 'Ümlauts: äöü ñ café résumé naïve' },
        { role: 'assistant', content: '数学: ∑∏∫√∞ π≈≠≤≥ Greek: αβγδ Cyrillic: привет' },
      ],
    });

    await sleep(500);
    const shot = await saveScreenshot('chat-unicode-emoji');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { hasEmoji: true, hasUnicode: true },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 2: Very long single word (no natural breaks)
async function testLongWord(): Promise<void> {
  const testName = 'chat-long-word';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const longWord = 'supercalifragilisticexpialidocious'.repeat(5);
    const longUrl = 'https://example.com/very/long/path/' + 'segment/'.repeat(20);

    const chatPromise = chat({
      placeholder: 'Long word test',
      messages: [
        { role: 'user', content: `Here's a long word: ${longWord}` },
        { role: 'assistant', content: `And a long URL: ${longUrl}` },
      ],
    });

    await sleep(500);
    const shot = await saveScreenshot('chat-long-word');

    log({
      test: testName,
      status: 'pass',
      timestamp: new Date().toISOString(),
      duration_ms: Date.now() - start,
      screenshot: shot,
      details: { longWordLength: longWord.length, longUrlLength: longUrl.length },
    });
    process.exit(0);
  } catch (error) {
    log({ test: testName, status: 'fail', timestamp: new Date().toISOString(), duration_ms: Date.now() - start, error: String(error) });
    process.exit(1);
  }
}

// Test 3: Special characters and escaping
async function testSpecialChars(): Promise<void> {
  const testName = 'chat-special-chars';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Special chars test',
      messages: [
        { role: 'user', content: 'HTML entities: <script>alert("xss")</script> &amp; &lt; &gt;' },
        { role: 'assistant', content: 'Quotes: "double" \'single\' `backtick` «guillemets»' },
        { role: 'user', content: 'Slashes: \\ / | backslash forward pipe' },
        { role: 'assistant', content: 'Brackets: [] {} () <> «» 「」' },
        { role: 'user', content: 'Newlines:\nLine 2\nLine 3\n\nDouble space above' },
      ],
    });

    await sleep(500);
    const shot = await saveScreenshot('chat-special-chars');

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

// Test 4: Empty and whitespace messages
async function testEmptyMessages(): Promise<void> {
  const testName = 'chat-empty-messages';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const chatPromise = chat({
      placeholder: 'Empty message test',
      messages: [
        { role: 'user', content: '' },
        { role: 'assistant', content: '   ' },
        { role: 'user', content: '\n\n\n' },
        { role: 'assistant', content: 'Normal message after empty ones' },
      ],
    });

    await sleep(500);
    const shot = await saveScreenshot('chat-empty-messages');

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

// Test 5: Complex nested markdown
async function testComplexMarkdown(): Promise<void> {
  const testName = 'chat-complex-markdown';
  const start = Date.now();
  log({ test: testName, status: 'running', timestamp: new Date().toISOString() });

  try {
    const complexMd = `# Heading 1
## Heading 2
### Heading 3

**Bold** *italic* ***bold italic*** ~~strikethrough~~

> Blockquote
> Multiple lines

1. Numbered list
2. Second item
   - Nested bullet
   - Another nested

| Table | Header |
|-------|--------|
| Cell  | Data   |

\`\`\`javascript
// Code with syntax highlighting
const x = () => {
  return { nested: { object: true } };
};
\`\`\`

[Link text](https://example.com)

---

Horizontal rule above`;

    const chatPromise = chat({
      placeholder: 'Complex markdown',
      messages: [
        { role: 'user', content: 'Show complex markdown' },
        { role: 'assistant', content: complexMd },
      ],
    });

    await sleep(600);
    const shot = await saveScreenshot('chat-complex-markdown');

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
debug(`Running chat edge case test ${testArg}`);

switch (testArg) {
  case '1': case 'unicode': await testUnicodeEmoji(); break;
  case '2': case 'longword': await testLongWord(); break;
  case '3': case 'special': await testSpecialChars(); break;
  case '4': case 'empty': await testEmptyMessages(); break;
  case '5': case 'markdown': await testComplexMarkdown(); break;
  default: await testUnicodeEmoji();
}
