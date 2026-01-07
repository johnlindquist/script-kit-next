// Name: Editor Layout Debug
// Description: Debug the editor layout with detailed measurements
import "../../scripts/kit-sdk";
import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";

// Show editor
await editor({
  value: "// Debug test\nconst x = 1;\n\n// More content",
  language: "typescript",
});

// Wait for render
await new Promise((r) => setTimeout(r, 800));

// Get layout info
const layout = await getLayoutInfo();
console.error("[LAYOUT] Window: " + layout.windowWidth + "x" + layout.windowHeight);
console.error("[LAYOUT] Prompt type: " + layout.promptType);
console.error("[LAYOUT] Components:");
for (const comp of layout.components) {
  console.error(`[LAYOUT]   ${comp.name}: ${comp.bounds.width}x${comp.bounds.height} at (${comp.bounds.x}, ${comp.bounds.y})`);
}

// Capture screenshot
const shot = await captureScreenshot();
const dir = join(process.cwd(), "test-screenshots", "layout-debug");
mkdirSync(dir, { recursive: true });
const path = join(dir, `editor-layout-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, "base64"));
console.error("[SCREENSHOT] " + path);

process.exit(0);
