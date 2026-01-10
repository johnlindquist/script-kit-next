import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

// Show confirm dialog and capture screenshot

async function main() {
  console.error('[TEST] Showing confirm dialog...');

  // Use setTimeout to capture screenshot while dialog is visible
  setTimeout(async () => {
    try {
      console.error('[TEST] Capturing screenshot...');
      const shot = await captureScreenshot();
      const dir = join(process.cwd(), '.test-screenshots');
      mkdirSync(dir, { recursive: true });
      const path = join(dir, `confirm-${Date.now()}.png`);
      writeFileSync(path, Buffer.from(shot.data, 'base64'));
      console.error(`[TEST] Screenshot saved to ${path}`);
    } catch (e) {
      console.error('[TEST] Screenshot error:', e);
    }
  }, 500);

  // Show confirm dialog - user must interact to continue
  const result = await confirm({
    message: "Delete this file permanently?",
    confirmText: "Delete",
    cancelText: "Keep"
  });

  console.error(`[TEST] User chose: ${result ? 'Delete' : 'Keep'}`);
  process.exit(0);
}

main().catch((err) => {
  console.error('[TEST] Error:', err);
  process.exit(1);
});
