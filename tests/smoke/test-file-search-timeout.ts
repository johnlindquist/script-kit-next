// No SDK import - relying on preload

console.error('[TEST] test-file-search-timeout.ts starting...');
console.error('[TEST] SDK globals: fileSearch=' + (typeof fileSearch));

console.error('[TEST] Test 1: Verify fileSearch() function exists');

if (typeof fileSearch !== 'function') {
  console.error('[TEST] fileSearch is not a function!');
  exit(1);
}

console.error('[TEST] fileSearch is a function');

console.error('[TEST] Test 2: fileSearch("package.json") - starting...');

// Call with promise but add a timeout
const searchPromise = fileSearch('package.json');
const timeoutPromise = new Promise((_, reject) => {
  setTimeout(() => reject(new Error('fileSearch timeout after 3s')), 3000);
});

try {
  const results = await Promise.race([searchPromise, timeoutPromise]) as any[];
  console.error('[TEST] Test 2: Got ' + results.length + ' results');
} catch (e) {
  console.error('[TEST] Error:', e);
}

console.error('[TEST] test-file-search-timeout.ts completed!');
exit(0);
