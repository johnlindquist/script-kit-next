// Name: Test App Icons Color Channels
// Description: Test that app icons have correct color channels (not BGRA/RGBA swapped)

import '../../scripts/kit-sdk';
import { saveScreenshot } from '../autonomous/screenshot-utils';

console.error('[SMOKE] Testing app icon color channels...');

// Capture screenshot BEFORE showing the prompt to test icon rendering
// Wait a moment for the app to fully initialize
await new Promise(resolve => setTimeout(resolve, 500));

// Capture screenshot for visual verification first
try {
    const screenshot = await captureScreenshot();
    console.error(`[SMOKE] Initial screenshot captured: ${screenshot.width}x${screenshot.height}`);
    
    // Save screenshot for visual inspection
    const savedPath = await saveScreenshot(screenshot.data, 'app-icons-initial');
    console.error(`[SMOKE] Screenshot saved to: ${savedPath}`);
} catch (e) {
    console.error(`[SMOKE] Initial screenshot failed: ${e}`);
}

// Use a short timeout to show the prompt briefly then exit
// This avoids blocking on user input
const timeout = 2000; // 2 seconds to view

// Start a timer to exit
setTimeout(() => {
    console.error('[SMOKE] Test timeout - exiting');
    process.exit(0);
}, timeout);

// Show the prompt - this will trigger icon extraction
// The icons should appear with correct colors (not inverted R/B)
console.error('[SMOKE] Showing arg prompt with app choices...');
const result = await arg("Test app icons - check colors (auto-exits in 2s)", [
    {
        name: "Calculator",
        value: "calculator", 
        description: "Calculator icon should have correct colors (orange/white)",
    },
    {
        name: "Safari",
        value: "safari",
        description: "Safari icon should have blue/red compass (not yellow/cyan)",
    },
    {
        name: "Finder",
        value: "finder",
        description: "Finder icon should have blue face (not orange)",
    },
    {
        name: "Done - exit test",
        value: "done",
        description: "Select to exit",
    }
]);

// Log the result
console.error(`[SMOKE] Selected: ${result}`);
console.error('[SMOKE] App icon color test complete');
