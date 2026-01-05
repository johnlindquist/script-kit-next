// Test vibrancy with a simple semi-transparent div
// @ts-nocheck
import '../../scripts/kit-sdk';

// Show a simple div with semi-transparent background
// If vibrancy is working, the desktop should be visible (blurred) behind the div
await div(`
  <div style="
    width: 100%;
    height: 100%;
    background: rgba(30, 30, 30, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    color: white;
  ">
    If vibrancy works, you should see the desktop (blurred) behind this text
  </div>
`);
