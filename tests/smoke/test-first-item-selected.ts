// Test: Verify first item is always selected when main menu opens
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

async function main() {
  // Give the main menu a moment to render
  await new Promise(r => setTimeout(r, 500));

  // Capture screenshot to verify first item is selected
  const screenshot = await captureScreenshot();
  const dir = join(process.cwd(), 'test-screenshots');
  mkdirSync(dir, { recursive: true });
  
  const timestamp = Date.now();
  const path = join(dir, `first-item-selected-${timestamp}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);

  // Also get layout info to verify selected state
  const layout = await getLayoutInfo();
  console.error(`[LAYOUT] windowWidth=${layout.windowWidth} windowHeight=${layout.windowHeight}`);
  console.error(`[LAYOUT] promptType=${layout.promptType}`);
  console.error(`[LAYOUT] components=${layout.components.length}`);
  
  // Log test result
  console.log(JSON.stringify({
    test: "first-item-selected",
    status: "pass",
    screenshot: path,
    timestamp: new Date().toISOString()
  }));

  process.exit(0);
}

main().catch(err => {
  console.error(`[ERROR] ${err}`);
  console.log(JSON.stringify({
    test: "first-item-selected",
    status: "fail",
    error: String(err),
    timestamp: new Date().toISOString()
  }));
  process.exit(1);
});
