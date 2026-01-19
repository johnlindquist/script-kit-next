// Test: Confirm dialog Tab key handling
// Shows confirm dialog and waits so you can press Tab to test focus switching

import '../../scripts/kit-sdk';

console.error('[TEST] Starting confirm dialog Tab test');
console.error('[TEST] Press Tab to switch button focus');
console.error('[TEST] Press Enter/Space to confirm, Escape to cancel');

// Show the confirm dialog
const result = await confirm('Do you want to proceed? (Press Tab to switch focus)', 'Yes', 'No');

console.error(`[TEST] Result: ${result}`);
process.exit(0);
