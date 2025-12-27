// Name: Test Terminal Visual Fill
// Description: Verifies that the terminal fills the 700px window height correctly

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-term-visual.ts starting...');

// Show terminal with a command that outputs content
// The terminal should fill the full 700px window
// term() takes a command string directly, not an object
const result = await term('echo "=== TERMINAL VISUAL TEST ==="; echo "Terminal should fill 700px window"; echo ""; for i in $(seq 1 40); do echo "Line $i - Testing content fill"; done; echo ""; echo "If you see empty space below, terminal is NOT filling correctly"; sleep 10');

console.error('[SMOKE] Terminal result:', result);
console.error('[SMOKE] test-term-visual.ts completed!');
