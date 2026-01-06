// Test multiple HUDs and auto-dismiss behavior
import '../../scripts/kit-sdk';

console.error('[HUD_TEST] Starting multiple HUD test');

// Show first HUD (2s default duration)
hud('HUD 1 - default duration');
console.error('[HUD_TEST] HUD 1 shown');

// Wait 500ms then show second HUD
await new Promise(r => setTimeout(r, 500));
hud('HUD 2 - 3 seconds', { duration: 3000 });
console.error('[HUD_TEST] HUD 2 shown');

// Wait 500ms then show third HUD
await new Promise(r => setTimeout(r, 500));
hud('HUD 3 - 1 second', { duration: 1000 });
console.error('[HUD_TEST] HUD 3 shown');

// Wait for all HUDs to dismiss (4 seconds should be enough)
console.error('[HUD_TEST] Waiting for all HUDs to dismiss...');
await new Promise(r => setTimeout(r, 4000));

console.error('[HUD_TEST] Test complete - all HUDs should have auto-dismissed');
