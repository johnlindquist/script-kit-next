// Minimal stdin test
console.error('[STDIN-TEST] Script starting');
console.error('[STDIN-TEST] process.stdin readable:', process.stdin.readable);
console.error('[STDIN-TEST] process.stdin isTTY:', process.stdin.isTTY);

// Log all stdin
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk: string) => {
  console.error(`[STDIN-TEST] Got data: ${chunk.length} bytes - first 100 chars: ${chunk.slice(0, 100)}`);
});

process.stdin.on('end', () => {
  console.error('[STDIN-TEST] stdin ended');
});

process.stdin.on('error', (err) => {
  console.error('[STDIN-TEST] stdin error:', err);
});

console.error('[STDIN-TEST] Waiting for stdin...');

// Keep alive for a bit
setTimeout(() => {
  console.error('[STDIN-TEST] Timeout, exiting');
  process.exit(0);
}, 3000);
