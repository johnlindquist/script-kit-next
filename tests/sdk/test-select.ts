// Test: Multi-select prompt
// Usage: npx tsx tests/sdk/test-select.ts

import '../../scripts/kit-sdk.ts';

const selected = await select('Choose your favorite fruits', [
  'Apple',
  'Banana',
  { name: 'Cherry', value: 'cherry', description: 'Sweet and red' },
  { name: 'Date', value: 'date', description: 'Sweet and chewy' },
  'Elderberry',
]);

console.log('Selected fruits:', selected);
