#!/usr/bin/env bun
// Test Runner: Runs all chat smoke tests
// Usage: bun run tests/smoke/run-chat-tests.ts

import { spawn } from 'child_process';
import { join } from 'path';

interface TestCase {
  file: string;
  name: string;
  args: string[];
}

const projectRoot = join(import.meta.dir, '../..');
const binary = join(projectRoot, 'target/debug/script-kit-gpui');

// All chat test cases
const testCases: TestCase[] = [
  // test-chat-oninit.ts
  { file: 'test-chat-oninit.ts', name: 'onInit-called', args: ['1'] },
  { file: 'test-chat-oninit.ts', name: 'onInit-streaming', args: ['2'] },
  { file: 'test-chat-oninit.ts', name: 'onInit-error', args: ['3'] },

  // test-chat-errors.ts
  { file: 'test-chat-errors.ts', name: 'setError', args: ['1'] },
  { file: 'test-chat-errors.ts', name: 'clearError', args: ['2'] },
  { file: 'test-chat-errors.ts', name: 'multipleErrors', args: ['3'] },
  { file: 'test-chat-errors.ts', name: 'errorRecovery', args: ['4'] },

  // test-chat-ai-sdk-compat.ts
  { file: 'test-chat-ai-sdk-compat.ts', name: 'coreMessage', args: ['1'] },
  { file: 'test-chat-ai-sdk-compat.ts', name: 'systemPrompt', args: ['2'] },
  { file: 'test-chat-ai-sdk-compat.ts', name: 'getResult', args: ['3'] },
  { file: 'test-chat-ai-sdk-compat.ts', name: 'mixedFormats', args: ['4'] },
  { file: 'test-chat-ai-sdk-compat.ts', name: 'vercelCompat', args: ['5'] },

  // test-chat-visual-layout.ts
  { file: 'test-chat-visual-layout.ts', name: 'basicLayout', args: ['1'] },
  { file: 'test-chat-visual-layout.ts', name: 'positioning', args: ['2'] },
  { file: 'test-chat-visual-layout.ts', name: 'longMessage', args: ['3'] },
  { file: 'test-chat-visual-layout.ts', name: 'emptyState', args: ['4'] },

  // test-chat-visual-content.ts
  { file: 'test-chat-visual-content.ts', name: 'markdown', args: ['1'] },
  { file: 'test-chat-visual-content.ts', name: 'streaming', args: ['2'] },
  { file: 'test-chat-visual-content.ts', name: 'manyMessages', args: ['3'] },
  { file: 'test-chat-visual-content.ts', name: 'fullConversation', args: ['4'] },

  // test-chat-callbacks.ts
  { file: 'test-chat-callbacks.ts', name: 'onMessage', args: ['1'] },
  { file: 'test-chat-callbacks.ts', name: 'clearMethod', args: ['2'] },
  { file: 'test-chat-callbacks.ts', name: 'concurrentStreams', args: ['3'] },

  // test-chat-edge-cases.ts
  { file: 'test-chat-edge-cases.ts', name: 'unicodeEmoji', args: ['1'] },
  { file: 'test-chat-edge-cases.ts', name: 'longWord', args: ['2'] },
  { file: 'test-chat-edge-cases.ts', name: 'specialChars', args: ['3'] },
  { file: 'test-chat-edge-cases.ts', name: 'emptyMessages', args: ['4'] },
  { file: 'test-chat-edge-cases.ts', name: 'complexMarkdown', args: ['5'] },

  // Original test
  { file: 'test-chat-prompt.ts', name: 'originalChatPrompt', args: [] },
];

interface TestResult {
  name: string;
  passed: boolean;
  duration: number;
  error?: string;
}

async function runTest(test: TestCase): Promise<TestResult> {
  const testPath = join(import.meta.dir, test.file);
  const startTime = Date.now();

  return new Promise((resolve) => {
    const command = JSON.stringify({
      type: 'run',
      path: testPath,
      args: test.args,
    });

    const proc = spawn(binary, [], {
      env: { ...process.env, SCRIPT_KIT_AI_LOG: '1' },
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';
    let timedOut = false;

    const timeout = setTimeout(() => {
      timedOut = true;
      proc.kill('SIGTERM');
    }, 30000); // 30 second timeout

    proc.stdout.on('data', (data) => { stdout += data.toString(); });
    proc.stderr.on('data', (data) => { stderr += data.toString(); });

    proc.stdin.write(command + '\n');
    proc.stdin.end();

    proc.on('close', (code) => {
      clearTimeout(timeout);
      const duration = Date.now() - startTime;

      // Check for pass in output
      const passed = !timedOut && (
        stderr.includes('"status":"pass"') ||
        stdout.includes('"status":"pass"') ||
        code === 0
      );

      resolve({
        name: `${test.file}:${test.name}`,
        passed,
        duration,
        error: timedOut ? 'Timeout' : (!passed ? 'Test failed' : undefined),
      });
    });
  });
}

async function main() {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║           Chat Feature Smoke Test Suite                     ║');
  console.log('╚════════════════════════════════════════════════════════════╝\n');

  const results: TestResult[] = [];
  let passed = 0;
  let failed = 0;

  for (const test of testCases) {
    process.stdout.write(`  Running ${test.file}:${test.name}...`);
    const result = await runTest(test);
    results.push(result);

    if (result.passed) {
      passed++;
      console.log(` ✓ (${result.duration}ms)`);
    } else {
      failed++;
      console.log(` ✗ (${result.error})`);
    }
  }

  console.log('\n────────────────────────────────────────────────────────────');
  console.log(`  Total: ${testCases.length} | Passed: ${passed} | Failed: ${failed}`);
  console.log('────────────────────────────────────────────────────────────\n');

  if (failed > 0) {
    console.log('Failed tests:');
    for (const result of results) {
      if (!result.passed) {
        console.log(`  - ${result.name}: ${result.error}`);
      }
    }
    process.exit(1);
  }

  console.log('All tests passed!');
  process.exit(0);
}

main().catch(console.error);
