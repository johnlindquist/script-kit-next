// Name: SDK Actions Test
// Description: Tests setActions() with onAction handlers and shortcuts

/**
 * SMOKE TEST: test-sdk-actions.ts
 *
 * Tests the SDK Actions feature:
 * 1. setActions() sends SetActions message to Rust
 * 2. Actions with onAction handlers (has_action=true) trigger ActionTriggered
 * 3. Actions without handlers (has_action=false) submit value directly
 * 4. Keyboard shortcuts are registered and work
 *
 * Expected behavior:
 * - Rust receives SetActions with actions array
 * - Shortcut "cmd+t" triggers "Test Action" which has onAction
 * - Rust sends ActionTriggered message back to SDK
 * - SDK calls the onAction handler
 */

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-sdk-actions.ts starting...');

// Track which actions were triggered
let testActionTriggered = false;
let submitActionTriggered = false;

// Set up actions with different configurations
await setActions([
  {
    name: 'Test Action',
    description: 'Action with onAction handler (has_action=true)',
    shortcut: 'cmd+t',
    onAction: async (input, state) => {
      console.error('[SMOKE] Test Action triggered!');
      console.error('[SMOKE] Input:', input);
      console.error('[SMOKE] State:', JSON.stringify(state));
      testActionTriggered = true;
    },
  },
  {
    name: 'Submit Action',
    description: 'Action without handler (has_action=false)',
    shortcut: 'cmd+s',
    value: 'submitted-value',
  },
  {
    name: 'No Shortcut',
    description: 'Action without shortcut',
    onAction: async () => {
      console.error('[SMOKE] No Shortcut action triggered!');
    },
  },
]);

console.error('[SMOKE] Actions set. Showing prompt...');

// Show a div with instructions
await div(
  md(`# SDK Actions Test

## Actions Registered:
1. **Test Action** (Cmd+T) - Has onAction handler
2. **Submit Action** (Cmd+S) - No handler, submits value
3. **No Shortcut** - Only accessible via Cmd+K menu

## Test Instructions:
- Press **Cmd+T** to trigger Test Action (should log to stderr)
- Press **Cmd+K** to open Actions dialog
- Press **Escape** to exit

---

*Check stderr for "[SMOKE] Test Action triggered!" messages*`)
);

console.error('[SMOKE] test-sdk-actions.ts completed');
console.error('[SMOKE] Test Action was triggered:', testActionTriggered);
