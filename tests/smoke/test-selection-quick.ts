// Quick visual test for selection styling
import '../../scripts/kit-sdk';

// Wait a bit then take screenshot
setTimeout(async () => {
  const fs = await import('fs');
  const path = await import('path');
  
  const screenshot = await captureScreenshot();
  const dir = path.join(process.cwd(), 'test-screenshots');
  fs.mkdirSync(dir, { recursive: true });
  
  const filePath = path.join(dir, `selection-test-${Date.now()}.png`);
  fs.writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] Saved to: ${filePath}`);
  process.exit(0);
}, 1000);

// Show the arg prompt with choices
await arg({
  placeholder: "Test Selection Styling",
  choices: [
    { name: "First Item (Selected)", description: "This should have subtle gold tint" },
    { name: "Second Item", description: "This should be normal" },
    { name: "Third Item", description: "Another normal item" },
    { name: "Fourth Item", description: "Yet another item" },
  ]
});
