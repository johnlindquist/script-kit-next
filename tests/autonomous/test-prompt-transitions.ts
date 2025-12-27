// Prompt Transition Stress Tests
// Tests rapid transitions between different prompt types to catch resize race conditions
// Uses getWindowBounds() to verify window dimensions after each transition

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

interface TransitionMetric {
  from: string;
  to: string;
  transition_ms: number;
  resize_ms: number;
  final_bounds: WindowBounds;
  expected_height: number;
  height_match: boolean;
  width_match: boolean;
}

interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Layout constants from AGENTS.md
const MIN_HEIGHT = 120;  // arg with no choices
const MAX_HEIGHT = 700;  // editor/div/term
const WINDOW_WIDTH = 750;
const RESIZE_SETTLE_TIME = 100; // ms to wait for resize to complete

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

function logMetric(metric: TransitionMetric) {
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'prompt_transition',
    ...metric,
    timestamp: new Date().toISOString()
  }));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

async function runTest(name: string, fn: () => Promise<void>) {
  logTest(name, 'running');
  const start = Date.now();
  try {
    await fn();
    logTest(name, 'pass', { duration_ms: Date.now() - start });
  } catch (err) {
    logTest(name, 'fail', { error: String(err), duration_ms: Date.now() - start });
  }
}

/**
 * Measure a prompt transition with timing metrics
 */
async function measureTransition(
  fromPrompt: string,
  toPrompt: string,
  promptFn: () => Promise<unknown>,
  expectedHeight: number | { min: number; max: number }
): Promise<TransitionMetric> {
  const transitionStart = Date.now();
  
  // Execute the prompt
  await promptFn();
  
  const transitionEnd = Date.now();
  
  // Wait for resize to settle
  await wait(RESIZE_SETTLE_TIME);
  
  // Measure final bounds
  const bounds = await getWindowBounds();
  const resizeEnd = Date.now();
  
  const expectedH = typeof expectedHeight === 'number' ? expectedHeight : expectedHeight.max;
  const heightMatch = typeof expectedHeight === 'number'
    ? bounds.height === expectedHeight
    : bounds.height >= expectedHeight.min && bounds.height <= expectedHeight.max;
  
  const metric: TransitionMetric = {
    from: fromPrompt,
    to: toPrompt,
    transition_ms: transitionEnd - transitionStart,
    resize_ms: resizeEnd - transitionEnd,
    final_bounds: bounds,
    expected_height: expectedH,
    height_match: heightMatch,
    width_match: bounds.width === WINDOW_WIDTH
  };
  
  logMetric(metric);
  return metric;
}

/**
 * Assert window bounds match expectations
 */
function assertBounds(
  bounds: WindowBounds,
  expectedWidth: number,
  expectedHeight: number | { min: number; max: number },
  context: string
) {
  if (bounds.width !== expectedWidth) {
    throw new Error(`${context}: width mismatch - expected ${expectedWidth}, got ${bounds.width}`);
  }
  
  if (typeof expectedHeight === 'number') {
    if (bounds.height !== expectedHeight) {
      throw new Error(`${context}: height mismatch - expected ${expectedHeight}, got ${bounds.height}`);
    }
  } else {
    if (bounds.height < expectedHeight.min || bounds.height > expectedHeight.max) {
      throw new Error(
        `${context}: height out of range - expected ${expectedHeight.min}-${expectedHeight.max}, got ${bounds.height}`
      );
    }
  }
}

// =============================================================================
// Tests
// =============================================================================

debug('test-prompt-transitions.ts starting...');

// -----------------------------------------------------------------------------
// Test: Initial bounds check
// -----------------------------------------------------------------------------

await runTest('initial-bounds', async () => {
  const bounds = await getWindowBounds();
  debug(`Initial bounds: ${JSON.stringify(bounds)}`);
  
  // Width should always be WINDOW_WIDTH
  if (bounds.width !== WINDOW_WIDTH) {
    throw new Error(`Initial width should be ${WINDOW_WIDTH}, got ${bounds.width}`);
  }
  
  // Height should be in valid range
  if (bounds.height < MIN_HEIGHT || bounds.height > MAX_HEIGHT) {
    throw new Error(
      `Initial height ${bounds.height} out of valid range [${MIN_HEIGHT}, ${MAX_HEIGHT}]`
    );
  }
});

// -----------------------------------------------------------------------------
// Test: arg -> editor transition (compact to full)
// -----------------------------------------------------------------------------

await runTest('arg-to-editor-transition', async () => {
  // Start with arg (with choices - should be > MIN_HEIGHT)
  await arg('Select option', ['Option A', 'Option B', 'Option C']);
  await wait(RESIZE_SETTLE_TIME);
  
  const argBounds = await getWindowBounds();
  debug(`arg bounds: ${JSON.stringify(argBounds)}`);
  
  assertBounds(argBounds, WINDOW_WIDTH, { min: MIN_HEIGHT, max: MAX_HEIGHT }, 'arg with choices');
  
  // Transition to editor (should be MAX_HEIGHT)
  await editor('// Test content\nconsole.log("hello");', 'javascript');
  await wait(RESIZE_SETTLE_TIME);
  
  const editorBounds = await getWindowBounds();
  debug(`editor bounds: ${JSON.stringify(editorBounds)}`);
  
  assertBounds(editorBounds, WINDOW_WIDTH, MAX_HEIGHT, 'editor');
});

// -----------------------------------------------------------------------------
// Test: editor -> div transition (full to full)
// -----------------------------------------------------------------------------

await runTest('editor-to-div-transition', async () => {
  await editor('Test content', 'text');
  await wait(RESIZE_SETTLE_TIME);
  
  const editorBounds = await getWindowBounds();
  assertBounds(editorBounds, WINDOW_WIDTH, MAX_HEIGHT, 'editor');
  
  // Transition to div (should also be MAX_HEIGHT)
  await div('<h1>Test HTML</h1><p>This is a test paragraph with some content.</p>');
  await wait(RESIZE_SETTLE_TIME);
  
  const divBounds = await getWindowBounds();
  debug(`div bounds: ${JSON.stringify(divBounds)}`);
  
  assertBounds(divBounds, WINDOW_WIDTH, MAX_HEIGHT, 'div');
});

// -----------------------------------------------------------------------------
// Test: div -> arg transition (full to compact/medium)
// -----------------------------------------------------------------------------

await runTest('div-to-arg-transition', async () => {
  await div('<h1>Starting with div</h1>');
  await wait(RESIZE_SETTLE_TIME);
  
  const divBounds = await getWindowBounds();
  assertBounds(divBounds, WINDOW_WIDTH, MAX_HEIGHT, 'div');
  
  // Transition to arg with choices
  await arg('Back to arg', ['Choice 1', 'Choice 2']);
  await wait(RESIZE_SETTLE_TIME);
  
  const argBounds = await getWindowBounds();
  debug(`arg bounds after div: ${JSON.stringify(argBounds)}`);
  
  assertBounds(argBounds, WINDOW_WIDTH, { min: MIN_HEIGHT, max: MAX_HEIGHT }, 'arg after div');
});

// -----------------------------------------------------------------------------
// Test: Full cycle arg -> editor -> div -> arg with timing
// -----------------------------------------------------------------------------

await runTest('full-cycle-transitions', async () => {
  const metrics: TransitionMetric[] = [];
  
  // Start with arg
  debug('Starting full cycle test');
  
  // 1. arg -> editor
  let metric = await measureTransition(
    'arg',
    'editor',
    async () => {
      await arg('Step 1: arg', ['A', 'B', 'C']);
      await wait(RESIZE_SETTLE_TIME);
      await editor('Step 2: editor content', 'javascript');
    },
    MAX_HEIGHT
  );
  metrics.push(metric);
  
  // 2. editor -> div
  metric = await measureTransition(
    'editor',
    'div',
    async () => await div('<h1>Step 3: div content</h1>'),
    MAX_HEIGHT
  );
  metrics.push(metric);
  
  // 3. div -> arg
  metric = await measureTransition(
    'div',
    'arg',
    async () => await arg('Step 4: back to arg', ['X', 'Y', 'Z']),
    { min: MIN_HEIGHT, max: MAX_HEIGHT }
  );
  metrics.push(metric);
  
  // Verify all transitions had correct dimensions
  for (const m of metrics) {
    if (!m.width_match) {
      throw new Error(`Width mismatch during ${m.from} -> ${m.to} transition`);
    }
    if (!m.height_match) {
      throw new Error(
        `Height mismatch during ${m.from} -> ${m.to} transition: expected ~${m.expected_height}, got ${m.final_bounds.height}`
      );
    }
  }
  
  debug(`Full cycle completed with ${metrics.length} transitions`);
});

// -----------------------------------------------------------------------------
// Test: Rapid transitions stress test
// -----------------------------------------------------------------------------

await runTest('rapid-transitions-stress', async () => {
  const iterations = 5;
  const errors: string[] = [];
  
  debug(`Starting rapid transition stress test with ${iterations} iterations`);
  
  for (let i = 0; i < iterations; i++) {
    debug(`Iteration ${i + 1}/${iterations}`);
    
    // Rapid arg -> editor -> div sequence
    await arg(`Rapid ${i}: arg`, ['Fast A', 'Fast B']);
    await wait(50); // Shorter wait to stress test
    
    let bounds = await getWindowBounds();
    if (bounds.width !== WINDOW_WIDTH) {
      errors.push(`Iteration ${i} arg: width ${bounds.width} != ${WINDOW_WIDTH}`);
    }
    
    await editor(`Rapid ${i}: editor content`, 'text');
    await wait(50);
    
    bounds = await getWindowBounds();
    if (bounds.width !== WINDOW_WIDTH) {
      errors.push(`Iteration ${i} editor: width ${bounds.width} != ${WINDOW_WIDTH}`);
    }
    if (bounds.height !== MAX_HEIGHT) {
      errors.push(`Iteration ${i} editor: height ${bounds.height} != ${MAX_HEIGHT}`);
    }
    
    await div(`<h1>Rapid ${i}: div</h1>`);
    await wait(50);
    
    bounds = await getWindowBounds();
    if (bounds.width !== WINDOW_WIDTH) {
      errors.push(`Iteration ${i} div: width ${bounds.width} != ${WINDOW_WIDTH}`);
    }
    if (bounds.height !== MAX_HEIGHT) {
      errors.push(`Iteration ${i} div: height ${bounds.height} != ${MAX_HEIGHT}`);
    }
  }
  
  if (errors.length > 0) {
    throw new Error(`Rapid transition errors:\n${errors.join('\n')}`);
  }
  
  debug(`Rapid stress test completed successfully`);
});

// -----------------------------------------------------------------------------
// Test: Compact to full height transitions
// -----------------------------------------------------------------------------

await runTest('compact-to-full-height', async () => {
  // arg with no choices should be at MIN_HEIGHT (compact mode)
  await arg('Compact mode - no choices');
  await wait(RESIZE_SETTLE_TIME);
  
  const compactBounds = await getWindowBounds();
  debug(`Compact bounds (no choices): ${JSON.stringify(compactBounds)}`);
  
  // Height should be MIN_HEIGHT for compact mode
  if (compactBounds.height > MIN_HEIGHT + 50) { // Allow small tolerance
    debug(`Note: compact height ${compactBounds.height} is larger than expected MIN_HEIGHT ${MIN_HEIGHT}`);
  }
  
  // Transition to editor (MAX_HEIGHT)
  await editor('Full height editor', 'text');
  await wait(RESIZE_SETTLE_TIME);
  
  const fullBounds = await getWindowBounds();
  debug(`Full bounds (editor): ${JSON.stringify(fullBounds)}`);
  
  assertBounds(fullBounds, WINDOW_WIDTH, MAX_HEIGHT, 'editor full height');
  
  // Measure the height delta
  const heightDelta = fullBounds.height - compactBounds.height;
  debug(`Height delta (compact -> full): ${heightDelta}px`);
  
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'height_transition',
    compact_height: compactBounds.height,
    full_height: fullBounds.height,
    delta: heightDelta,
    expected_delta: MAX_HEIGHT - MIN_HEIGHT,
    timestamp: new Date().toISOString()
  }));
});

// -----------------------------------------------------------------------------
// Test: Timing metrics for resize operations
// -----------------------------------------------------------------------------

await runTest('resize-timing-metrics', async () => {
  const timings: { prompt: string; transition_ms: number; total_ms: number }[] = [];
  
  // Measure arg timing
  let start = Date.now();
  await arg('Timing test: arg', ['T1', 'T2', 'T3']);
  let transitionEnd = Date.now();
  await wait(RESIZE_SETTLE_TIME);
  await getWindowBounds();
  let totalEnd = Date.now();
  timings.push({
    prompt: 'arg',
    transition_ms: transitionEnd - start,
    total_ms: totalEnd - start
  });
  
  // Measure editor timing
  start = Date.now();
  await editor('Timing test: editor', 'text');
  transitionEnd = Date.now();
  await wait(RESIZE_SETTLE_TIME);
  await getWindowBounds();
  totalEnd = Date.now();
  timings.push({
    prompt: 'editor',
    transition_ms: transitionEnd - start,
    total_ms: totalEnd - start
  });
  
  // Measure div timing
  start = Date.now();
  await div('<h1>Timing test: div</h1>');
  transitionEnd = Date.now();
  await wait(RESIZE_SETTLE_TIME);
  await getWindowBounds();
  totalEnd = Date.now();
  timings.push({
    prompt: 'div',
    transition_ms: transitionEnd - start,
    total_ms: totalEnd - start
  });
  
  // Output timing summary
  for (const t of timings) {
    console.log(JSON.stringify({
      type: 'metric',
      metric: 'prompt_timing',
      prompt: t.prompt,
      transition_ms: t.transition_ms,
      total_with_resize_ms: t.total_ms,
      resize_settle_time_ms: RESIZE_SETTLE_TIME,
      timestamp: new Date().toISOString()
    }));
  }
  
  // Calculate averages
  const avgTransition = timings.reduce((sum, t) => sum + t.transition_ms, 0) / timings.length;
  const avgTotal = timings.reduce((sum, t) => sum + t.total_ms, 0) / timings.length;
  
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'timing_summary',
    avg_transition_ms: Math.round(avgTransition),
    avg_total_ms: Math.round(avgTotal),
    samples: timings.length,
    timestamp: new Date().toISOString()
  }));
  
  debug(`Timing test completed: avg transition=${Math.round(avgTransition)}ms, avg total=${Math.round(avgTotal)}ms`);
});

// -----------------------------------------------------------------------------
// Test: Width stability across all transitions
// -----------------------------------------------------------------------------

await runTest('width-stability', async () => {
  const widthReadings: { prompt: string; width: number }[] = [];
  
  // Test width after each prompt type
  await arg('Width test: arg', ['W1', 'W2']);
  await wait(RESIZE_SETTLE_TIME);
  let bounds = await getWindowBounds();
  widthReadings.push({ prompt: 'arg', width: bounds.width });
  
  await editor('Width test: editor', 'text');
  await wait(RESIZE_SETTLE_TIME);
  bounds = await getWindowBounds();
  widthReadings.push({ prompt: 'editor', width: bounds.width });
  
  await div('<h1>Width test: div</h1>');
  await wait(RESIZE_SETTLE_TIME);
  bounds = await getWindowBounds();
  widthReadings.push({ prompt: 'div', width: bounds.width });
  
  await mini('Width test: mini', ['M1', 'M2']);
  await wait(RESIZE_SETTLE_TIME);
  bounds = await getWindowBounds();
  widthReadings.push({ prompt: 'mini', width: bounds.width });
  
  await micro('Width test: micro', ['X1', 'X2']);
  await wait(RESIZE_SETTLE_TIME);
  bounds = await getWindowBounds();
  widthReadings.push({ prompt: 'micro', width: bounds.width });
  
  // Check all widths match WINDOW_WIDTH
  const widthErrors = widthReadings.filter(r => r.width !== WINDOW_WIDTH);
  
  if (widthErrors.length > 0) {
    throw new Error(
      `Width instability detected:\n${widthErrors.map(e => `  ${e.prompt}: ${e.width}`).join('\n')}`
    );
  }
  
  debug(`Width stability confirmed across ${widthReadings.length} prompt types`);
  
  console.log(JSON.stringify({
    type: 'metric',
    metric: 'width_stability',
    all_correct: true,
    expected_width: WINDOW_WIDTH,
    readings: widthReadings,
    timestamp: new Date().toISOString()
  }));
});

debug('test-prompt-transitions.ts completed!');
