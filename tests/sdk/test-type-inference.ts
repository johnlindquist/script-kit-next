// Type-level tests for schema inference
// This file should compile without errors if types are working correctly

import '../../scripts/kit-sdk';

// Test: defineSchema returns typed input/output functions
const { input, output } = defineSchema({
  input: {
    greeting: { type: 'string', required: true },
    count: { type: 'number' },
  },
  output: {
    message: { type: 'string' },
    success: { type: 'boolean' },
  },
} as const);

// Test: input() should return typed object
async function testInput() {
  const result = await input();
  
  // greeting should be string (required)
  const g: string = result.greeting;
  
  // count should be number | undefined (optional)
  const c: number | undefined = result.count;
  
  // @ts-expect-error - unknown property should error
  const x = result.unknownProp;
  
  return { g, c };
}

// Test: output() should accept typed object
function testOutput() {
  // Should work - partial output
  output({ message: 'test' });
  output({ success: true });
  output({ message: 'test', success: false });
  
  // @ts-expect-error - wrong type for message
  output({ message: 123 });
  
  // @ts-expect-error - wrong type for success
  output({ success: 'yes' });
}

console.log('Type inference tests - if this compiles, types are working!');
testInput();
testOutput();
