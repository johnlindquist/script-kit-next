import '../../scripts/kit-sdk';
console.error('[TEST] Starting file search simple test');
console.error('[TEST] typeof fileSearch =', typeof fileSearch);

console.error('[TEST] Calling fileSearch("package.json")...');
const results = await fileSearch('package.json');
console.error('[TEST] fileSearch returned:', results.length, 'results');

console.log(JSON.stringify({ test: 'file-search-basic', status: 'pass', result: results.length }));
exit(0);
