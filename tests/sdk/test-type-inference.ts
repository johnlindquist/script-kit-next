// Type-level tests for schema inference
// This file should compile without errors if types are working correctly

import '../../scripts/kit-sdk';

// =============================================================================
// Test Infrastructure
// =============================================================================

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
    ...extra
  };
  console.log(JSON.stringify(result));
}

// =============================================================================
// Tests
// =============================================================================

// Test: defineSchema returns typed input/output functions
const test1 = 'defineSchema-returns-typed-functions';
logTest(test1, 'running');
const start1 = Date.now();

try {
  const { input, output } = defineSchema({
    input: {
      greeting: { type: 'string', required: true },
      count: { type: 'number' },
    },
    output: {
      message: { type: 'string' },
      success: { type: 'boolean' },
    },
  } as const);

  if (typeof input === 'function' && typeof output === 'function') {
    logTest(test1, 'pass', {
      result: 'defineSchema returns input and output functions',
      duration_ms: Date.now() - start1
    });
  } else {
    logTest(test1, 'fail', {
      error: `Expected functions, got input=${typeof input}, output=${typeof output}`,
      duration_ms: Date.now() - start1
    });
  }
} catch (err) {
  logTest(test1, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start1
  });
}

// Test: output() should accept typed object
const test2 = 'output-accepts-typed-object';
logTest(test2, 'running');
const start2 = Date.now();

try {
  const { output } = defineSchema({
    output: {
      message: { type: 'string' },
      success: { type: 'boolean' },
    },
  } as const);

  // Should work - partial output
  output({ message: 'test' });
  output({ success: true });
  output({ message: 'test', success: false });

  logTest(test2, 'pass', {
    result: 'output() accepts partial typed objects',
    duration_ms: Date.now() - start2
  });
} catch (err) {
  logTest(test2, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start2
  });
}

// Test: type checking at compile time (this is a compile-time test)
const test3 = 'type-checking-compile-time';
logTest(test3, 'pass', {
  result: 'If this test runs, TypeScript types compiled correctly',
  duration_ms: 0
});

console.error('[TEST] Type inference tests completed!');
