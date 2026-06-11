/**
 * Probe: show the main window (built from the current working tree) and print
 * its screen bounds + pid so the caller can screencapture the region over a
 * white backdrop and verify the vibrancy tint is backdrop-independent.
 */
import { Driver } from "../devtools/driver.ts";

const HOLD_MS = Number(process.env.VIBRANCY_PROBE_HOLD_MS ?? 8000);

const driver = await Driver.launch({
  binary: "target-agent/artifacts/vibrancy-085/script-kit-gpui",
  sandboxHome: true,
  sessionName: "vibrancy-tint-probe",
});

try {
  (driver as any).send({ type: "show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
  const layout = (await driver.getLayoutInfo()) as any;
  const win = layout.windowBounds ?? layout.window ?? layout;
  console.log(
    JSON.stringify({
      ok: true,
      pid: driver.pid,
      sessionDir: driver.sessionDir,
      windowBounds: win,
    }),
  );
  // Hold the window on screen so the caller can screencapture it.
  await new Promise((r) => setTimeout(r, HOLD_MS));
} finally {
  await driver.close();
}
