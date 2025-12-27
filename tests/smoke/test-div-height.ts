// Name: Test Div Height
// Description: Tests that div uses standard 500px window height (matches main window)

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-div-height.ts starting...');

// Div should use STANDARD_HEIGHT (500px) to match the main window
// This is different from editor/terminal which use MAX_HEIGHT (700px)
await div(`
<h1>Div Height Test</h1>
<p>This div should use the <strong>standard 500px</strong> window height.</p>
<p>The window should NOT resize when div opens (stays at 500px).</p>
<hr>
<p>This matches the main script list window height for visual consistency.</p>
<p>Line 6</p>
<p>Line 7</p>
<p>Line 8</p>
<p>Line 9</p>
<p>Line 10</p>
<p>Line 11</p>
<p>Line 12</p>
<p>Line 13</p>
<p>Line 14</p>
<p>Line 15</p>
<hr>
<p><em>Press Enter or Escape to continue</em></p>
`);

console.error('[SMOKE] test-div-height.ts completed!');
