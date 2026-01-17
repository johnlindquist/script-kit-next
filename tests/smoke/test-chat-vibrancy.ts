import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test chat message vibrancy backgrounds
// The message containers should show system blur through white @ 8% opacity

// Don't await - we want to capture the UI while it's showing
const chatPromise = chat({
  placeholder: 'Ask follow-up...',
  messages: [
    { role: 'user', content: 'What is the capital of France?' },
    { role: 'assistant', content: 'The capital of France is **Paris**. It is the largest city in France and serves as the country\'s political, economic, and cultural center.\n\n## Key Facts\n- Population: ~2.2 million (city), ~12 million (metro)\n- Located on the River Seine\n- Known for the Eiffel Tower, Louvre Museum, and Notre-Dame' },
    { role: 'user', content: 'What about Germany?' },
    { role: 'assistant', content: 'The capital of Germany is **Berlin**. It is also the largest city in Germany.\n\n## Key Facts\n- Population: ~3.7 million\n- Historic significance as divided city (East/West)\n- Known for Brandenburg Gate and Berlin Wall remnants' },
  ],
});

// Wait for render
await new Promise(r => setTimeout(r, 1200));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `chat-vibrancy-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
