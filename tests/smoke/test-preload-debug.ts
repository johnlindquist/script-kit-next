// No import - just relying on preload

console.error('[TEST] test-preload-debug.ts starting...');
console.error('[TEST] typeof fileSearch =', typeof fileSearch);

// Add a listener to see what messages come through
// This requires us to access the SDK internals somehow
console.error('[TEST] Checking if SDK is loaded...');

// Try calling fileSearch and log what happens
console.error('[TEST] Calling fileSearch...');

const promise = fileSearch('test');
console.error('[TEST] fileSearch returned a promise');

// Set a timeout to check if it resolves
let resolved = false;
promise.then((results: any[]) => {
  resolved = true;
  console.error('[TEST] fileSearch resolved with', results.length, 'results');
}).catch((err: any) => {
  console.error('[TEST] fileSearch rejected:', err);
});

setTimeout(() => {
  if (!resolved) {
    console.error('[TEST] fileSearch did not resolve in 2 seconds');
  }
  console.error('[TEST] Exiting...');
  process.exit(0);
}, 2000);
