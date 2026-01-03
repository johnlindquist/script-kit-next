// Manual test for template offset tracking
// Run this and follow the instructions in the console

import '../../scripts/kit-sdk';

export const metadata = {
  name: "Template Manual Test",
  description: "Manual test for offset tracking",
};

console.error('');
console.error('=== TEMPLATE OFFSET TRACKING TEST ===');
console.error('');
console.error('INSTRUCTIONS:');
console.error('1. You should see "Hello name, welcome to place!" with "name" selected');
console.error('2. Type "John Doe" (or any text) to replace "name"');
console.error('3. Press Tab');
console.error('4. EXPECTED: "place" should be selected');
console.error('5. If something else is selected (or nothing), the offset tracking is broken');
console.error('');
console.error('Press Cmd+Enter when done to submit and exit');
console.error('');

const template = "Hello ${1:name}, welcome to ${2:place}!";
const result = await editor(template, "plaintext");

console.error('');
console.error('RESULT:', result);
console.error('');
