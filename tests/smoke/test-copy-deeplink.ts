// Test script for verifying copy_deeplink action
// This script is used to test that the copy_deeplink action works correctly
import '../../scripts/kit-sdk';

export const metadata = {
  name: "Test Copy Deeplink",
  description: "Test script for copy_deeplink action",
};

// Simple arg prompt that shows a choice
// User can then trigger actions via Cmd+K to test copy_deeplink
const result = await arg("Select an item to test actions", [
  { name: "Item One", value: "one" },
  { name: "Item Two", value: "two" },
  { name: "Item Three", value: "three" },
]);

console.error(`Selected: ${result}`);
process.exit(0);
