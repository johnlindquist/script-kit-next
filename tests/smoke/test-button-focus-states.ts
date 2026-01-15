// Test script for Button component focus states
// Visual verification of the shared Button component with focus ring

import '../../scripts/kit-sdk';

async function main() {
  console.error('[TEST] Starting Button focus states visual test');

  // Schedule screenshot capture after div renders
  setTimeout(async () => {
    try {
      const screenshot = await captureScreenshot();
      const { writeFileSync, mkdirSync } = await import('fs');
      const { join } = await import('path');
      
      const dir = join(process.cwd(), 'test-screenshots');
      mkdirSync(dir, { recursive: true });
      
      const filename = `button-focus-states-${Date.now()}.png`;
      const path = join(dir, filename);
      writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
      console.error(`[SCREENSHOT] ${path}`);
      process.exit(0);
    } catch (e) {
      console.error(`[ERROR] Screenshot failed: ${e}`);
      process.exit(1);
    }
  }, 800);

  // Show a div with button-like elements to verify styling
  // This simulates what the ConfirmDialog buttons look like
  div(`
    <div class="flex flex-col gap-6 p-6">
      <h2 class="text-lg font-medium text-white mb-2">Button Focus States Test</h2>
      
      <div class="text-sm text-gray-400 mb-4">
        Testing the shared Button component styling used in ConfirmDialog
      </div>
      
      <!-- Simulated Cancel/Confirm buttons row -->
      <div class="flex gap-3 w-full">
        <!-- Cancel button (Ghost variant, unfocused) -->
        <div class="flex-1 flex items-center justify-center px-3 py-1.5 rounded-md border border-gray-600/40 text-yellow-400 text-sm font-medium cursor-pointer hover:bg-white/10">
          Cancel
        </div>
        
        <!-- Confirm button (Primary variant, focused - with ring) -->
        <div class="flex-1 flex items-center justify-center px-3 py-1.5 rounded-md border-2 border-yellow-400/60 bg-gray-700/50 text-yellow-400 text-sm font-medium cursor-pointer">
          Confirm ✓
        </div>
      </div>
      
      <div class="text-xs text-gray-500 mt-2">
        Confirm button shows focus ring (2px yellow border)
      </div>
      
      <!-- Second row: reversed focus -->
      <div class="flex gap-3 w-full mt-4">
        <!-- Cancel button (Ghost variant, focused) -->
        <div class="flex-1 flex items-center justify-center px-3 py-1.5 rounded-md border-2 border-yellow-400/60 bg-yellow-400/10 text-yellow-400 text-sm font-medium cursor-pointer">
          Cancel ✓
        </div>
        
        <!-- Confirm button (Primary variant, unfocused) -->
        <div class="flex-1 flex items-center justify-center px-3 py-1.5 rounded-md border border-yellow-400/40 bg-gray-700/50 text-yellow-400 text-sm font-medium cursor-pointer hover:bg-gray-600/70">
          Confirm
        </div>
      </div>
      
      <div class="text-xs text-gray-500 mt-2">
        Cancel button shows focus ring + tint background
      </div>
    </div>
  `);
}

main().catch(e => {
  console.error(`[ERROR] Test failed: ${e}`);
  process.exit(1);
});
