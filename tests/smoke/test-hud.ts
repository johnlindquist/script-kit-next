// Name: Test HUD
// Description: Tests the HUD overlay message functionality

import '../../scripts/kit-sdk';

console.error('[SMOKE] Testing HUD display...');

// Show HUD message
await hud("Hello from HUD!");

// Wait a bit for visual verification
await new Promise(r => setTimeout(r, 1000));

console.error('[SMOKE] HUD test complete');
process.exit(0);
