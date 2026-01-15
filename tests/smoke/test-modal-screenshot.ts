// Capture screenshot of current window state
// Run this AFTER triggering a modal via stdin
import '../../scripts/kit-sdk';

async function main() {
  // Small delay to ensure modal is fully rendered
  await new Promise(r => setTimeout(r, 300));
  
  try {
    const screenshot = await captureScreenshot();
    const { writeFileSync, mkdirSync } = await import('fs');
    const { join } = await import('path');
    
    const dir = join(process.cwd(), 'test-screenshots');
    mkdirSync(dir, { recursive: true });
    
    // Get name from command line or use default
    const name = process.argv[2] || 'modal';
    const path = join(dir, `${name}-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${path}`);
  } catch (e) {
    console.error(`[ERROR] ${e}`);
  }
  
  process.exit(0);
}

main();
