#!/usr/bin/env npx tsx
/**
 * Scroll Performance Benchmark
 * 
 * Runs scroll performance tests multiple times and outputs statistics.
 * Use this to compare performance before and after fixes.
 * 
 * Usage:
 *   npx tsx scripts/scroll-bench.ts [options]
 * 
 * Options:
 *   --iterations N   Number of test iterations (default: 5)
 *   --list-size N    Number of items in the list (default: 500)
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
  listSize: number;
  outputFormat: 'json' | 'text';
}

// =============================================================================
// Utilities
// =============================================================================

function debug(msg: string) {
  console.error(`[BENCH] ${msg}`);
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

function generateChoices(count: number): string[] {
  const choices: string[] = [];
  for (let i = 0; i < count; i++) {
    choices.push(`Item ${i.toString().padStart(4, '0')} - Lorem ipsum dolor sit amet`);
  }
  return choices;
}

function formatMs(ms: number): string {
  return ms.toFixed(2).padStart(8);
}

function printTextReport(results: BenchmarkResult[]) {
  console.log('\n' + '='.repeat(80));
  console.log('SCROLL PERFORMANCE BENCHMARK REPORT');
  console.log('='.repeat(80));
  console.log(`Date: ${new Date().toISOString()}`);
  console.log('');

  for (const result of results) {
    console.log(`\n${'─'.repeat(80)}`);
    console.log(`TEST: ${result.test}`);
    console.log(`Iterations: ${result.iterations}`);
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
    
    // Pass/Fail assessment
    const threshold = 50;
    const passed = result.summary.p95 < threshold;
    console.log(`\n  Status: ${passed ? '✓ PASS' : '✗ FAIL'} (P95 threshold: ${threshold}ms)`);
  }

  console.log('\n' + '='.repeat(80));
}

// =============================================================================
// Benchmark Tests
// =============================================================================

async function runRapidScrollBenchmark(
  choices: string[],
  keyCount: number,
  intervalMs: number
): Promise<LatencyStats> {
  // Start the arg prompt
  const argPromise = arg('Benchmark: Rapid Scroll', choices);
  
  // Wait for UI to initialize
  await wait(500);
  
  // Collect latency samples
  const latencies: number[] = [];
  for (let i = 0; i < keyCount; i++) {
    const start = performance.now();
    await keyboard.tap('down');
    const end = performance.now();
    latencies.push(end - start);
    await wait(intervalMs);
  }
  
  // Submit to close
  submit(choices[Math.min(keyCount, choices.length - 1)]);
  await argPromise;
  
  return calculateStats(latencies);
}

async function runBurstScrollBenchmark(
  choices: string[],
  burstCount: number,
  keysPerBurst: number,
  burstIntervalMs: number,
  pauseMs: number
): Promise<LatencyStats> {
  // Start the arg prompt
  const argPromise = arg('Benchmark: Burst Scroll', choices);
  
  // Wait for UI to initialize
  await wait(500);
  
  // Collect latency samples
  const latencies: number[] = [];
  for (let burst = 0; burst < burstCount; burst++) {
    for (let i = 0; i < keysPerBurst; i++) {
      const start = performance.now();
      await keyboard.tap('down');
      const end = performance.now();
      latencies.push(end - start);
      await wait(burstIntervalMs);
    }
    await wait(pauseMs);
  }
  
  // Submit to close
  const finalIndex = Math.min(burstCount * keysPerBurst, choices.length - 1);
  submit(choices[finalIndex]);
  await argPromise;
  
  return calculateStats(latencies);
}

// =============================================================================
// Main
// =============================================================================

debug('scroll-bench.ts starting...');

// Parse config (would use process.argv in real implementation)
const config: BenchmarkConfig = {
  iterations: 3,
  listSize: 500,
  outputFormat: 'text',
};

debug(`Config: iterations=${config.iterations}, listSize=${config.listSize}`);

const choices = generateChoices(config.listSize);
const results: BenchmarkResult[] = [];

// -----------------------------------------------------------------------------
// Benchmark 1: Rapid Scroll (10ms interval - simulates fast key repeat)
// -----------------------------------------------------------------------------
debug('Running Benchmark 1: Rapid Scroll...');

const rapidResults: LatencyStats[] = [];
for (let i = 0; i < config.iterations; i++) {
  debug(`  Iteration ${i + 1}/${config.iterations}`);
  const stats = await runRapidScrollBenchmark(choices, 100, 10);
  rapidResults.push(stats);
  await wait(500); // Cool-down between iterations
}

results.push({
  test: 'rapid-scroll',
  iterations: config.iterations,
  latencies: rapidResults,
  summary: aggregateStats(rapidResults),
});

// -----------------------------------------------------------------------------
// Benchmark 2: Burst Scroll (rapid bursts with pauses)
// -----------------------------------------------------------------------------
debug('Running Benchmark 2: Burst Scroll...');

const burstResults: LatencyStats[] = [];
for (let i = 0; i < config.iterations; i++) {
  debug(`  Iteration ${i + 1}/${config.iterations}`);
  const stats = await runBurstScrollBenchmark(choices, 5, 20, 5, 200);
  burstResults.push(stats);
  await wait(500); // Cool-down between iterations
}

results.push({
  test: 'burst-scroll',
  iterations: config.iterations,
  latencies: burstResults,
  summary: aggregateStats(burstResults),
});

// -----------------------------------------------------------------------------
// Output Results
// -----------------------------------------------------------------------------

if (config.outputFormat === 'json') {
  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    config,
    results,
  }, null, 2));
} else {
  printTextReport(results);
}

// -----------------------------------------------------------------------------
// Summary Display
// -----------------------------------------------------------------------------
await div(md(`# Scroll Benchmark Complete

## Configuration
- **Iterations**: ${config.iterations}
- **List Size**: ${config.listSize}

## Results Summary
${results.map(r => `
### ${r.test}
- **P95 Latency**: ${r.summary.p95.toFixed(2)}ms
- **Status**: ${r.summary.p95 < 50 ? '✓ PASS' : '✗ FAIL'}
`).join('\n')}

---

*Full statistics printed to console*

Press Escape or click to exit.`));

debug('scroll-bench.ts completed!');
