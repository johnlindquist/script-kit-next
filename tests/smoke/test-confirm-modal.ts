// Test: Confirm Modal
// Verifies that the confirm modal shows up and doesn't auto-dismiss

import '../../scripts/kit-sdk';

console.error('[TEST] Starting confirm modal test');

// Use the SDK confirm function
const result = await confirm({
    message: "Test confirm modal - click Cancel or Confirm",
    confirmText: "Confirm",
    cancelText: "Cancel"
});

console.error(`[TEST] Confirm result: ${result}`);
console.error('[TEST] If you see this, the confirm modal worked!');

// Show result
await div(`
    <div class="p-8 text-center">
        <h1 class="text-2xl mb-4">Confirm Modal Result</h1>
        <p class="text-lg">You chose: <strong>${result ? "Confirm" : "Cancel"}</strong></p>
    </div>
`);
