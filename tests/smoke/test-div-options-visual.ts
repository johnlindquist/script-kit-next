// Name: Visual Test - div() Container Options
// Description: Captures screenshots of different container options

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOT_DIR = join(process.cwd(), '.test-screenshots');

async function captureAndSave(name: string): Promise<string> {
  await new Promise(resolve => setTimeout(resolve, 600));
  const screenshot = await captureScreenshot();
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const filename = `div-options-${name}.png`;
  const filepath = join(SCREENSHOT_DIR, filename);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  return filepath;
}

async function runTests() {
  console.error('[TEST] Starting div container options visual tests...');

  // Test 1: Transparent background with gradient content
  console.error('[TEST] 1. Transparent container + gradient content');
  
  // Use setTimeout to auto-submit after screenshot
  setTimeout(async () => {
    await captureAndSave('transparent-gradient');
    // Force submit to move to next test
    submit();
  }, 800);
  
  await div(`
    <div class="bg-gradient-to-br from-purple-600 via-pink-500 to-orange-400 p-8 rounded-xl text-white h-full w-full flex flex-col justify-center items-center">
      <h1 class="text-3xl font-bold mb-4">Transparent Container</h1>
      <p class="text-lg">The gradient extends edge-to-edge because containerBg is transparent and padding is none.</p>
    </div>
  `, { containerBg: 'transparent', containerPadding: 'none' });

  // Test 2: Custom hex with alpha
  console.error('[TEST] 2. Semi-transparent blue (#0066ff80)');
  
  setTimeout(async () => {
    await captureAndSave('semitransparent-blue');
    submit();
  }, 800);
  
  await div(`
    <div class="text-white p-6">
      <h1 class="text-2xl font-bold mb-2">Semi-Transparent Blue</h1>
      <p>Container background: #0066ff80 (50% blue)</p>
      <p class="mt-4 text-sm opacity-75">The window content behind should be partially visible.</p>
    </div>
  `, { containerBg: '#0066ff80' });

  // Test 3: Tailwind color
  console.error('[TEST] 3. Tailwind color (rose-500)');
  
  setTimeout(async () => {
    await captureAndSave('tailwind-rose');
    submit();
  }, 800);
  
  await div(`
    <div class="text-white p-6">
      <h1 class="text-2xl font-bold mb-2">Tailwind Color: rose-500</h1>
      <p>Container uses Tailwind's rose-500 color.</p>
    </div>
  `, { containerBg: 'rose-500' });

  // Test 4: No padding
  console.error('[TEST] 4. No container padding');
  
  setTimeout(async () => {
    await captureAndSave('no-padding');
    submit();
  }, 800);
  
  await div(`
    <div class="bg-emerald-500 text-white p-6 h-full w-full">
      <h1 class="text-2xl font-bold mb-2">No Container Padding</h1>
      <p>This green box should touch the window edges.</p>
    </div>
  `, { containerPadding: 'none' });

  // Test 5: Custom padding (large)
  console.error('[TEST] 5. Custom padding (64px)');
  
  setTimeout(async () => {
    await captureAndSave('custom-padding');
    submit();
  }, 800);
  
  await div(`
    <div class="bg-indigo-500 text-white p-4 rounded-lg">
      <h1 class="text-2xl font-bold mb-2">Large Container Padding</h1>
      <p>There should be 64px of theme background around this box.</p>
    </div>
  `, { containerPadding: 64 });

  // Test 6: Opacity on theme background
  console.error('[TEST] 6. Container opacity (30%)');
  
  setTimeout(async () => {
    await captureAndSave('opacity-30');
    submit();
  }, 800);
  
  await div(`
    <div class="text-white p-6">
      <h1 class="text-2xl font-bold mb-2">30% Opacity Container</h1>
      <p>The theme background should be very transparent (30%).</p>
      <p class="mt-4 text-sm">Desktop/content behind should be clearly visible.</p>
    </div>
  `, { opacity: 30 });

  console.error('[TEST] All visual tests captured!');
  process.exit(0);
}

runTests().catch(err => {
  console.error('[TEST] Error:', err);
  process.exit(1);
});
