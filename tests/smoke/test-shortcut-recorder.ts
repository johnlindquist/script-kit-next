// Name: Test Shortcut Recorder
// Description: Smoke test for the inline keyboard shortcut recorder modal

import '../../scripts/kit-sdk';

// @ts-ignore - Node.js modules available at runtime
const fs = require('fs');

// @ts-ignore - process available at runtime
const dir = `${process.cwd()}/test-screenshots`;
fs.mkdirSync(dir, { recursive: true });

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
  screenshot?: string;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

// Helper to save screenshot
async function saveScreenshot(name: string): Promise<string | null> {
  try {
    const screenshot = await captureScreenshot();
    const filepath = `${dir}/${name}-${Date.now()}.png`;
    // @ts-ignore - Buffer available at runtime
    fs.writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${name}: ${filepath}`);
    console.error(`[SCREENSHOT] Dimensions: ${screenshot.width}x${screenshot.height}`);
    return filepath;
  } catch (e) {
    console.error(`[SCREENSHOT] Error capturing ${name}:`, e);
    return null;
  }
}

async function runTests() {
  const testName = 'shortcut-recorder-smoke';
  logTest(testName, 'running');
  const start = Date.now();

  console.error('[SMOKE] test-shortcut-recorder starting...');
  console.error('[SMOKE] This test verifies the inline shortcut recorder UI');

  try {
    // Create test choices that would trigger the shortcut recorder
    // Using scriptlet-like names since scriptlets trigger the recorder
    const choices = [
      { 
        name: "Test Scriptlet Alpha", 
        description: "A test scriptlet for shortcut testing",
        value: "alpha"
      },
      { 
        name: "Test Scriptlet Beta", 
        description: "Another test scriptlet",
        value: "beta"
      },
      { 
        name: "Test Scriptlet Gamma", 
        description: "Third test scriptlet",
        value: "gamma"
      },
    ];

    // Define actions including Configure Shortcut
    // The configure_shortcut action triggers the inline recorder for non-scripts
    const actions = [
      { 
        name: "Run", 
        shortcut: "enter",
        description: "Execute the selected item"
      },
      { 
        name: "Assign Shortcut", 
        shortcut: "cmd+shift+k",
        description: "Open the shortcut recorder to assign a keyboard shortcut",
        // Note: The actual configure_shortcut action is handled by the app
      },
      { 
        name: "Copy Name", 
        shortcut: "cmd+c",
        description: "Copy the item name to clipboard"
      },
      { 
        name: "View Details", 
        shortcut: "cmd+i",
        description: "Show item details"
      },
    ];

    console.error('[SMOKE] Setting up arg prompt with choices and actions...');
    console.error('[SMOKE] Actions panel can be opened with Cmd+K');
    console.error('[SMOKE] To test the shortcut recorder:');
    console.error('[SMOKE]   1. Press Cmd+K to open actions');
    console.error('[SMOKE]   2. Select "Assign Shortcut" or press Cmd+Shift+K');
    console.error('[SMOKE]   3. The shortcut recorder modal should appear');

    // Capture initial state before interaction
    setTimeout(async () => {
      const initialPath = await saveScreenshot('shortcut-recorder-initial');
      console.error('[SMOKE] Initial state captured');
      console.error('[SMOKE] Expected: Main list with choices visible');
      if (initialPath) {
        console.error(`[SMOKE] Screenshot at: ${initialPath}`);
      }
    }, 500);

    // Capture after a delay (allows time for manual Cmd+K)
    setTimeout(async () => {
      const actionsPath = await saveScreenshot('shortcut-recorder-actions-panel');
      console.error('[SMOKE] Actions panel screenshot captured');
      console.error('[SMOKE] If Cmd+K was pressed, actions panel should be visible');
      if (actionsPath) {
        console.error(`[SMOKE] Screenshot at: ${actionsPath}`);
      }
    }, 2000);

    // Capture after more delay (allows time for shortcut recorder to appear)
    setTimeout(async () => {
      const recorderPath = await saveScreenshot('shortcut-recorder-modal');
      console.error('[SMOKE] Recorder modal screenshot captured');
      console.error('[SMOKE] If Configure Shortcut was triggered, recorder should be visible');
      console.error('[SMOKE] Expected UI elements:');
      console.error('[SMOKE]   - Modal overlay with command name');
      console.error('[SMOKE]   - Compact "Press keys" prompt');
      console.error('[SMOKE]   - Clear, Cancel, Save buttons');
      console.error('[SMOKE]   - Keycaps showing captured modifiers/keys');
      if (recorderPath) {
        console.error(`[SMOKE] Screenshot at: ${recorderPath}`);
      }
    }, 4000);

    // Start the arg prompt
    const result = await arg({
      placeholder: "Select item (Cmd+K for actions, or Cmd+Shift+K to assign shortcut)",
      choices,
      actions,
    });

    console.error(`[SMOKE] User selected: ${result}`);
    
    logTest(testName, 'pass', {
      result,
      duration_ms: Date.now() - start,
    });

  } catch (err) {
    console.error('[SMOKE] Test error:', err);
    logTest(testName, 'fail', {
      error: String(err),
      duration_ms: Date.now() - start,
    });
  }

  console.error('[SMOKE] test-shortcut-recorder completed');
  // @ts-ignore - process available at runtime
  process.exit(0);
}

runTests();
