// Visual test for file search directory navigation
// Tests that typing ~/dev/ shows directory contents
import '../../scripts/kit-sdk';

async function runTest() {
  const fs = await import('fs');
  const path = await import('path');
  
  console.error('[TEST] Starting file search directory navigation visual test');
  
  // Wait for app to initialize
  await new Promise(r => setTimeout(r, 500));
  
  // Simulate typing a directory path
  setInput('~/dev/');
  
  // Wait for debounce (200ms) + processing
  await new Promise(r => setTimeout(r, 800));
  
  // Capture screenshot
  const screenshot = await captureScreenshot();
  const dir = path.join(process.cwd(), 'test-screenshots');
  fs.mkdirSync(dir, { recursive: true });
  const screenshotPath = path.join(dir, `file-search-dir-nav-${Date.now()}.png`);
  fs.writeFileSync(screenshotPath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${screenshotPath}`);
  
  // Get layout info to check component count
  const layout = await getLayoutInfo();
  console.error(`[TEST] Layout: ${layout.components.length} components, window=${layout.windowWidth}x${layout.windowHeight}`);
  
  // Check if we have list items (directory contents)
  const listItems = layout.components.filter(c => c.type === 'listItem');
  console.error(`[TEST] Found ${listItems.length} list items`);
  
  if (listItems.length > 0) {
    console.error('[TEST] PASS - Directory contents are showing');
  } else {
    console.error('[TEST] FAIL - No directory contents shown');
  }
  
  process.exit(0);
}

runTest();
