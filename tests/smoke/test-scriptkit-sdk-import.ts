// Test: Verify @scriptkit/sdk import works
// This tests that the import redirect is configured correctly

import "@scriptkit/sdk";

// Verify the SDK is loaded by checking SDK_VERSION export
const version = (await import("@scriptkit/sdk")).SDK_VERSION;
console.log(`SDK_VERSION: ${version}`);

// Verify globals are available
console.log(`arg available: ${typeof arg}`);
console.log(`div available: ${typeof div}`);
console.log(`md available: ${typeof md}`);

// Show a simple div to confirm everything works
await div(md(`
# Import Test Passed!

- \`import "@scriptkit/sdk"\` ✓
- SDK_VERSION: ${version} ✓
- Global functions available ✓
`));
