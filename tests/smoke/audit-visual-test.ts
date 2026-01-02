// Name: Audit Visual Test
// Description: Captures screenshots of audit test scripts to verify rendering

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

interface TestResult {
  test: string;
  status: 'pass' | 'fail' | 'error';
  screenshot?: string;
  error?: string;
  duration_ms: number;
}

const results: TestResult[] = [];

async function captureTest(name: string, setup: () => Promise<void>) {
  const start = Date.now();
  console.error(`[AUDIT] Testing: ${name}`);
  
  try {
    await setup();
    await new Promise(r => setTimeout(r, 500)); // Wait for render
    
    const screenshot = await captureScreenshot();
    const filename = `audit-${name}-${Date.now()}.png`;
    const filepath = join(screenshotDir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    
    console.error(`[SCREENSHOT] ${filepath}`);
    results.push({
      test: name,
      status: 'pass',
      screenshot: filepath,
      duration_ms: Date.now() - start
    });
  } catch (err) {
    console.error(`[ERROR] ${name}: ${err}`);
    results.push({
      test: name,
      status: 'error',
      error: String(err),
      duration_ms: Date.now() - start
    });
  }
}

// Test 1: arg() with choices
await captureTest('arg-choices', async () => {
  // Don't await - just show the UI
  arg('Select a fruit', ['Apple', 'Banana', 'Cherry', 'Date', 'Elderberry']);
  await new Promise(r => setTimeout(r, 300));
});

console.error('[AUDIT] Results:', JSON.stringify(results, null, 2));
console.error('[AUDIT] Complete');
process.exit(0);
