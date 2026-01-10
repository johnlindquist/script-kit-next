import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

// Test the confirm() modal

async function main() {
  console.error('[TEST] Starting confirm modal test');

  // Test 1: Basic confirm with default buttons
  console.error('[TEST] Test 1: Basic confirm dialog');
  const result1 = await confirm("Are you sure you want to proceed?");
  console.error(`[TEST] Result 1: ${result1}`);

  // Test 2: Confirm with custom button text
  console.error('[TEST] Test 2: Custom button text');
  const result2 = await confirm({
    message: "Delete this important file?",
    confirmText: "Yes, Delete It",
    cancelText: "No, Keep It"
  });
  console.error(`[TEST] Result 2: ${result2}`);

  // Test 3: Shorthand with custom buttons
  console.error('[TEST] Test 3: Shorthand syntax');
  const result3 = await confirm("Save changes before closing?", "Save", "Don't Save");
  console.error(`[TEST] Result 3: ${result3}`);

  console.error('[TEST] All confirm tests completed');
  console.error(`[TEST] Results: test1=${result1}, test2=${result2}, test3=${result3}`);

  process.exit(0);
}

main().catch((err) => {
  console.error('[TEST] Error:', err);
  process.exit(1);
});
