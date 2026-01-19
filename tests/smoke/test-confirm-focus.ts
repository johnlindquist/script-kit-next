import '../../scripts/kit-sdk';

// Test the confirm dialog's keyboard focus behavior
console.error('[TEST] Starting confirm focus test');

// Show a confirm dialog
const result = await confirm({
  message: "Test keyboard focus - press Tab then Space",
  confirmText: "Confirm",
  cancelText: "Cancel",
});

console.error(`[TEST] Confirm result: ${result}`);
process.exit(0);
