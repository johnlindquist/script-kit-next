/**
 * Probe: capture the main window over a colorful backdrop in one backdrop
 * configuration (controlled by SCRIPT_KIT_DEBUG_HIDE_VEV /
 * SCRIPT_KIT_DEBUG_NO_GLASS set by the caller), tagged via VIB_TAG.
 * Used to isolate what each native layer (glass vs effect views)
 * contributes to the on-screen result.
 */
import { Driver } from "../devtools/driver.ts";

const TAG = process.env.VIB_TAG ?? "untagged";

const driver = await Driver.launch({
  binary: "target-agent/artifacts/vibrancy-085/script-kit-gpui",
  sandboxHome: true,
  sessionName: `vib-iso-${TAG}`,
});

try {
  (driver as any).send({ type: "show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
  await new Promise((r) => setTimeout(r, 2500));
  const proc = Bun.spawn([
    "screencapture",
    "-x",
    `/tmp/vibiso-${TAG}-d1.png`,
    `/tmp/vibiso-${TAG}-d2.png`,
    `/tmp/vibiso-${TAG}-d3.png`,
  ]);
  await proc.exited;

  // Instance-isolation proof: enumerate ALL on-screen windows owned by any
  // script-kit process at capture time (compiled C tool over
  // CGWindowListCopyWindowInfo; fails loudly rather than reading as "none").
  const wins = Bun.spawnSync(["/tmp/list-sk-windows"]);
  if (wins.exitCode !== 0) {
    throw new Error(`list-sk-windows exit ${wins.exitCode}`);
  }
  const onScreen = JSON.parse(wins.stdout.toString().trim());

  console.log(
    JSON.stringify({
      ok: true,
      tag: TAG,
      appPid: driver.pid,
      onScreenScriptKitWindows: onScreen,
      sessionDir: driver.sessionDir,
    }),
  );
} finally {
  await driver.close();
}
