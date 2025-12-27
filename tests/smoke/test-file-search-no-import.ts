// No SDK import - relying on preload

console.error('[TEST] test-file-search-no-import.ts starting...');
console.error('[TEST] SDK globals: fileSearch=' + (typeof fileSearch));

console.log(JSON.stringify({ test: 'fileSearch-exists', status: 'running', timestamp: new Date().toISOString() }));
console.error('[TEST] Test 1: Verify fileSearch() function exists');

if (typeof fileSearch !== 'function') {
  console.log(JSON.stringify({ test: 'fileSearch-exists', status: 'fail', error: 'fileSearch is not a function', timestamp: new Date().toISOString() }));
  exit(1);
}

console.log(JSON.stringify({ test: 'fileSearch-exists', status: 'pass', result: { fileSearch: 'function' }, timestamp: new Date().toISOString() }));

console.error('[TEST] Test 2: fileSearch("package.json") - basic search');
const results = await fileSearch('package.json');
console.error('[TEST] Test 2: Got ' + results.length + ' results');

console.log(JSON.stringify({ test: 'fileSearch-basic', status: 'pass', result: { count: results.length }, timestamp: new Date().toISOString() }));

console.error('[TEST] test-file-search-no-import.ts completed!');
exit(0);
