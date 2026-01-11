import '../../scripts/kit-sdk';

/**
 * Window Visibility Scenarios Test Suite
 *
 * Tests various scenarios for main menu visibility behavior.
 * Run with: echo '{"type":"run","path":"tests/smoke/test-window-visibility-scenarios.ts"}' | ./target/debug/script-kit-gpui
 */

type TestResult = {
  test: string;
  status: 'pass' | 'fail' | 'skip';
  reason?: string;
  duration_ms: number;
};

const results: TestResult[] = [];

function log(test: string, status: string, extra: any = {}) {
  console.error(JSON.stringify({
    suite: 'window-visibility',
    test,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  }));
}

async function runTest(name: string, fn: () => Promise<void>): Promise<void> {
  const start = Date.now();
  log(name, 'running');
  try {
    await fn();
    results.push({ test: name, status: 'pass', duration_ms: Date.now() - start });
    log(name, 'pass', { duration_ms: Date.now() - start });
  } catch (e: any) {
    results.push({ test: name, status: 'fail', reason: e.message, duration_ms: Date.now() - start });
    log(name, 'fail', { error: e.message, duration_ms: Date.now() - start });
  }
}

// ============================================
// TEST 1: Script that hides and exits cleanly
// Expected: Main menu should come back
// ============================================
await runTest('hide-then-exit', async () => {
  await hide();
  await new Promise(r => setTimeout(r, 50));
  // Script exits after this - window should show
});

// Small delay between tests
await new Promise(r => setTimeout(r, 100));

// ============================================
// TEST 2: Script that shows HUD (which hides) and exits
// Expected: Main menu should come back after HUD
// ============================================
await runTest('hud-then-exit', async () => {
  // HUD auto-hides the window
  await hud('Test HUD message', 500);
  await new Promise(r => setTimeout(r, 100));
  // Script exits - window should show
});

await new Promise(r => setTimeout(r, 100));

// ============================================
// TEST 3: getSelectedText failure pattern
// Expected: Main menu should come back
// ============================================
await runTest('getSelectedText-failure-pattern', async () => {
  let text: string | undefined;
  try {
    text = await getSelectedText();
  } catch {
    // Fall through - expected (accessibility permission denied)
  }

  if (!text?.trim()) {
    await hud('No text selected');
  }
  // Script exits - window should show
});

await new Promise(r => setTimeout(r, 100));

// ============================================
// TEST 4: Multiple hide calls
// Expected: Main menu should come back (only need one show)
// ============================================
await runTest('multiple-hides', async () => {
  await hide();
  await new Promise(r => setTimeout(r, 20));
  await hide();
  await new Promise(r => setTimeout(r, 20));
  await hide();
  // Script exits - window should show (once)
});

await new Promise(r => setTimeout(r, 100));

// ============================================
// TEST 5: Script that does NOT hide
// Expected: Window state should be unchanged
// ============================================
await runTest('no-hide', async () => {
  // Just do some work without hiding
  const x = 1 + 1;
  await new Promise(r => setTimeout(r, 50));
  // Script exits - window should remain in whatever state it was
});

// ============================================
// Summary
// ============================================
log('summary', 'complete', {
  total: results.length,
  passed: results.filter(r => r.status === 'pass').length,
  failed: results.filter(r => r.status === 'fail').length,
  results
});

exit(0);
