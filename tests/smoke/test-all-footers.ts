// Test: Verify all prompts show footer consistently
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const testName = "all-footers-visibility";

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

async function capturePrompt(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 500));
  
  const screenshot = await captureScreenshot();
  const dir = join(process.cwd(), 'test-screenshots', 'footers');
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  
  const path = join(dir, `${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${path}`);
  return path;
}

log(testName, "running");
const start = Date.now();

try {
  // Test 1: Editor prompt
  console.error('[TEST] Testing editor prompt footer...');
  editor({
    value: "// Test editor content\nconst x = 1;",
    description: "Editor Footer Test"
  });
  await new Promise(r => setTimeout(r, 1000));
  const editorPath = await capturePrompt('editor-footer');
  
  // Test 2: Div prompt  
  console.error('[TEST] Testing div prompt footer...');
  await div(`
    <div class="p-8 bg-gray-800 text-white">
      <h1 class="text-2xl mb-4">Div Footer Test</h1>
      <p>This div should have a footer at the bottom.</p>
    </div>
  `);
  await new Promise(r => setTimeout(r, 800));
  const divPath = await capturePrompt('div-footer');
  
  // Test 3: Arg prompt
  console.error('[TEST] Testing arg prompt footer...');
  arg({
    placeholder: "Arg Footer Test",
    choices: ["Option A", "Option B", "Option C"]
  });
  await new Promise(r => setTimeout(r, 800));
  const argPath = await capturePrompt('arg-footer');
  
  log(testName, "pass", { 
    duration_ms: Date.now() - start,
    screenshots: { editor: editorPath, div: divPath, arg: argPath }
  });
  
  process.exit(0);
} catch (e) {
  log(testName, "fail", { error: String(e), duration_ms: Date.now() - start });
  process.exit(1);
}
