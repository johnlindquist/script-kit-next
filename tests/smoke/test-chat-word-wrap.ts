// Test chat prompt word wrapping
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Create a chat with long text that should wrap
const longText = "The poem continues for many more stanzas, exploring themes of existence, journey, discovery, and the infinite nature of the universe and human consciousness through beautifully crafted verses.";

await chat({
  messages: [
    { role: "user", text: "Tell me a long story" },
    { role: "assistant", text: `Centuries passed like moments in my stride,
Millennia compressed to single breaths,
The boundaries of time itself denied,
Transcending mortal limits, conquering deaths.

My companions were silence and the stars,
The moonlight's silver and the sunrise gold,
The memories of ancient inner scars,
The stories waiting patiently untold.

And so I wandered, searching without end,
For meaning in the vast and spinning world,
With curiosity my truest friend,
And wonder like a banner ever-furled.

${longText}` },
  ],
  placeholder: "Ask anything...",
});

// Wait for render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `chat-word-wrap-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
