// Name: Error Display Smoke Test
// Description: Tests that script errors display as toast notifications

/**
 * SMOKE TEST: test-error-display.ts
 * 
 * This script intentionally throws an error to test the error toast system.
 * 
 * Expected behavior:
 * 1. Script throws an error
 * 2. Executor captures stderr and exit code
 * 3. Error toast is displayed with the error message
 * 4. Toast includes "Copy Error" button
 * 
 * Usage:
 *   echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-error-display.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 * 
 * Expected log output:
 *   - "Captured stderr" or similar
 *   - "Pushing error toast" or similar
 *   - Error toast rendered in UI
 */

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-error-display.ts starting...');
console.error('[SMOKE] About to throw an intentional error...');

// Wait a moment so the app window is visible
await new Promise(resolve => setTimeout(resolve, 500));

// Throw an error that the executor should capture
throw new Error('This is a test error message - the toast system should display this!');
