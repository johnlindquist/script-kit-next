/**
 * Test Helpers
 * 
 * Common utilities for SDK tests. Import these instead of relying on SDK globals
 * for test-specific utilities.
 * 
 * Usage:
 *   import { wait, logTest, debug } from './test-helpers';
 */

// =============================================================================
// Timing Utilities
// =============================================================================

/**
 * Promise-based delay for tests (local helper, not from SDK)
 * @param ms - Milliseconds to wait
 */
export const wait = (ms: number): Promise<void> => new Promise(r => setTimeout(r, ms));

// =============================================================================
// Test Infrastructure
// =============================================================================

export interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
  expected?: string;
  actual?: string;
  reason?: string;
}

export function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

export function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}
