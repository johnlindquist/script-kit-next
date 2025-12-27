// Name: Toast Variants Smoke Test
// Description: Tests different error types that trigger toast notifications

/**
 * SMOKE TEST: test-toast-variants.ts
 * 
 * This script tests multiple error scenarios to verify the toast system
 * handles different error types correctly.
 * 
 * Usage:
 *   echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-toast-variants.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 */

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-toast-variants.ts starting...');

// Show a prompt to select which error type to test
const errorType = await arg('Select error type to test:', [
  { name: 'Standard Error', value: 'standard', description: 'throw new Error()' },
  { name: 'Module Not Found', value: 'module', description: 'Simulates missing import' },
  { name: 'Syntax-like Error', value: 'syntax', description: 'Simulates syntax issue' },
  { name: 'Permission Error', value: 'permission', description: 'Simulates EACCES' },
  { name: 'No Error (Success)', value: 'success', description: 'Shows success case' },
]);

console.error(`[SMOKE] Selected error type: ${errorType}`);

switch (errorType) {
  case 'standard':
    throw new Error('Standard error: Something went wrong');
    
  case 'module':
    // Simulate a module not found error message
    throw new Error("Cannot find module 'nonexistent-package'");
    
  case 'syntax':
    // Simulate a syntax-related error
    throw new SyntaxError('Unexpected token at line 42');
    
  case 'permission': {
    // Simulate a permission error
    const err: Error & { code?: string } = new Error('EACCES: permission denied, open /etc/passwd');
    err.code = 'EACCES';
    throw err;
  }
    
  case 'success':
    console.error('[SMOKE] No error - displaying success div');
    await div(md(`# Success!
    
Toast variant test completed without errors.

This is the control case to verify the app handles successful exits correctly.`));
    break;
}

console.error('[SMOKE] test-toast-variants.ts completed');
