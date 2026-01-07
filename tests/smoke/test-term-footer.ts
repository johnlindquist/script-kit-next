// Test: Verify term prompt shows footer
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const testName = "term-footer-visibility";

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

log(testName, "running");
const start = Date.now();

// Set up exit handler - capture screenshot after terminal renders
setTimeout(async () => {
  try {
    // Capture screenshot
    const screenshot = await captureScreenshot();
    const dir = join(process.cwd(), 'test-screenshots');
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
    
    const path = join(dir, `term-footer-${Date.now()}.png`);
    writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${path}`);
    
    // Also get layout info
    const layout = await getLayoutInfo();
    console.error(`[LAYOUT] Window: ${layout.windowWidth}x${layout.windowHeight}, Type: ${layout.promptType}`);
    console.error(`[LAYOUT] Components: ${layout.components.map(c => `${c.name}(y=${c.bounds.y},h=${c.bounds.height})`).join(', ')}`);
    
    log(testName, "pass", { 
      duration_ms: Date.now() - start,
      screenshot: path,
      windowHeight: layout.windowHeight
    });
    
    process.exit(0);
  } catch (e) {
    log(testName, "fail", { error: String(e), duration_ms: Date.now() - start });
    process.exit(1);
  }
}, 2000);

// Show terminal prompt - term() takes a string command
term("echo 'Testing terminal footer visibility'");
