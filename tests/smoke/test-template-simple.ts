// Name: Simple Template Test
// Description: Simple test for template - auto-exits after delay

import '../../scripts/kit-sdk';

export const metadata = {
  name: "Simple Template Test",
  description: "Opens template editor and exits after delay"
};

console.error('[TEST] Starting simple template test...');

// Set a timeout to exit after 3 seconds (enough to see the editor and press Tab)
setTimeout(() => {
  console.error('[TEST] Timeout reached, exiting...');
  process.exit(0);
}, 5000);

console.error('[TEST] Calling template()...');

// This will open the editor - user can press Tab to test navigation
// Then it will timeout and exit
const result = await template('Hello ${1:name}, welcome to ${2:place}!');

console.error('[TEST] Got result:', result);
process.exit(0);
