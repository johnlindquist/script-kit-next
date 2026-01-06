// Test HUD auto-dismiss behavior
// HUDs should auto-disappear after their timeout regardless of main window visibility
import '../../scripts/kit-sdk';

console.error('[HUD_TEST] Starting HUD auto-dismiss test');

// Show a HUD message (should auto-dismiss after 2 seconds by default)
hud('Test HUD - should disappear in 2s');

console.error('[HUD_TEST] HUD shown, waiting 3 seconds to verify dismissal...');

// Wait 3 seconds (longer than default 2s HUD duration)
await new Promise(r => setTimeout(r, 3000));

console.error('[HUD_TEST] Test complete - HUD should have auto-dismissed');
