import '../../scripts/kit-sdk';

const fs = require('fs');
const dir = `${process.cwd()}/.test-screenshots/grid-audit`;
fs.mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing ARG prompt with grid overlay');

// Set up screenshot capture BEFORE await arg - fires while prompt is displayed
setTimeout(async () => {
  const ss = await captureScreenshot();
  fs.writeFileSync(`${dir}/01-arg-choices.png`, Buffer.from(ss.data, 'base64'));
  console.error(`[AUDIT] Screenshot saved: ${dir}/01-arg-choices.png`);
  process.exit(0);
}, 1000);

// Show arg prompt with choices (blocks, but setTimeout fires while waiting)
await arg({
  placeholder: 'Select a fruit',
  choices: [
    { name: 'Apple', description: 'A red fruit', value: 'apple' },
    { name: 'Banana', description: 'A yellow fruit', value: 'banana' },
    { name: 'Cherry', description: 'A small red fruit', value: 'cherry' },
    { name: 'Date', description: 'A sweet fruit', value: 'date' },
    { name: 'Elderberry', description: 'A purple berry', value: 'elderberry' },
  ]
});
