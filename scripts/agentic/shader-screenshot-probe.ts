// Glamour capture: main launcher with background shader effects enabled.
// Produces .test-screenshots/glamour/{22-shader-background,23-shader-background-alt}.png.
//
// Effects are read once at startup (src/effects.rs startup_prefs OnceLock),
// so each shot pre-seeds a sandbox home with `effects.background` in
// config.ts and launches a fresh app instance, then captures the main
// window via the in-app captureScreenshot protocol command.
//
// Run: bun scripts/agentic/shader-screenshot-probe.ts

import { join } from "node:path";
import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname;
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/glamour");
mkdirSync(OUT_DIR, { recursive: true });

const SHOTS: Array<{ name: string; effect: string }> = [
  { name: "22-shader-background", effect: "aurora" },
  { name: "23-shader-background-alt", effect: "starfield" },
];

const receipt: any = { shots: [], errors: [] };

for (const shot of SHOTS) {
  const home = `/tmp/sk-shader-glamour-home-${shot.effect}`;
  rmSync(home, { recursive: true, force: true });
  const kitDir = join(home, ".scriptkit");
  mkdirSync(kitDir, { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {\n  effects: { background: "${shot.effect}", intensity: 0.9 },\n};\n`,
  );

  const driver = await Driver.launch({
    binary: join(PROJECT_ROOT, "target-agent/artifacts/glamour/script-kit-gpui"),
    sessionName: `shader-${shot.effect}`,
    env: { HOME: home, SK_PATH: kitDir },
  });
  try {
    driver.send({ type: "show" });
    await driver.waitForSettle();
    // let the shader animate a few seconds to a visually interesting frame
    await Bun.sleep(3000);
    const png = join(OUT_DIR, `${shot.name}.png`);
    const result = (await driver.captureScreenshot({
      hiDpi: true,
      target: { type: "kind", kind: "main" },
      savePath: png,
    })) as { error?: string; data?: string };
    if (result.error || !result.data) {
      receipt.errors.push({ shot: shot.name, error: result.error ?? "no data" });
    } else {
      receipt.shots.push({ shot: shot.name, effect: shot.effect, png });
    }
  } finally {
    await driver.close();
  }
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.errors.length > 0 || receipt.shots.length !== SHOTS.length) process.exit(1);
