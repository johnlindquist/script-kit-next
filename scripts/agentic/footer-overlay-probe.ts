#!/usr/bin/env bun
/** One-off: does the GPUI footer overlay window exist when the spike is on? */
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/pools/agent-debug/debug/script-kit-gpui",
);

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "footer-overlay-probe",
  sandboxHome: true,
  env: { SCRIPT_KIT_GPUI_FOOTER_OVERLAY_SPIKE: "1" },
});
try {
  driver.send({ type: "show" });
  await driver.waitForState({ windowFocused: true }, { timeoutMs: 5000 }).catch(() => {});
  await Bun.sleep(800);
  const windows = await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 5000 },
  );
  const main = (windows.windows ?? []).find((w: any) => w.id === "main");
  if (!main) throw new Error("main window not found");
  const { x, y, width, height } = main.bounds;
  const shot = join(
    PROJECT_ROOT,
    ".test-screenshots/footer-flexbox-compare/footer-flexbox-screen.png",
  );
  const { execSync } = await import("node:child_process");
  execSync(`screencapture -x -R${x},${y},${width},${height} "${shot}"`);
  console.log(JSON.stringify({ shot, bounds: main.bounds }, null, 2));
} finally {
  await driver.close();
}
