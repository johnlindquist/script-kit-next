// Name: Smoke Test - arg text submit
// Description: Verifies arg can submit typed text when no choices provided

/**
 * SMOKE TEST: test-arg-text-submit.ts
 * 
 * This script tests that arg() can submit typed text when no choices are provided.
 * This is the primary use case for text input prompts.
 * 
 * Expected behavior:
 * 1. Window appears with input field
 * 2. User can type text
 * 3. Pressing Enter submits the typed text
 * 4. Script receives the text and outputs it
 * 
 * Usage:
 *   echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-arg-text-submit.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 * 
 * Expected exit code: 0
 */

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-arg-text-submit.ts starting...');

// Test 1: arg with only placeholder, no choices parameter
console.error('[SMOKE] Test 1: arg("placeholder") with no choices parameter');
const result1 = await arg("Type something and press Enter");
console.error(`[SMOKE] Result 1: "${result1}"`);

if (result1 && result1.length > 0) {
  console.error('[SMOKE] SUCCESS - arg submitted text without choices');
} else {
  console.error('[SMOKE] WARNING - result was empty or undefined');
}

// Test 2: arg with empty choices array (different path)
console.error('[SMOKE] Test 2: arg with empty choices array');
const result2 = await arg("Enter another value", []);
console.error(`[SMOKE] Result 2: "${result2}"`);

// Test 3: arg with empty string placeholder
console.error('[SMOKE] Test 3: arg with minimal placeholder');
const result3 = await arg("");
console.error(`[SMOKE] Result 3: "${result3}"`);

// Summary
console.error('[SMOKE] All tests completed');
console.error('[SMOKE] Results summary:');
console.error(`[SMOKE]   Test 1 (no choices): "${result1}"`);
console.error(`[SMOKE]   Test 2 (empty array): "${result2}"`);
console.error(`[SMOKE]   Test 3 (empty placeholder): "${result3}"`);

console.error('[SMOKE] test-arg-text-submit.ts completed');
