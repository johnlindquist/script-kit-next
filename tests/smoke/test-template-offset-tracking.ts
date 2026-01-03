// Test: Template offset tracking when user types different length text
// This verifies that tabstop 2 is correctly selected after user edits tabstop 1

import '../../scripts/kit-sdk';

export const metadata = {
  name: "Template Offset Tracking Test",
  description: "Verifies tab navigation works after editing placeholders",
};

console.error('[TEST] Starting template offset tracking test...');

// Template: "Hello ${1:name}, welcome to ${2:place}!"
// Initial text: "Hello name, welcome to place!"
//               012345678901234567890123456789
//                     ^   ^           ^    ^
//                     6   10          22   27
// Tabstop 1: [6, 10) = "name" (4 chars)
// Tabstop 2: [22, 27) = "place" (5 chars)
//
// If user types "John Doe" (8 chars) at tabstop 1:
// New text: "Hello John Doe, welcome to place!"
//           0123456789012345678901234567890123
//                 ^       ^           ^    ^
//                 6       14          26   31
// Tabstop 2 should now be at [26, 31) = "place"
// Offset shift: +4 chars (8 - 4 = +4)

const template = "Hello ${1:name}, welcome to ${2:place}!";

console.error('[TEST] Creating editor with template:', template);

await editor(template, "plaintext");

// Wait for initial render and tabstop 1 selection
await new Promise(r => setTimeout(r, 800));

console.error('[TEST] Expected: "name" should be selected (tabstop 1)');
console.error('[TEST] Next step: User would type "John Doe" to replace "name"');
console.error('[TEST] Then press Tab - tabstop 2 "place" should be selected at adjusted offset');

// Keep window open for manual testing
console.error('[TEST] Waiting for manual interaction...');
console.error('[TEST] Type something to replace "name", then press Tab');
console.error('[TEST] Verify that "place" gets selected correctly');

// Wait longer to allow manual testing
await new Promise(r => setTimeout(r, 30000));

process.exit(0);
