// Name: Test Resize Behavior
// Description: Validates window resize across different prompts

import '../../scripts/kit-sdk';

console.error("[TEST] Starting resize behavior test...");

// Layout constants for reference:
// HEADER_HEIGHT = 100px
// LIST_ITEM_HEIGHT = 52px
// FOOTER_HEIGHT = 44px
// LIST_PADDING = 8px
// MIN_HEIGHT = 120px
// MAX_HEIGHT = 700px
// MAX_VISIBLE_ITEMS = 10

// Test 1: arg with many choices (15) - should show 10 visible, height ~672px
console.error("[TEST] Test 1: arg with 15 choices");
const colors = await arg("Pick a color (15 choices)", [
  "Red", "Blue", "Green", "Yellow", "Purple",
  "Orange", "Pink", "Brown", "Black", "White",
  "Cyan", "Magenta", "Lime", "Navy", "Teal"
]);
console.error(`[TEST] Selected: ${colors}`);

const bounds1 = await getWindowBounds();
// Expected: 100 + (10 * 52) + 44 + 8 = 672px (capped at 10 visible)
console.error(`[TEST] After arg(15): height=${bounds1.height} (expected ~672)`);

// Test 2: arg with few choices (3) - should shrink to fit
console.error("[TEST] Test 2: arg with 3 choices");
const size = await arg("Pick a size (3 choices)", ["Small", "Medium", "Large"]);
console.error(`[TEST] Selected: ${size}`);

const bounds2 = await getWindowBounds();
// Expected: 100 + (3 * 52) + 44 + 8 = 308px
console.error(`[TEST] After arg(3): height=${bounds2.height} (expected ~308)`);

// Test 3: editor - should be MAX_HEIGHT (700)
console.error("[TEST] Test 3: editor prompt");
const code = await editor("// Edit me\nconsole.log('hello')");
console.error(`[TEST] Edited: ${code.substring(0, 30)}...`);

const bounds3 = await getWindowBounds();
console.error(`[TEST] After editor: height=${bounds3.height} (expected 700)`);

// Test 4: div - should be MAX_HEIGHT (700)
console.error("[TEST] Test 4: div prompt");
await div(md(`# Resize Test Complete

Window heights observed:
- After arg(15 choices): **${bounds1.height}px** (expected ~672)
- After arg(3 choices): **${bounds2.height}px** (expected ~308)
- After editor: **${bounds3.height}px** (expected 700)

Expected layout constants:
- MIN_HEIGHT = 120px (input only)
- MAX_HEIGHT = 700px (editor/div/term)
- Formula: header(100) + items*52 + footer(44) + padding(8)

Press Escape to exit.`));

console.error("[TEST] Resize behavior test complete!");
