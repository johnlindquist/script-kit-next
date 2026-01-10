import '../../scripts/kit-sdk';

/**
 * Test: Script Exit Shows Window
 *
 * This test verifies that when a script hides the window (e.g., for getSelectedText)
 * and then exits, the main menu comes back.
 *
 * Expected behavior:
 * 1. Script starts
 * 2. Script sends Hide message (simulating getSelectedText behavior)
 * 3. Script shows HUD
 * 4. Script exits
 * 5. Main window should be shown again (NOT stay hidden)
 */

const test = 'script-exit-shows-window';

function log(status: string, extra: any = {}) {
  console.error(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

log('running');
const start = Date.now();

// Step 1: Hide the window (simulating what getSelectedText does)
// This should set SCRIPT_REQUESTED_HIDE = true
log('hiding_window');
await hide();

// Give time for the hide to process
await new Promise(r => setTimeout(r, 100));

// Step 2: Show a HUD (simulating what happens after getSelectedText fails)
log('showing_hud');
await hud('Test HUD - window should come back after exit');

// Step 3: The exit() call will send Exit message
// The ScriptExit handler should see SCRIPT_REQUESTED_HIDE = true
// and request showing the main window
log('exiting');
log('pass', {
  result: 'Script executed hide -> hud -> exit sequence',
  duration_ms: Date.now() - start,
  expected: 'Main window should reappear after this script exits'
});

// Exit the script - this should trigger window show
exit(0);
