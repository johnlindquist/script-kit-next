/**
 * Runtime proof for the stuck-HUD fix (hud_manager deterministic NSWindow config).
 *
 * Red condition (pre-fix): showing several identically-sized HUD pills in quick
 * succession made configure_hud_window_by_size() match the OLDEST pill in
 * [NSApp windows] ("backdrop reused" in logs) and orderFrontRegardless a window
 * GPUI was concurrently closing — leaving a permanent ghost pill on screen.
 *
 * Green condition (post-fix): every HUD logs its own "Configured HUD NSWindow"
 * with a fresh "backdrop installed" (never "reused" while siblings are alive),
 * and after all durations elapse no HUD automation windows remain registered.
 *
 * Usage: bun scripts/agentic/hud-overlap-dismiss-probe.ts [path-to-binary]
 */
import { Driver } from "../devtools/driver.ts";

const binary = process.argv[2] ?? "target-agent/artifacts/tab-hud-fix/script-kit-gpui";

const receipt: Record<string, unknown> = { probe: "hud-overlap-dismiss", binary };
const driver = await Driver.launch({ binary, sandboxHome: true, sessionName: "hud-overlap" });
try {
  // Mirror the incident timing: 4 HUDs ~150ms apart (slots 0..2 + one queued),
  // then 2 more after the first wave starts expiring so a dismissal's cleanup
  // sweep overlaps a sibling's open_window in the same update tick.
  for (let i = 0; i < 4; i++) {
    driver.send({ type: "hud", text: "Failed to switch browser tab", duration_ms: 2000 });
    await Bun.sleep(150);
  }
  await Bun.sleep(2100);
  for (let i = 0; i < 2; i++) {
    driver.send({ type: "hud", text: "Failed to switch browser tab", duration_ms: 2000 });
    await Bun.sleep(150);
  }

  // Wait for every duration + dismissal timer to fire, then settle.
  await Bun.sleep(4000);

  const windows = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 5000 },
  )) as { windows?: Array<{ id?: string; kind?: string }> };
  const windowRows = windows.windows ?? [];
  const hudWindows = windowRows.filter(
    (w) => (w.id ?? "").includes("hud") || (w.kind ?? "").toLowerCase().includes("hud"),
  );
  receipt.remainingHudAutomationWindows = hudWindows;

  const log = await Bun.file(
    `${driver.sessionDir}/home/.scriptkit/logs/script-kit-gpui.jsonl`,
  ).text();
  receipt.sessionDir = driver.sessionDir;
  const count = (needle: string) => log.split(needle).length - 1;
  receipt.shown = count("Showing HUD: 'Failed to switch browser tab'");
  receipt.windowsCreated = count("HUD window created");
  receipt.configured = count("Configured HUD NSWindow");
  receipt.backdropInstalled = count("backdrop installed");
  receipt.backdropReused = count("backdrop reused");
  receipt.dismissed = count("Dismissed HUD id=");
  receipt.cleanupSweeps = count("Cleaned up");
  receipt.configResolutionFailures =
    count("Could not resolve raw window handle") + count("HUD NSView has no NSWindow");

  // Invariants:
  // - 6 windows were created and EACH got its own native configuration with a
  //   fresh glass backdrop (a "reused" backdrop means the size scan hit an old
  //   pill — the red condition that produced immortal ghost HUDs).
  // - The dismissal/cleanup overlap path actually ran (>=1 cleanup sweep).
  // - Nothing is left registered once all durations elapsed.
  const pass =
    hudWindows.length === 0 &&
    (receipt.windowsCreated as number) === 6 &&
    (receipt.configured as number) === 6 &&
    (receipt.backdropInstalled as number) === 6 &&
    (receipt.backdropReused as number) === 0 &&
    (receipt.cleanupSweeps as number) >= 1 &&
    (receipt.configResolutionFailures as number) === 0;
  receipt.pass = pass;
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
