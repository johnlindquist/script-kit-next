// Name: Test Terminal Height
// Description: Tests that terminal fills the full 700px window height without cutoff

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-term-height.ts starting...');

// MAX_HEIGHT = 700px (from window_resize.rs)
// CELL_HEIGHT = 18.2px (14pt * 1.3 line height)
// Default padding = 8px top + 8px bottom = 16px
// Available height = 700 - 16 = 684px
// Expected rows = floor(684 / 18.2) = 37 rows

const EXPECTED_ROWS = 37;
const CELL_HEIGHT = 18.2;
const MAX_HEIGHT = 700;

console.error('[SMOKE] Expected configuration:');
console.error(`[SMOKE]   MAX_HEIGHT: ${MAX_HEIGHT}px`);
console.error(`[SMOKE]   CELL_HEIGHT: ${CELL_HEIGHT}px`);
console.error(`[SMOKE]   Padding: 8px top + 8px bottom = 16px`);
console.error(`[SMOKE]   Available: ${MAX_HEIGHT - 16}px`);
console.error(`[SMOKE]   Expected rows: ${EXPECTED_ROWS}`);

// Generate exactly 37 lines of output to verify all lines are visible
// If there's a padding bug, lines at the bottom will be cut off
const lines: string[] = [];
for (let i = 1; i <= EXPECTED_ROWS; i++) {
  lines.push(`echo "Line ${String(i).padStart(2, '0')} of ${EXPECTED_ROWS} - ${i === EXPECTED_ROWS ? 'LAST LINE (should be visible!)' : 'OK'}"`);
}

// Terminal should trigger a window resize to MAX_HEIGHT (700px)
// and the terminal content should fill the entire window without cutoff
await term(`echo "=== Terminal Height Regression Test ==="
echo ""
echo "Testing that terminal correctly accounts for BOTH top AND bottom padding."
echo "If you can see 'Line 37 of 37 - LAST LINE' below, the fix is working!"
echo ""
${lines.join('\n')}
echo ""
echo "=== TEST COMPLETE ==="
echo "If Line 37 is visible without scrolling, PASS!"
echo "If Line 37 is cut off or requires scrolling, FAIL (padding regression)!"
echo ""
echo "Press any key to exit..."
read -n 1 -s
`);

console.error('[SMOKE] test-term-height.ts completed!');
