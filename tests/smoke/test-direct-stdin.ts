// Direct stdin test - no SDK preload

import * as readline from 'node:readline';

console.error('[TEST] test-direct-stdin.ts starting...');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

console.error('[TEST] readline interface created');

let messageCount = 0;
rl.on('line', (line: string) => {
  messageCount++;
  console.error('[TEST] Received line #' + messageCount + ': ' + line);
});

rl.on('close', () => {
  console.error('[TEST] stdin closed');
});

// Send a fileSearch message
const message = { type: 'fileSearch', requestId: '999', query: 'test' };
process.stdout.write(JSON.stringify(message) + '\n');
console.error('[TEST] Sent: ' + JSON.stringify(message));

// Wait for response
setTimeout(() => {
  console.error('[TEST] Timeout - received ' + messageCount + ' messages');
  process.exit(0);
}, 2000);
