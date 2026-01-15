// Test ShortcutRecorder modal (inline overlay on main window)
// This tests the actual modal rendered by the app
import '../../scripts/kit-sdk';

async function main() {
  console.error('[TEST] Starting ShortcutRecorder modal test');

  // First show the main window
  // Then we'll send stdin commands to trigger the modal
  
  // Schedule screenshot capture after modal renders
  setTimeout(async () => {
    try {
      console.error('[TEST] Capturing screenshot of ShortcutRecorder modal');
      const screenshot = await captureScreenshot();
      const { writeFileSync, mkdirSync } = await import('fs');
      const { join } = await import('path');
      
      const dir = join(process.cwd(), 'test-screenshots');
      mkdirSync(dir, { recursive: true });
      
      const path = join(dir, `shortcut-recorder-modal-${Date.now()}.png`);
      writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
      console.error(`[SCREENSHOT] ${path}`);
      process.exit(0);
    } catch (e) {
      console.error(`[ERROR] ${e}`);
      process.exit(1);
    }
  }, 1500);

  // Show a simple prompt first, then we'll trigger the modal via stdin
  div(`
    <div style="padding: 24px; text-align: center;">
      <h2 style="color: white;">Waiting for ShortcutRecorder modal...</h2>
      <p style="color: #888;">The modal should appear as an overlay</p>
    </div>
  `);
}

main();
