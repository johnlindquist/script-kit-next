// Test script for verifying window positioning on mouse display
// Run with: echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-window-positioning.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep -i position

import '../../scripts/kit-sdk';

console.error('[TEST] Window positioning test starting');
console.error('[TEST] This should appear on the display where your mouse cursor is located');

// Show a simple prompt to verify the window appeared
const result = await arg({
  placeholder: "Window positioning test",
  hint: "Press Enter to close - check logs for POSITION entries",
  choices: [
    { name: "Window appeared on correct display", value: "correct" },
    { name: "Window appeared on WRONG display", value: "wrong" },
  ],
});

console.error(`[TEST] User selected: ${result}`);
console.error('[TEST] Window positioning test complete');

process.exit(0);
