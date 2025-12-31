// Name: Test div() Container Options
// Description: Tests containerBg, containerPadding, and opacity options

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOT_DIR = join(process.cwd(), '.test-screenshots');

async function captureAndSave(name: string): Promise<string> {
  await new Promise(resolve => setTimeout(resolve, 500));
  const screenshot = await captureScreenshot();
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const filename = `div-container-${name}-${Date.now()}.png`;
  const filepath = join(SCREENSHOT_DIR, filename);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  return filepath;
}

async function runTests() {
  console.error('[TEST] Starting div container options tests...');

  // Test 1: Default (no options) - should have theme background and padding
  console.error('[TEST] 1. Default container (theme bg + padding)');
  await div(`
    <div class="text-white text-2xl">
      <h1>Default Container</h1>
      <p>This should have theme background and default padding.</p>
    </div>
  `);
  // User sees this and presses Enter/Escape to continue

  // Test 2: Transparent background
  console.error('[TEST] 2. Transparent container background');
  await div(`
    <div class="bg-gradient-to-r from-purple-500 to-pink-500 p-8 rounded-lg text-white">
      <h1 class="text-2xl font-bold">Transparent Container</h1>
      <p>The container is transparent - this gradient is the only background.</p>
    </div>
  `, { containerBg: 'transparent', containerPadding: 'none' });

  // Test 3: Custom hex color
  console.error('[TEST] 3. Custom hex background (#ff6b6b)');
  await div(`
    <div class="text-white text-2xl p-4">
      <h1>Custom Hex Color</h1>
      <p>Container should be coral/salmon colored (#ff6b6b)</p>
    </div>
  `, { containerBg: '#ff6b6b' });

  // Test 4: Hex with alpha (semi-transparent)
  console.error('[TEST] 4. Semi-transparent hex (#0000ff80 = 50% blue)');
  await div(`
    <div class="text-white text-2xl p-4">
      <h1>Semi-Transparent Blue</h1>
      <p>Container should be 50% transparent blue (#0000ff80)</p>
    </div>
  `, { containerBg: '#0000ff80' });

  // Test 5: Tailwind color name
  console.error('[TEST] 5. Tailwind color name (emerald-600)');
  await div(`
    <div class="text-white text-2xl p-4">
      <h1>Tailwind Color</h1>
      <p>Container should be emerald-600</p>
    </div>
  `, { containerBg: 'emerald-600' });

  // Test 6: Custom padding
  console.error('[TEST] 6. Custom padding (48px)');
  await div(`
    <div class="bg-blue-500 text-white text-2xl p-4 rounded">
      <h1>Custom Padding</h1>
      <p>Container should have 48px padding around this blue box.</p>
    </div>
  `, { containerPadding: 48 });

  // Test 7: No padding
  console.error('[TEST] 7. No padding (containerPadding: "none")');
  await div(`
    <div class="bg-green-500 text-white text-2xl p-4 h-full w-full">
      <h1>No Padding</h1>
      <p>This green box should extend to the window edges.</p>
    </div>
  `, { containerPadding: 'none' });

  // Test 8: Opacity
  console.error('[TEST] 8. Container opacity (50%)');
  await div(`
    <div class="text-white text-2xl p-4">
      <h1>50% Opacity</h1>
      <p>The entire container (including theme bg) should be 50% transparent.</p>
    </div>
  `, { opacity: 50 });

  // Test 9: Combination - transparent bg with tailwind classes
  console.error('[TEST] 9. Transparent bg + tailwind root classes');
  await div(`
    <div class="text-white p-4">
      <h1 class="text-2xl font-bold">Combined</h1>
      <p>Root has flex/center classes from tailwind param.</p>
      <p>Container is transparent.</p>
    </div>
  `, 'flex items-center justify-center bg-indigo-600 rounded-xl', { containerBg: 'transparent', containerPadding: 'none' });

  console.error('[TEST] All div container option tests completed!');
  process.exit(0);
}

runTests().catch(err => {
  console.error('[TEST] Error:', err);
  process.exit(1);
});
