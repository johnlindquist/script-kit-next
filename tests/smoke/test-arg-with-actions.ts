// Name: Test arg() with actions
// Description: Tests the actions parameter for arg()

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-arg-with-actions starting...');

// Test 1: Basic actions with arg() 3rd argument
const choices = ['Apple', 'Banana', 'Cherry'];

// Track action calls for verification
let copyActionCalls = 0;
let persistentActionCalls = 0;

const actions = [
  {
    name: 'Copy Value',
    shortcut: 'cmd+c',
    onAction: (input: string) => {
      copyActionCalls++;
      console.error('[SMOKE] Copy action triggered with input:', input, 'calls:', copyActionCalls);
    },
  },
  {
    name: 'Preview',
    shortcut: 'cmd+p',
    onAction: (input: string) => {
      console.error('[SMOKE] Preview action triggered');
    },
  },
  {
    name: 'Open in Editor',
    description: 'Opens the selected item in default editor',
    shortcut: 'cmd+e',
    onAction: (input: string) => {
      console.error('[SMOKE] Open in Editor action triggered');
    },
  },
  // Test close: false - action should NOT close the dialog
  {
    name: 'Persistent Action',
    description: 'This action keeps the dialog open (close: false)',
    shortcut: 'cmd+shift+p',
    close: false,
    onAction: (input: string) => {
      persistentActionCalls++;
      console.error('[SMOKE] Persistent action triggered, call #:', persistentActionCalls);
      console.error('[SMOKE] Dialog should still be open!');
    },
  },
  // Test visible: false - action should be hidden from the actions panel
  {
    name: 'Hidden Action',
    description: 'This action is not visible in the actions panel',
    shortcut: 'cmd+h',
    visible: false,
    onAction: (input: string) => {
      console.error('[SMOKE] Hidden action triggered via shortcut (not from panel)');
    },
  },
];

console.error('[SMOKE] Showing arg prompt with 3 choices and 5 actions...');
console.error('[SMOKE] Actions:');
console.error('[SMOKE]   - Copy Value (cmd+c) - normal action, closes dialog');
console.error('[SMOKE]   - Preview (cmd+p) - normal action, closes dialog');
console.error('[SMOKE]   - Open in Editor (cmd+e) - normal action, closes dialog');
console.error('[SMOKE]   - Persistent Action (cmd+shift+p) - close: false, keeps dialog open');
console.error('[SMOKE]   - Hidden Action (cmd+h) - visible: false, hidden from panel');
console.error('[SMOKE] Expected behavior:');
console.error('[SMOKE]   - Cmd+K should show 4 actions (Hidden Action should NOT appear)');
console.error('[SMOKE]   - Persistent Action should keep dialog open when triggered');
console.error('[SMOKE]   - Hidden Action can still be triggered via cmd+h shortcut');

const result = await arg('Pick a fruit (press Cmd+K for actions):', choices, actions);

console.error('[SMOKE] Result:', result);
console.error('[SMOKE] Copy action was called', copyActionCalls, 'times');
console.error('[SMOKE] Persistent action was called', persistentActionCalls, 'times');
console.error('[SMOKE] test-arg-with-actions completed successfully');

process.exit(0);
