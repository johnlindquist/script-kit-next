# Layout Info Details

## Full Interface

```ts
interface LayoutInfo {
  windowWidth: number;
  windowHeight: number;
  promptType: string; // "arg"|"div"|"editor"|"mainMenu"|...
  components: LayoutComponentInfo[];
  timestamp: string; // ISO
}

interface LayoutComponentInfo {
  name: string;
  type: "prompt"|"input"|"button"|"list"|"listItem"|"header"|"container"|"panel"|"other";
  bounds: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  boxModel?: {
    padding?: { top: number; right: number; bottom: number; left: number };
    margin?: { top: number; right: number; bottom: number; left: number };
    gap?: number;
  };
  flex?: {
    direction?: "row" | "column";
    grow?: number;
    shrink?: number;
    basis?: string;
    alignItems?: string;
    justifyContent?: string;
  };
  depth: number;
  parent?: string;
  children: string[];
  explanation?: string;
}
```

## Combining Layout + Screenshot

Typical debug pattern:
```ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Render UI
await div(`<div class="p-4">Test content</div>`);
await new Promise(r => setTimeout(r, 500));

// Capture both
const [layout, shot] = await Promise.all([
  getLayoutInfo(),
  captureScreenshot()
]);

// Log key bounds
console.error('[LAYOUT]', JSON.stringify(layout.components.map(c => ({
  name: c.name,
  bounds: c.bounds
})), null, 2));

// Save screenshot
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `debug-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
```

## Grid Overlay with Test

Show overlay then run a test:
```bash
(echo '{"type":"showGrid","showBounds":true,"showDimensions":true}'; \
 echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-layout.ts"}') | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

Env alternative: `SCRIPT_KIT_DEBUG_GRID=1`
