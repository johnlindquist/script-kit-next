// Name: SDK Test - Scroll Performance
// Description: Tests scroll performance with rapid keyboard events

/**
 * SDK TEST: test-scroll-perf.ts
 * 
 * Reproduces and tests the scroll hang condition that occurs when holding
 * down arrow keys with a fast repeat rate. Measures latency between
 * key events and UI response.
 * 
 * Test cases:
 * 1. scroll-normal: Normal speed scrolling (200ms between keys)
 * 2. scroll-fast: Fast scrolling (50ms between keys)
 * 3. scroll-rapid: Rapid-fire scrolling (10ms between keys - simulates hold)
 * 
 * Expected behavior:
 * - 95th percentile latency should stay under 50ms
 * - No UI hangs or freezes during rapid scrolling
 */

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

interface LatencyStats {
  count: number;
  min: number;
  max: number;
  avg: number;
  p50: number;
  p95: number;
  p99: number;
  samples: number[];
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

function debug(msg: string) {
  console.error(`[SCROLL-PERF] ${msg}`);
}

function calculateStats(samples: number[]): LatencyStats {
  if (samples.length === 0) {
    return {
      count: 0,
      min: 0,
      max: 0,
      avg: 0,
      p50: 0,
      p95: 0,
      p99: 0,
      samples: [],
    };
  }

  const sorted = [...samples].sort((a, b) => a - b);
  const sum = sorted.reduce((acc, val) => acc + val, 0);
  
  return {
    count: sorted.length,
    min: sorted[0],
    max: sorted[sorted.length - 1],
    avg: sum / sorted.length,
    p50: sorted[Math.floor(sorted.length * 0.5)],
    p95: sorted[Math.floor(sorted.length * 0.95)],
    p99: sorted[Math.floor(sorted.length * 0.99)],
    samples: sorted,
  };
}

// Generate a large list of choices for scrolling tests
function generateChoices(count: number): string[] {
  const choices: string[] = [];
  for (let i = 0; i < count; i++) {
    choices.push(`Item ${i.toString().padStart(4, '0')} - Lorem ipsum dolor sit amet`);
  }
  return choices;
}

// Note: simulateKeyEvents helper is available for future extensions
// Currently using inline loops for more granular control

// =============================================================================
// Tests
// =============================================================================

debug('test-scroll-perf.ts starting...');
debug(`SDK globals: arg=${typeof arg}, keyboard=${typeof keyboard}`);

// Configuration
const LIST_SIZE = 500; // Number of items in the list
const LATENCY_THRESHOLD_P95 = 50; // 95th percentile must be under 50ms

// Generate choices once for all tests
const choices = generateChoices(LIST_SIZE);

// -----------------------------------------------------------------------------
// Test 1: Normal speed scrolling
// -----------------------------------------------------------------------------
const test1 = 'scroll-normal';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: Normal speed scrolling (200ms interval)');
  
  // Start the arg prompt in the background
  const argPromise = arg('Scroll Performance Test - Normal Speed', choices);
  
  // Wait a bit for the UI to initialize
  await wait(500);
  
  // Simulate 20 down arrow key presses at normal speed
  const latencies: number[] = [];
  for (let i = 0; i < 20; i++) {
    const start = performance.now();
    await keyboard.tap('down');
    const end = performance.now();
    latencies.push(end - start);
    await wait(200);
  }
  
  // Submit to close the prompt
  submit(choices[20]);
  await argPromise;
  
  const stats = calculateStats(latencies);
  debug(`Test 1 stats: min=${stats.min.toFixed(2)}ms, max=${stats.max.toFixed(2)}ms, avg=${stats.avg.toFixed(2)}ms, p95=${stats.p95.toFixed(2)}ms`);
  
  if (stats.p95 < LATENCY_THRESHOLD_P95) {
    logTest(test1, 'pass', { 
      result: stats, 
      duration_ms: Date.now() - start1 
    });
  } else {
    logTest(test1, 'fail', { 
      result: stats,
      error: `p95 latency ${stats.p95.toFixed(2)}ms exceeds threshold ${LATENCY_THRESHOLD_P95}ms`,
      duration_ms: Date.now() - start1 
    });
  }
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: Fast scrolling
// -----------------------------------------------------------------------------
const test2 = 'scroll-fast';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: Fast scrolling (50ms interval)');
  
  // Start the arg prompt
  const argPromise = arg('Scroll Performance Test - Fast Speed', choices);
  
  // Wait for UI to initialize
  await wait(500);
  
  // Simulate 50 down arrow key presses at fast speed
  const latencies: number[] = [];
  for (let i = 0; i < 50; i++) {
    const start = performance.now();
    await keyboard.tap('down');
    const end = performance.now();
    latencies.push(end - start);
    await wait(50);
  }
  
  // Submit to close the prompt
  submit(choices[50]);
  await argPromise;
  
  const stats = calculateStats(latencies);
  debug(`Test 2 stats: min=${stats.min.toFixed(2)}ms, max=${stats.max.toFixed(2)}ms, avg=${stats.avg.toFixed(2)}ms, p95=${stats.p95.toFixed(2)}ms`);
  
  if (stats.p95 < LATENCY_THRESHOLD_P95) {
    logTest(test2, 'pass', { 
      result: stats, 
      duration_ms: Date.now() - start2 
    });
  } else {
    logTest(test2, 'fail', { 
      result: stats,
      error: `p95 latency ${stats.p95.toFixed(2)}ms exceeds threshold ${LATENCY_THRESHOLD_P95}ms`,
      duration_ms: Date.now() - start2 
    });
  }
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Rapid-fire scrolling (simulates holding key with fast repeat)
// -----------------------------------------------------------------------------
const test3 = 'scroll-rapid';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: Rapid-fire scrolling (10ms interval - simulates key hold)');
  
  // Start the arg prompt
  const argPromise = arg('Scroll Performance Test - Rapid Fire', choices);
  
  // Wait for UI to initialize
  await wait(500);
  
  // Simulate 100 down arrow key presses at rapid-fire speed
  // This simulates holding down the arrow key with a fast repeat rate
  const latencies: number[] = [];
  for (let i = 0; i < 100; i++) {
    const start = performance.now();
    await keyboard.tap('down');
    const end = performance.now();
    latencies.push(end - start);
    await wait(10); // 10ms = 100 keys per second repeat rate
  }
  
  // Submit to close the prompt
  submit(choices[100]);
  await argPromise;
  
  const stats = calculateStats(latencies);
  debug(`Test 3 stats: min=${stats.min.toFixed(2)}ms, max=${stats.max.toFixed(2)}ms, avg=${stats.avg.toFixed(2)}ms, p95=${stats.p95.toFixed(2)}ms`);
  
  // This is the critical test - rapid key events should not cause hangs
  if (stats.p95 < LATENCY_THRESHOLD_P95) {
    logTest(test3, 'pass', { 
      result: stats, 
      duration_ms: Date.now() - start3 
    });
  } else {
    logTest(test3, 'fail', { 
      result: stats,
      error: `p95 latency ${stats.p95.toFixed(2)}ms exceeds threshold ${LATENCY_THRESHOLD_P95}ms`,
      duration_ms: Date.now() - start3 
    });
  }
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Test 4: Burst scrolling (rapid bursts with pauses)
// -----------------------------------------------------------------------------
const test4 = 'scroll-burst';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: Burst scrolling (rapid bursts with pauses)');
  
  // Start the arg prompt
  const argPromise = arg('Scroll Performance Test - Burst Mode', choices);
  
  // Wait for UI to initialize
  await wait(500);
  
  const latencies: number[] = [];
  
  // Perform 5 bursts of 20 rapid key presses each
  for (let burst = 0; burst < 5; burst++) {
    debug(`Burst ${burst + 1}/5`);
    
    // Rapid burst
    for (let i = 0; i < 20; i++) {
      const start = performance.now();
      await keyboard.tap('down');
      const end = performance.now();
      latencies.push(end - start);
      await wait(5); // Very fast within burst
    }
    
    // Pause between bursts
    await wait(200);
  }
  
  // Submit to close the prompt
  submit(choices[100]);
  await argPromise;
  
  const stats = calculateStats(latencies);
  debug(`Test 4 stats: min=${stats.min.toFixed(2)}ms, max=${stats.max.toFixed(2)}ms, avg=${stats.avg.toFixed(2)}ms, p95=${stats.p95.toFixed(2)}ms`);
  
  if (stats.p95 < LATENCY_THRESHOLD_P95) {
    logTest(test4, 'pass', { 
      result: stats, 
      duration_ms: Date.now() - start4 
    });
  } else {
    logTest(test4, 'fail', { 
      result: stats,
      error: `p95 latency ${stats.p95.toFixed(2)}ms exceeds threshold ${LATENCY_THRESHOLD_P95}ms`,
      duration_ms: Date.now() - start4 
    });
  }
} catch (err) {
  logTest(test4, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug('test-scroll-perf.ts completed!');

await div(md(`# Scroll Performance Tests Complete

All scroll performance tests have been executed.

## Test Cases Run
1. **scroll-normal**: Normal speed scrolling (200ms interval)
2. **scroll-fast**: Fast scrolling (50ms interval)
3. **scroll-rapid**: Rapid-fire scrolling (10ms interval)
4. **scroll-burst**: Burst scrolling (rapid bursts with pauses)

## Performance Thresholds
- **p95 Latency Threshold**: ${LATENCY_THRESHOLD_P95}ms
- **List Size**: ${LIST_SIZE} items

---

*Check the JSONL output for detailed latency statistics*

Press Escape or click to exit.`));

debug('test-scroll-perf.ts exiting...');
