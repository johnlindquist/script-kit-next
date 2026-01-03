// Name: Template SDK Test
// Description: Tests the template() function with snippet expansion

/**
 * SMOKE TEST: test-editor-template.ts
 * 
 * Tests the template() SDK function which:
 * - Parses VSCode snippet syntax ($1, ${1:placeholder}, etc.)
 * - Opens an editor with the template content
 * - Enables Tab/Shift+Tab navigation between tabstops
 * - Returns the final edited content on submit
 * 
 * The template() function sends: {type: 'editor', template: '...', language: '...'}
 * 
 * Expected behavior:
 * - Editor opens with template content expanded
 * - Placeholders are highlighted/selectable
 * - User can navigate between tabstops
 * - Cmd+Enter submits the final content
 */

import '../../scripts/kit-sdk';

export const metadata = {
  name: "Template SDK Test",
  description: "Tests the template() function with snippet expansion"
};

console.error('[SMOKE] test-editor-template.ts starting...');
console.error('[SMOKE] template function available:', typeof template);

// Test 1: Simple template with a single placeholder
console.error('[TEST] Running template with simple placeholder...');
const simpleResult = await template('Hello ${1:World}!', { language: 'plaintext' });

console.error(JSON.stringify({
  test: 'template-simple-placeholder',
  status: simpleResult ? 'pass' : 'fail',
  resultLength: simpleResult?.length ?? 0,
  result: simpleResult
}));

console.error('[SMOKE] test-editor-template.ts completed!');
