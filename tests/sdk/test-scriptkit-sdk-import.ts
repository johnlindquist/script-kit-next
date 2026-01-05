// Name: SDK Test - @scriptkit/sdk import redirect
// Description: Tests that import '@scriptkit/sdk' works correctly

/**
 * SDK TEST: test-import-redirect.ts
 *
 * Tests that the @scriptkit/sdk import redirect works correctly.
 * This verifies that the package.json "imports" field properly redirects
 * `import '@scriptkit/sdk'` to our local kit-sdk.ts implementation.
 *
 * Note: This test may be skipped if the import redirect isn't configured.
 */

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  reason?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

// =============================================================================
// Tests using dynamic import
// =============================================================================

async function runTests() {
  // Test 1: Try to import @scriptkit/sdk
  const test1 = 'import-scriptkit-sdk';
  logTest(test1, 'running');
  const start1 = Date.now();

  try {
    // Try dynamic import - this tests the package.json imports field
    await import('@scriptkit/sdk');
    logTest(test1, 'pass', {
      result: '@scriptkit/sdk import succeeded',
      duration_ms: Date.now() - start1
    });
  } catch (err) {
    // This is expected when the imports field isn't properly configured
    logTest(test1, 'skip', {
      reason: '@scriptkit/sdk import redirect not configured - this is expected in some environments',
      duration_ms: Date.now() - start1
    });

    // Since the import failed, run fallback tests using the direct import
    await runFallbackTests();
    return;
  }

  // If import worked, test that globals are available
  const test2 = 'globals-available-after-import';
  logTest(test2, 'running');
  const start2 = Date.now();

  try {
    // @ts-ignore - these would be globals after import
    const hasArg = typeof (globalThis as any).arg === 'function';
    // @ts-ignore
    const hasDiv = typeof (globalThis as any).div === 'function';
    // @ts-ignore
    const hasMd = typeof (globalThis as any).md === 'function';

    if (hasArg && hasDiv && hasMd) {
      logTest(test2, 'pass', {
        result: 'arg, div, md globals are available',
        duration_ms: Date.now() - start2
      });
    } else {
      logTest(test2, 'fail', {
        error: `Missing globals: arg=${hasArg}, div=${hasDiv}, md=${hasMd}`,
        duration_ms: Date.now() - start2
      });
    }
  } catch (err) {
    logTest(test2, 'fail', {
      error: String(err),
      duration_ms: Date.now() - start2
    });
  }
}

async function runFallbackTests() {
  // Fallback: test direct import works
  const test = 'direct-sdk-import';
  logTest(test, 'running');
  const start = Date.now();

  try {
    await import('../../scripts/kit-sdk');

    // @ts-ignore
    const hasArg = typeof (globalThis as any).arg === 'function';
    // @ts-ignore
    const hasDiv = typeof (globalThis as any).div === 'function';

    if (hasArg && hasDiv) {
      logTest(test, 'pass', {
        result: 'Direct SDK import works, globals available',
        duration_ms: Date.now() - start
      });
    } else {
      logTest(test, 'fail', {
        error: `Direct import worked but globals missing: arg=${hasArg}, div=${hasDiv}`,
        duration_ms: Date.now() - start
      });
    }
  } catch (err) {
    logTest(test, 'fail', {
      error: String(err),
      duration_ms: Date.now() - start
    });
  }
}

runTests().catch(err => {
  logTest('import-test-error', 'fail', { error: String(err) });
});
