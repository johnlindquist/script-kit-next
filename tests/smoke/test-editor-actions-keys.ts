// Name: Test Editor Actions Key Navigation
// Description: Verify that arrow keys navigate actions panel, not editor cursor

import '../../scripts/kit-sdk';

console.error('[SMOKE] Starting editor actions key test...');

const editorContent = `function hello() {
  console.log("Hello World");
  console.log("Line 2");
  console.log("Line 3");
  console.log("Line 4");
  return 42;
}`;

// Actions passed as third argument (not options object)
const actions = [
  { name: "Action 1 - First", shortcut: "cmd+1" },
  { name: "Action 2 - Second", shortcut: "cmd+2" },
  { name: "Action 3 - Third", shortcut: "cmd+3" },
  { name: "Action 4 - Fourth", shortcut: "cmd+4" },
  { name: "Action 5 - Fifth", shortcut: "cmd+5" },
];

// Start the editor with actions: editor(content, language, actions)
const editorPromise = editor(editorContent, "typescript", actions);

console.error('[SMOKE] Editor opened with 5 actions');
console.error('[SMOKE] Instructions:');
console.error('[SMOKE]   1. Press Cmd+K to open the actions panel');
console.error('[SMOKE]   2. Press Down arrow - should move selection in actions panel');
console.error('[SMOKE]   3. Press Up arrow - should move selection in actions panel');
console.error('[SMOKE]   4. Arrow keys should NOT move the editor cursor');
console.error('[SMOKE]   5. Press Escape to close actions panel');
console.error('[SMOKE]   6. Press Cmd+Enter to submit');

const result = await editorPromise;
console.error('[SMOKE] Editor result:', result?.substring(0, 50) + '...');
console.error('[SMOKE] Test complete');
