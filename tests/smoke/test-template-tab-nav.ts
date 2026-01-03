// Name: Template Tab Navigation Test
// Description: Tests Tab key navigation in template mode via SimulateKey

import '../../scripts/kit-sdk';

export const metadata = {
  name: "Template Tab Navigation Test",
  description: "Tests Tab key navigation in template mode"
};

console.error('[TEST] Starting template Tab navigation test...');
console.error('[TEST] This test opens a template editor.');
console.error('[TEST] Use SimulateKey {"type":"simulateKey","key":"tab"} to test Tab navigation.');

// Set a timeout to exit after 10 seconds
setTimeout(() => {
  console.error('[TEST] Timeout reached, exiting...');
  process.exit(0);
}, 10000);

console.error('[TEST] Calling template()...');

// This opens the editor with template tabstops
// Expected: "name" should be selected initially
// After Tab: "place" should be selected
const result = await template('Hello ${1:name}, welcome to ${2:place}!');

console.error('[TEST] Template completed with result:', result);
process.exit(0);
