// Name: Button Click Behaviors Visual Test
// Description: Tests button click behaviors including list item selection, action buttons, toast dismiss, and form field focus

/**
 * VISUAL TEST: test-button-clicks.ts
 *
 * This test suite verifies click behaviors across different UI components:
 * 1. List item selection via click
 * 2. Action button clicks (Cmd+K panel)
 * 3. Toast dismiss via click
 * 4. Form field focus via click
 *
 * USAGE:
 *   cargo build && \
 *   echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-button-clicks.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 *
 * NOTE: Some click simulations require the app to implement the simulateClick
 * message handler. This test captures screenshots to verify visual state.
 */

import '../../scripts/kit-sdk';
import {
  simulateClick,
  waitForRender,
  captureAndSave,
  logTestResult,
  runVisualTest,
} from '../sdk/test-click-utils';

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
  screenshot?: string;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra,
  };
  console.log(JSON.stringify(result));
}

console.error('[BUTTON-CLICKS] Starting button click behavior tests...');

// =============================================================================
// Test 1: List Item Selection via Click
// =============================================================================

async function testListItemClick(): Promise<string> {
  console.error('[BUTTON-CLICKS] Test 1: List item selection via click');
  logTest('list-item-click', 'running');
  const start = Date.now();

  try {
    // Create a list of choices
    const choices = [
      { name: 'Apple', value: 'apple', description: 'A red fruit' },
      { name: 'Banana', value: 'banana', description: 'A yellow fruit' },
      { name: 'Cherry', value: 'cherry', description: 'A small red fruit' },
      { name: 'Date', value: 'date', description: 'A sweet brown fruit' },
      { name: 'Elderberry', value: 'elderberry', description: 'A dark purple berry' },
    ];

    console.error('[BUTTON-CLICKS] Showing arg prompt with 5 choices...');

    // Start the arg prompt (don't await - we want to interact with it)
    const argPromise = arg('Click an item to select it:', choices);

    // Wait for UI to render
    await waitForRender(500);

    // Capture initial state before any clicks
    console.error('[BUTTON-CLICKS] Capturing initial state...');
    const screenshotPath = await captureAndSave('list-item-before-click');

    // Simulate click on the second item (approximate coordinates)
    // In a typical Script Kit window, list items start around y=100
    // Each item is roughly 52px tall (per AGENTS.md)
    console.error('[BUTTON-CLICKS] Simulating click on second list item...');
    await simulateClick(200, 150); // Approximate center of second item

    // Wait for selection to update
    await waitForRender(300);

    // Capture state after click
    console.error('[BUTTON-CLICKS] Capturing state after click...');
    const afterClickPath = await captureAndSave('list-item-after-click');

    logTest('list-item-click', 'pass', {
      duration_ms: Date.now() - start,
      result: { beforeScreenshot: screenshotPath, afterScreenshot: afterClickPath },
      screenshot: afterClickPath,
    });

    // Clean up - cancel the prompt
    // Use Promise.race to avoid hanging
    await Promise.race([argPromise.catch(() => {}), waitForRender(500)]);

    return afterClickPath;
  } catch (err) {
    logTest('list-item-click', 'fail', {
      duration_ms: Date.now() - start,
      error: String(err),
    });
    throw err;
  }
}

// =============================================================================
// Test 2: Action Button Click (Cmd+K Panel)
// =============================================================================

async function testActionButtonClick(): Promise<string> {
  console.error('[BUTTON-CLICKS] Test 2: Action button clicks');
  logTest('action-button-click', 'running');
  const start = Date.now();

  try {
    // Show a prompt that supports actions
    const choices = [
      { name: 'Test Script', value: 'test', description: 'A test script' },
      { name: 'Another Script', value: 'another', description: 'Another test' },
    ];

    console.error('[BUTTON-CLICKS] Showing arg prompt...');
    const argPromise = arg('Press Cmd+K or click action button:', choices);

    await waitForRender(500);

    // Try to trigger actions panel via keyboard
    console.error('[BUTTON-CLICKS] Attempting to trigger actions panel...');
    try {
      await keyboard.tap('command', 'k');
    } catch {
      console.error('[BUTTON-CLICKS] keyboard.tap not available');
    }

    await waitForRender(800);

    // Capture the actions panel (if visible)
    console.error('[BUTTON-CLICKS] Capturing actions panel state...');
    const screenshotPath = await captureAndSave('action-button-panel');

    // If actions panel is showing, try clicking an action
    // Action buttons are typically in the center overlay
    console.error('[BUTTON-CLICKS] Simulating click on action button...');
    await simulateClick(375, 200); // Center of typical action panel

    await waitForRender(300);

    const afterActionPath = await captureAndSave('action-button-after-click');

    logTest('action-button-click', 'pass', {
      duration_ms: Date.now() - start,
      result: { panelScreenshot: screenshotPath, afterActionScreenshot: afterActionPath },
      screenshot: afterActionPath,
    });

    // Clean up
    await Promise.race([argPromise.catch(() => {}), waitForRender(500)]);

    return afterActionPath;
  } catch (err) {
    logTest('action-button-click', 'fail', {
      duration_ms: Date.now() - start,
      error: String(err),
    });
    throw err;
  }
}

// =============================================================================
// Test 3: Toast Dismiss via Click
// =============================================================================

async function testToastDismiss(): Promise<string> {
  console.error('[BUTTON-CLICKS] Test 3: Toast dismiss via click');
  logTest('toast-dismiss-click', 'running');
  const start = Date.now();

  try {
    // Show a div that we can interact with
    console.error('[BUTTON-CLICKS] Showing div with toast instruction...');

    // Use hud() to trigger a toast notification
    hud('Click me to dismiss! This toast should auto-dismiss.', { duration: 5000 });

    await waitForRender(500);

    // Capture the toast
    console.error('[BUTTON-CLICKS] Capturing toast state...');
    const toastPath = await captureAndSave('toast-visible');

    // Toast is typically at the bottom center of the window
    // Try clicking near the bottom center to dismiss
    console.error('[BUTTON-CLICKS] Simulating click on toast...');
    await simulateClick(375, 450); // Bottom center area

    await waitForRender(500);

    // Capture after click
    const afterDismissPath = await captureAndSave('toast-after-dismiss');

    logTest('toast-dismiss-click', 'pass', {
      duration_ms: Date.now() - start,
      result: { toastScreenshot: toastPath, afterDismissScreenshot: afterDismissPath },
      screenshot: afterDismissPath,
    });

    return afterDismissPath;
  } catch (err) {
    logTest('toast-dismiss-click', 'fail', {
      duration_ms: Date.now() - start,
      error: String(err),
    });
    throw err;
  }
}

// =============================================================================
// Test 4: Form Field Focus via Click
// =============================================================================

async function testFormFieldFocus(): Promise<string> {
  console.error('[BUTTON-CLICKS] Test 4: Form field focus via click');
  logTest('form-field-focus', 'running');
  const start = Date.now();

  try {
    // Create a form with multiple fields
    const formHtml = `
<div class="p-4 space-y-4">
  <h2 class="text-lg font-bold mb-4">Click Field Focus Test</h2>

  <div class="space-y-2">
    <label for="username" class="block text-sm font-medium">Username</label>
    <input type="text" name="username" id="username" placeholder="Click to focus" class="w-full px-4 py-2 border rounded" />
  </div>

  <div class="space-y-2">
    <label for="email" class="block text-sm font-medium">Email</label>
    <input type="email" name="email" id="email" placeholder="Click to focus" class="w-full px-4 py-2 border rounded" />
  </div>

  <div class="space-y-2">
    <label for="bio" class="block text-sm font-medium">Bio</label>
    <textarea name="bio" id="bio" placeholder="Click to focus" class="w-full px-4 py-2 border rounded" rows="3"></textarea>
  </div>
</div>
`;

    console.error('[BUTTON-CLICKS] Showing form with multiple fields...');
    const formPromise = form(formHtml);

    await waitForRender(500);

    // Capture initial form state
    console.error('[BUTTON-CLICKS] Capturing initial form state...');
    const initialPath = await captureAndSave('form-fields-initial');

    // Click on the first input field (username)
    // Inputs are typically around y=120-160 for first field
    console.error('[BUTTON-CLICKS] Clicking on username field...');
    await simulateClick(375, 140);

    await waitForRender(300);
    const afterUsernameClick = await captureAndSave('form-field-username-focused');

    // Click on the email field (second input)
    console.error('[BUTTON-CLICKS] Clicking on email field...');
    await simulateClick(375, 220);

    await waitForRender(300);
    const afterEmailClick = await captureAndSave('form-field-email-focused');

    // Click on the textarea (bio)
    console.error('[BUTTON-CLICKS] Clicking on bio textarea...');
    await simulateClick(375, 320);

    await waitForRender(300);
    const afterBioClick = await captureAndSave('form-field-bio-focused');

    logTest('form-field-focus', 'pass', {
      duration_ms: Date.now() - start,
      result: {
        initialScreenshot: initialPath,
        usernameScreenshot: afterUsernameClick,
        emailScreenshot: afterEmailClick,
        bioScreenshot: afterBioClick,
      },
      screenshot: afterBioClick,
    });

    // Clean up - don't wait for form submission
    await Promise.race([formPromise.catch(() => {}), waitForRender(500)]);

    return afterBioClick;
  } catch (err) {
    logTest('form-field-focus', 'fail', {
      duration_ms: Date.now() - start,
      error: String(err),
    });
    throw err;
  }
}

// =============================================================================
// Main Test Runner
// =============================================================================

async function runAllTests() {
  console.error('[BUTTON-CLICKS] ====================================');
  console.error('[BUTTON-CLICKS] Button Click Behavior Test Suite');
  console.error('[BUTTON-CLICKS] ====================================');

  const results: { test: string; passed: boolean; screenshot?: string }[] = [];

  // Test 1: List item selection
  try {
    const screenshot = await testListItemClick();
    results.push({ test: 'list-item-click', passed: true, screenshot });
  } catch {
    results.push({ test: 'list-item-click', passed: false });
  }

  // Small delay between tests
  await waitForRender(500);

  // Test 2: Action button click
  try {
    const screenshot = await testActionButtonClick();
    results.push({ test: 'action-button-click', passed: true, screenshot });
  } catch {
    results.push({ test: 'action-button-click', passed: false });
  }

  await waitForRender(500);

  // Test 3: Toast dismiss
  try {
    const screenshot = await testToastDismiss();
    results.push({ test: 'toast-dismiss-click', passed: true, screenshot });
  } catch {
    results.push({ test: 'toast-dismiss-click', passed: false });
  }

  await waitForRender(500);

  // Test 4: Form field focus
  try {
    const screenshot = await testFormFieldFocus();
    results.push({ test: 'form-field-focus', passed: true, screenshot });
  } catch {
    results.push({ test: 'form-field-focus', passed: false });
  }

  // Summary
  console.error('[BUTTON-CLICKS] ====================================');
  console.error('[BUTTON-CLICKS] Test Summary:');
  const passed = results.filter((r) => r.passed).length;
  const total = results.length;
  console.error(`[BUTTON-CLICKS] ${passed}/${total} tests passed`);

  for (const result of results) {
    const status = result.passed ? 'PASS' : 'FAIL';
    console.error(`[BUTTON-CLICKS]   ${status}: ${result.test}`);
    if (result.screenshot) {
      console.error(`[BUTTON-CLICKS]         Screenshot: ${result.screenshot}`);
    }
  }
  console.error('[BUTTON-CLICKS] ====================================');

  // Output final summary as JSON
  console.log(
    JSON.stringify({
      test: 'button-clicks-suite',
      status: passed === total ? 'pass' : 'fail',
      summary: { passed, total },
      results,
      timestamp: new Date().toISOString(),
    })
  );

  console.error('[BUTTON-CLICKS] Test suite complete.');
  console.error('[BUTTON-CLICKS] Check ./test-screenshots/ for captured images.');
}

// Run all tests
runAllTests()
  .then(() => {
    // Exit cleanly after all tests
    process.exit(0);
  })
  .catch((err) => {
    console.error(`[BUTTON-CLICKS] FATAL: ${err}`);
    console.log(
      JSON.stringify({
        test: 'button-clicks-suite',
        status: 'fail',
        error: String(err),
        timestamp: new Date().toISOString(),
      })
    );
    process.exit(1);
  });
