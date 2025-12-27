// Name: Test Arg Prompt Visual
// Description: Tests various arg prompt configurations for proper sizing

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-arg-visual.ts starting...');

// Test 1: Arg with many choices (should be 500px - STANDARD_HEIGHT)
console.error('[SMOKE] Test 1: Arg with choices');
const choices = Array.from({ length: 50 }, (_, i) => ({
  name: `Choice ${i + 1}`,
  value: `choice-${i + 1}`,
  description: `Description for choice ${i + 1}`,
}));

const result1 = await arg('Select an item (window should be 500px):', choices);
console.error('[SMOKE] Test 1 result:', result1);

// Test 2: Arg without choices (should be compact - 120px MIN_HEIGHT)
console.error('[SMOKE] Test 2: Arg without choices');
const result2 = await arg('Enter some text (window should be compact 120px):');
console.error('[SMOKE] Test 2 result:', result2);

console.error('[SMOKE] test-arg-visual.ts completed!');
