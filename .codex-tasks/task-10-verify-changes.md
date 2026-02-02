# Task: Create Visual Test for Button Behaviors

Create a test script that verifies button styling and behaviors:

1. Create a test file at `tests/smoke/test-button-consistency.ts`
2. The test should render various button states:
   - Normal button
   - Hovered button (simulated)
   - Disabled button
   - Icon button
3. Capture a screenshot to verify visual appearance
4. Log any issues found

Template:
```typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Render a div with multiple button examples
await div(`
  <div class="p-4 flex flex-col gap-4 bg-white">
    <button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer">Normal Button</button>
    <button class="px-4 py-2 bg-gray-300 text-gray-500 rounded cursor-not-allowed" disabled>Disabled Button</button>
    <button class="p-2 rounded hover:bg-gray-200 cursor-pointer">🔧</button>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'button-consistency.png'), Buffer.from(screenshot.data, 'base64'));
console.error('[TEST] Button screenshot saved');
process.exit(0);
```

Run the test and verify the output.
