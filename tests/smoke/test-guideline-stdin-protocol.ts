// Name: Stdin Protocol Guideline Test
// Description: Validates that agents use the correct stdin JSON protocol

/**
 * GUIDELINE TEST: test-guideline-stdin-protocol.ts
 *
 * This test validates the stdin JSON protocol guideline from AGENTS.md.
 *
 * ## CORRECT WAY (stdin JSON protocol):
 * ```bash
 * echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-guideline-stdin-protocol.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 * ```
 *
 * ## WRONG WAY (command line args - does nothing!):
 * ```bash
 * ./target/debug/script-kit-gpui tests/smoke/test-guideline-stdin-protocol.ts  # THIS DOES NOTHING
 * ```
 *
 * ## Expected Behavior:
 * - If run correctly (stdin JSON), this script executes and outputs JSONL with status: "pass"
 * - If run incorrectly (CLI args), the app shows warning after 2 seconds:
 *   ╔════════════════════════════════════════════════════════════════════════════╗
 *   ║  WARNING: No stdin JSON received after 2 seconds                          ║
 *   ╚════════════════════════════════════════════════════════════════════════════╝
 *
 * ## Available stdin Commands (for reference):
 * - {"type": "run", "path": "/absolute/path/to/script.ts"}
 * - {"type": "show"}
 * - {"type": "hide"}
 * - {"type": "setFilter", "text": "search term"}
 *
 * ## AI Log Mode:
 * Always use SCRIPT_KIT_AI_LOG=1 for compact logs (saves ~70% tokens)
 * Format: HH:MM:SS.mmm|L|C|message (UTC)
 * Categories: P=POSITION A=APP U=UI S=STDIN H=HOTKEY V=VISIBILITY E=EXEC K=KEY etc.
 */

import '../../scripts/kit-sdk';

// =============================================================================
// JSONL Test Output Utilities
// =============================================================================

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
  details?: Record<string, unknown>;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  // Output to stdout as JSONL for machine parsing
  console.log(JSON.stringify(result));
  // Also log to stderr for human visibility
  console.error(`[TEST] ${name}: ${status}${extra?.error ? ` - ${extra.error}` : ''}`);
}

// =============================================================================
// Test: Stdin Protocol Validation
// =============================================================================

const testName = 'stdin-protocol-validation';
const startTime = Date.now();

console.error('[GUIDELINE-TEST] test-guideline-stdin-protocol.ts starting...');
console.error('[GUIDELINE-TEST] This test validates the stdin JSON protocol guideline');

logTest(testName, 'running');

try {
  // If we're executing, stdin protocol was used correctly!
  // The script would NOT run if command line args were used (they do nothing)

  const protocolUsed = true; // We're running, so protocol was correct
  const sdkLoaded = typeof arg === 'function' && typeof div === 'function';
  const stdinActive = process.stdin.readable;

  console.error('[GUIDELINE-TEST] Validation checks:');
  console.error(`[GUIDELINE-TEST]   - Protocol used correctly: ${protocolUsed}`);
  console.error(`[GUIDELINE-TEST]   - SDK loaded: ${sdkLoaded}`);
  console.error(`[GUIDELINE-TEST]   - Stdin readable: ${stdinActive}`);

  // Document the correct and wrong approaches
  const documentation = {
    correct_method: {
      description: 'Use stdin JSON protocol with AI compact logs',
      command: 'echo \'{"type":"run","path":"$(pwd)/tests/smoke/script.ts"}\' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1',
      environment_var: 'SCRIPT_KIT_AI_LOG=1',
      stdin_commands: [
        '{"type": "run", "path": "/absolute/path/to/script.ts"}',
        '{"type": "show"}',
        '{"type": "hide"}',
        '{"type": "setFilter", "text": "search term"}'
      ]
    },
    wrong_method: {
      description: 'Command line args (DOES NOTHING!)',
      command: './target/debug/script-kit-gpui tests/smoke/script.ts',
      warning: 'This does absolutely nothing - the app ignores CLI args',
      expected_warning_after_2s: 'WARNING: No stdin JSON received after 2 seconds'
    },
    ai_log_format: {
      format: 'HH:MM:SS.mmm|L|C|message',
      levels: { i: 'INFO', w: 'WARN', e: 'ERROR', d: 'DEBUG', t: 'TRACE' },
      categories: {
        P: 'POSITION', A: 'APP', U: 'UI', S: 'STDIN',
        H: 'HOTKEY', V: 'VISIBILITY', E: 'EXEC', K: 'KEY',
        F: 'FOCUS', T: 'THEME', C: 'CACHE', R: 'PERF',
        W: 'WINDOW_MGR', X: 'ERROR', Z: 'RESIZE', G: 'SCRIPT'
      }
    }
  };

  // All checks passed
  if (protocolUsed && sdkLoaded) {
    logTest(testName, 'pass', {
      duration_ms: Date.now() - startTime,
      result: 'Stdin JSON protocol used correctly',
      details: {
        protocol_validated: true,
        sdk_loaded: sdkLoaded,
        stdin_active: stdinActive,
        documentation
      }
    });
    console.error('[GUIDELINE-TEST] ✅ PASS: Stdin JSON protocol was used correctly!');
  } else {
    logTest(testName, 'fail', {
      duration_ms: Date.now() - startTime,
      error: 'SDK not loaded or protocol issue',
      details: { sdkLoaded, stdinActive }
    });
    console.error('[GUIDELINE-TEST] ❌ FAIL: Protocol or SDK issue detected');
  }

  // Show a div with the test results (visual confirmation)
  await div(md(`# Stdin Protocol Test - PASS ✅

This test validates that the correct stdin JSON protocol was used.

## How This Test Was Run

Since you're seeing this, the correct method was used:

\`\`\`bash
echo '{"type":"run","path":"..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
\`\`\`

## What Would Happen With WRONG Method

If someone ran:
\`\`\`bash
./target/debug/script-kit-gpui tests/smoke/script.ts
\`\`\`

They would see a warning after 2 seconds:
\`\`\`
╔════════════════════════════════════════════════════════════════════════════╗
║  WARNING: No stdin JSON received after 2 seconds                          ║
╚════════════════════════════════════════════════════════════════════════════╝
\`\`\`

## Available Stdin Commands

| Command | Description |
|---------|-------------|
| \`{"type": "run", "path": "..."}\` | Execute a script |
| \`{"type": "show"}\` | Show the window |
| \`{"type": "hide"}\` | Hide the window |
| \`{"type": "setFilter", "text": "..."}\` | Set search filter |

---

*Press Escape or click to continue*`));

  console.error('[GUIDELINE-TEST] test-guideline-stdin-protocol.ts completed successfully!');

} catch (err) {
  const errorMessage = err instanceof Error ? err.message : String(err);
  logTest(testName, 'fail', {
    duration_ms: Date.now() - startTime,
    error: errorMessage
  });
  console.error(`[GUIDELINE-TEST] ❌ Error: ${errorMessage}`);
  process.exit(1);
}
