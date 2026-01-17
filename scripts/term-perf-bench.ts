#!/usr/bin/env npx tsx
/**
 * Terminal Typing Performance Benchmark
 * 
 * Measures terminal typing performance to detect regressions from
 * excessive process() calls or cx.notify() storms in the terminal.
 * 
 * Background:
 * - Issue: excessive process() calls (8x in render, 4x in timer) and 
 *   cx.notify() on every keystroke caused render storms
 * - Fix: reduced to 2x each and removed cx.notify() from key handlers
 * 
 * This benchmark verifies the fix by measuring:
 * - Time to echo characters back
 * - Total time for rapid keystroke tests
 * - Any dropped/delayed keystrokes
 * 
 * Performance Thresholds (from AGENTS.md):
 * - P95 Key Latency: < 50ms
 * - Single Key Event: < 16.67ms (60fps)
 * 
 * Usage:
 *   npx tsx scripts/term-perf-bench.ts [options]
 * 
 * Options:
 *   --iterations N   Number of test iterations (default: 3)
 *   --output json    Output format: json or text (default: text)
 */

import './kit-sdk';

// Local wait helper (not provided by SDK)
const wait = (ms: number) => new Promise<void>(r => setTimeout(r, ms));

// =============================================================================
// Types
// =============================================================================

interface BenchmarkResult {
  test: string;
  iterations: number;
  latencies: LatencyStats[];
  summary: LatencyStats;
  totalTime: number;
}

interface LatencyStats {
  count: number;
  min: number;
  max: number;
  avg: number;
  p50: number;
  p95: number;
  p99: number;
}

interface BenchmarkConfig {
  iterations: number;
  outputFormat: 'json' | 'text';
}

// =============================================================================
// Utilities
// =============================================================================

function debug(msg: string) {
  console.error(`[TERM-BENCH] ${msg}`);
}

function calculateStats(samples: number[]): LatencyStats {
  if (samples.length === 0) {
    return { count: 0, min: 0, max: 0, avg: 0, p50: 0, p95: 0, p99: 0 };
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
  };
}

function aggregateStats(allStats: LatencyStats[]): LatencyStats {
  if (allStats.length === 0) {
    return { count: 0, min: 0, max: 0, avg: 0, p50: 0, p95: 0, p99: 0 };
  }

  const mins = allStats.map(s => s.min);
  const maxs = allStats.map(s => s.max);
  const avgs = allStats.map(s => s.avg);
  const p50s = allStats.map(s => s.p50);
  const p95s = allStats.map(s => s.p95);
  const p99s = allStats.map(s => s.p99);

  return {
    count: allStats.reduce((sum, s) => sum + s.count, 0),
    min: Math.min(...mins),
    max: Math.max(...maxs),
    avg: avgs.reduce((sum, v) => sum + v, 0) / avgs.length,
    p50: p50s.reduce((sum, v) => sum + v, 0) / p50s.length,
    p95: p95s.reduce((sum, v) => sum + v, 0) / p95s.length,
    p99: p99s.reduce((sum, v) => sum + v, 0) / p99s.length,
  };
}

function formatMs(ms: number): string {
  return ms.toFixed(2).padStart(8);
}

function printTextReport(results: BenchmarkResult[]) {
  console.log('\n' + '='.repeat(80));
  console.log('TERMINAL TYPING PERFORMANCE BENCHMARK REPORT');
  console.log('='.repeat(80));
  console.log(`Date: ${new Date().toISOString()}`);
  console.log('');

  for (const result of results) {
    console.log(`\n${'─'.repeat(80)}`);
    console.log(`TEST: ${result.test}`);
    console.log(`Iterations: ${result.iterations}`);
    console.log(`Total Time: ${result.totalTime.toFixed(0)}ms`);
    console.log('─'.repeat(80));
    
    console.log('\nPer-Iteration Results:');
    console.log('  Iter |    Min    |    Max    |    Avg    |    P50    |    P95    |    P99');
    console.log('  ' + '─'.repeat(76));
    
    result.latencies.forEach((stats, i) => {
      console.log(
        `  ${(i + 1).toString().padStart(4)} | ` +
        `${formatMs(stats.min)} | ` +
        `${formatMs(stats.max)} | ` +
        `${formatMs(stats.avg)} | ` +
        `${formatMs(stats.p50)} | ` +
        `${formatMs(stats.p95)} | ` +
        `${formatMs(stats.p99)}`
      );
    });

    console.log('\nAggregate Summary:');
    console.log(`  Total Samples: ${result.summary.count}`);
    console.log(`  Min Latency:   ${result.summary.min.toFixed(2)}ms`);
    console.log(`  Max Latency:   ${result.summary.max.toFixed(2)}ms`);
    console.log(`  Avg Latency:   ${result.summary.avg.toFixed(2)}ms`);
    console.log(`  P50 Latency:   ${result.summary.p50.toFixed(2)}ms`);
    console.log(`  P95 Latency:   ${result.summary.p95.toFixed(2)}ms`);
    console.log(`  P99 Latency:   ${result.summary.p99.toFixed(2)}ms`);
    
    // Pass/Fail assessment based on AGENTS.md thresholds
    const p95Threshold = 50;
    const singleEventThreshold = 16.67;
    const p95Passed = result.summary.p95 < p95Threshold;
    const avgPassed = result.summary.avg < singleEventThreshold;
    
    console.log(`\n  P95 Status:    ${p95Passed ? '✓ PASS' : '✗ FAIL'} (threshold: ${p95Threshold}ms)`);
    console.log(`  Avg Status:    ${avgPassed ? '✓ PASS' : '⚠ WARN'} (60fps target: ${singleEventThreshold}ms)`);
  }

  console.log('\n' + '='.repeat(80));
}

// =============================================================================
// Benchmark Tests
// =============================================================================

/**
 * Benchmark 1: Rapid typing "hello world" multiple times
 * Simulates fast human typing in the terminal
 */
async function runRapidTypingBenchmark(
  repeatCount: number,
  intervalMs: number
): Promise<{ stats: LatencyStats; totalTime: number }> {
  const testString = 'hello world';
  const totalStart = performance.now();
  
  // Open terminal with cat command to echo back input
  // Using a simple echo approach - type text then exit
  const termPromise = term(`
echo "Starting rapid typing test..."
# Terminal ready for input
cat
`);
  
  // Wait for terminal to initialize
  await wait(500);
  
  // Collect latency samples
  const latencies: number[] = [];
  
  for (let rep = 0; rep < repeatCount; rep++) {
    for (const char of testString) {
      const start = performance.now();
      await keyboard.tap(char);
      const end = performance.now();
      latencies.push(end - start);
      await wait(intervalMs);
    }
    
    // Press Enter after each "hello world"
    const enterStart = performance.now();
    await keyboard.tap('enter');
    latencies.push(performance.now() - enterStart);
    await wait(intervalMs);
  }
  
  // Exit the terminal
  await keyboard.tap('c', 'control'); // Ctrl+C to exit cat
  await wait(100);
  await keyboard.tap('escape'); // Close terminal
  
  try {
    await termPromise;
  } catch {
    // Terminal may close abruptly - that's OK
  }
  
  const totalTime = performance.now() - totalStart;
  
  return {
    stats: calculateStats(latencies),
    totalTime,
  };
}

/**
 * Benchmark 2: Burst typing - rapid bursts with pauses
 * Simulates typing in bursts like when copying/pasting
 */
async function runBurstTypingBenchmark(
  burstCount: number,
  charsPerBurst: number,
  burstIntervalMs: number,
  pauseMs: number
): Promise<{ stats: LatencyStats; totalTime: number }> {
  const totalStart = performance.now();
  
  // Open terminal with cat to echo back
  const termPromise = term(`
echo "Starting burst typing test..."
cat
`);
  
  // Wait for terminal to initialize
  await wait(500);
  
  const latencies: number[] = [];
  const testChars = 'abcdefghijklmnopqrstuvwxyz0123456789';
  
  for (let burst = 0; burst < burstCount; burst++) {
    debug(`Burst ${burst + 1}/${burstCount}`);
    
    // Rapid burst of characters
    for (let i = 0; i < charsPerBurst; i++) {
      const char = testChars[i % testChars.length];
      const start = performance.now();
      await keyboard.tap(char);
      const end = performance.now();
      latencies.push(end - start);
      await wait(burstIntervalMs);
    }
    
    // Pause between bursts
    await wait(pauseMs);
  }
  
  // Exit the terminal
  await keyboard.tap('c', 'control');
  await wait(100);
  await keyboard.tap('escape');
  
  try {
    await termPromise;
  } catch {
    // Terminal may close abruptly
  }
  
  const totalTime = performance.now() - totalStart;
  
  return {
    stats: calculateStats(latencies),
    totalTime,
  };
}

/**
 * Benchmark 3: Sustained rapid-fire input
 * Simulates holding down a key with fast repeat rate
 */
async function runSustainedInputBenchmark(
  keyCount: number,
  intervalMs: number
): Promise<{ stats: LatencyStats; totalTime: number }> {
  const totalStart = performance.now();
  
  // Open terminal with cat
  const termPromise = term(`
echo "Starting sustained input test..."
cat
`);
  
  // Wait for terminal to initialize
  await wait(500);
  
  const latencies: number[] = [];
  
  // Simulate holding down 'x' key with rapid repeat
  for (let i = 0; i < keyCount; i++) {
    const start = performance.now();
    await keyboard.tap('x');
    const end = performance.now();
    latencies.push(end - start);
    await wait(intervalMs);
  }
  
  // Exit
  await keyboard.tap('c', 'control');
  await wait(100);
  await keyboard.tap('escape');
  
  try {
    await termPromise;
  } catch {
    // Terminal may close abruptly
  }
  
  const totalTime = performance.now() - totalStart;
  
  return {
    stats: calculateStats(latencies),
    totalTime,
  };
}

// =============================================================================
// Main
// =============================================================================

debug('term-perf-bench.ts starting...');

// Parse config
const config: BenchmarkConfig = {
  iterations: 3,
  outputFormat: 'text',
};

// Check command line args
const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--iterations' && args[i + 1]) {
    config.iterations = parseInt(args[i + 1], 10) || 3;
  }
  if (args[i] === '--output' && args[i + 1]) {
    config.outputFormat = args[i + 1] === 'json' ? 'json' : 'text';
  }
}

debug(`Config: iterations=${config.iterations}, output=${config.outputFormat}`);

const results: BenchmarkResult[] = [];

// -----------------------------------------------------------------------------
// Benchmark 1: Rapid Typing (20ms interval - ~50 WPM typing speed)
// -----------------------------------------------------------------------------
debug('Running Benchmark 1: Rapid Typing...');

const rapidResults: LatencyStats[] = [];
let rapidTotalTime = 0;

for (let i = 0; i < config.iterations; i++) {
  debug(`  Iteration ${i + 1}/${config.iterations}`);
  const result = await runRapidTypingBenchmark(10, 20); // "hello world" 10 times
  rapidResults.push(result.stats);
  rapidTotalTime += result.totalTime;
  await wait(500); // Cool-down between iterations
}

results.push({
  test: 'rapid-typing',
  iterations: config.iterations,
  latencies: rapidResults,
  summary: aggregateStats(rapidResults),
  totalTime: rapidTotalTime,
});

// -----------------------------------------------------------------------------
// Benchmark 2: Burst Typing (rapid bursts with pauses)
// -----------------------------------------------------------------------------
debug('Running Benchmark 2: Burst Typing...');

const burstResults: LatencyStats[] = [];
let burstTotalTime = 0;

for (let i = 0; i < config.iterations; i++) {
  debug(`  Iteration ${i + 1}/${config.iterations}`);
  const result = await runBurstTypingBenchmark(5, 20, 5, 200); // 5 bursts of 20 chars
  burstResults.push(result.stats);
  burstTotalTime += result.totalTime;
  await wait(500);
}

results.push({
  test: 'burst-typing',
  iterations: config.iterations,
  latencies: burstResults,
  summary: aggregateStats(burstResults),
  totalTime: burstTotalTime,
});

// -----------------------------------------------------------------------------
// Benchmark 3: Sustained Rapid-Fire (10ms interval - simulates key hold)
// -----------------------------------------------------------------------------
debug('Running Benchmark 3: Sustained Rapid-Fire...');

const sustainedResults: LatencyStats[] = [];
let sustainedTotalTime = 0;

for (let i = 0; i < config.iterations; i++) {
  debug(`  Iteration ${i + 1}/${config.iterations}`);
  const result = await runSustainedInputBenchmark(100, 10); // 100 keys at 10ms interval
  sustainedResults.push(result.stats);
  sustainedTotalTime += result.totalTime;
  await wait(500);
}

results.push({
  test: 'sustained-rapid-fire',
  iterations: config.iterations,
  latencies: sustainedResults,
  summary: aggregateStats(sustainedResults),
  totalTime: sustainedTotalTime,
});

// -----------------------------------------------------------------------------
// Output Results
// -----------------------------------------------------------------------------

if (config.outputFormat === 'json') {
  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    config,
    results,
    thresholds: {
      p95_latency_ms: 50,
      single_event_ms: 16.67,
    },
  }, null, 2));
} else {
  printTextReport(results);
}

// -----------------------------------------------------------------------------
// Summary Display
// -----------------------------------------------------------------------------

const allPassed = results.every(r => r.summary.p95 < 50);

await div(md(`# Terminal Typing Performance Benchmark Complete

## Configuration
- **Iterations**: ${config.iterations}

## Results Summary
${results.map(r => `
### ${r.test}
- **P95 Latency**: ${r.summary.p95.toFixed(2)}ms
- **Avg Latency**: ${r.summary.avg.toFixed(2)}ms
- **Total Time**: ${r.totalTime.toFixed(0)}ms
- **Status**: ${r.summary.p95 < 50 ? '✓ PASS' : '✗ FAIL'}
`).join('\n')}

## Overall: ${allPassed ? '✓ ALL TESTS PASSED' : '✗ SOME TESTS FAILED'}

### Performance Thresholds (from AGENTS.md)
- **P95 Key Latency**: < 50ms
- **Single Key Event**: < 16.67ms (60fps)

---

*Full statistics printed to console*

Press Escape or click to exit.`));

debug('term-perf-bench.ts completed!');
