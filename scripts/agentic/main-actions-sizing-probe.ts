/**
 * Measurement probe: main-window Cmd+K actions popup dynamic sizing.
 *
 * For each state (full list, filtered, zero results "qwerty") capture:
 *  - NSWindow bounds (listAutomationWindows)
 *  - dialog rowGeometry (getState target actionsDialog): per-row rects,
 *    viewport bounds, search/header bounds
 *  - a screenshot of the popup window
 * and compare the window height against the sum of rendered parts to find
 * where dynamic sizing drifts from the list content.
 */
import { Driver } from "../devtools/driver";

const BINARY = "target-agent/artifacts/notes-popup-fix/script-kit-gpui";

function osa(script: string) {
  return Bun.$`osascript -e ${script}`.quiet();
}

async function keystroke(text: string, mods: string[] = []) {
  const using = mods.length
    ? ` using {${mods.map((m) => `${m} down`).join(", ")}}`
    : "";
  await osa(`tell application "System Events" to keystroke "${text}"${using}`);
}

async function keyCode(code: number) {
  await osa(`tell application "System Events" to key code ${code}`);
}

type Json = Record<string, any>;

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "main-actions-sizing-probe",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function dialogState(): Promise<Json | null> {
  try {
    const result = (await driver.request(
      { type: "getState", target: { type: "kind", kind: "actionsDialog" } },
      { expect: "stateResult", timeoutMs: 3000 },
    )) as Json;
    return result.actionsDialog ?? null;
  } catch {
    return null;
  }
}

async function popupBounds(): Promise<Json | null> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  const windows: Json[] = result.windows ?? [];
  return windows.find((w) => w.id === "actions-dialog")?.bounds ?? null;
}

async function screenshot(name: string): Promise<string | null> {
  try {
    const result = (await driver.request(
      {
        type: "captureScreenshot",
        target: { type: "kind", kind: "actionsDialog" },
      },
      { expect: "screenshotResult", timeoutMs: 5000 },
    )) as Json;
    if (result.data) {
      const dest = `.test-screenshots/sizing-${name}.png`;
      await Bun.$`mkdir -p .test-screenshots`.quiet();
      await Bun.write(dest, Buffer.from(result.data, "base64"));
      return dest;
    }
    return result.error ?? null;
  } catch (e) {
    return `error: ${e}`;
  }
}

function summarizeGeometry(state: Json | null, bounds: Json | null): Json {
  const geometry = state?.rowGeometry ?? {};
  const rows: Json[] = geometry.rows ?? [];
  const actionRows = rows.filter((r) => r.kind === "action");
  const sectionRows = rows.filter((r) => r.kind === "section");
  const contentSum = rows.reduce(
    (sum: number, r: Json) => sum + (r.bounds?.height ?? 0),
    0,
  );
  const viewport = geometry.viewport ?? {};
  const listBounds = viewport.listBounds ?? geometry.listViewportRect ?? {};
  const searchBounds = viewport.searchBounds ?? null;
  const headerBounds = viewport.contextHeaderBounds ?? null;
  const lastVisible = [...rows]
    .reverse()
    .find((r) => r.visible === true) as Json | undefined;
  const listTop = listBounds.y ?? 0;
  const listBottom = listTop + (listBounds.height ?? 0);
  return {
    windowHeight: bounds?.height ?? null,
    actionRowCount: actionRows.length,
    sectionRowCount: sectionRows.length,
    rowHeights: [...new Set(rows.map((r: Json) => r.bounds?.height))],
    contentSum,
    listViewport: listBounds,
    searchBounds,
    headerBounds,
    lastVisibleRow: lastVisible
      ? {
          index: lastVisible.visualIndex,
          y: lastVisible.bounds?.y,
          height: lastVisible.bounds?.height,
          bottom: (lastVisible.bounds?.y ?? 0) + (lastVisible.bounds?.height ?? 0),
          listBottom,
          gapBelowLastVisibleRow: listBottom - ((lastVisible.bounds?.y ?? 0) + (lastVisible.bounds?.height ?? 0)),
        }
      : null,
    // parts: search + header + listViewport + (window border etc) vs window
    partsSum:
      (searchBounds?.height ?? 0) +
      (headerBounds?.height ?? 0) +
      (listBounds.height ?? 0),
    windowMinusParts:
      (bounds?.height ?? 0) -
      ((searchBounds?.height ?? 0) +
        (headerBounds?.height ?? 0) +
        (listBounds.height ?? 0)),
    searchTextLength: state?.search?.textLength,
  };
}

const report: Json = {};

try {
  driver.send({ type: "show", requestId: "probe-show-main" });
  await Bun.sleep(1500);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(600);

  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  report.full_list = summarizeGeometry(await dialogState(), await popupBounds());
  report.full_list.screenshot = await screenshot("full-list");

  await keystroke("set");
  await Bun.sleep(900);
  report.filtered = summarizeGeometry(await dialogState(), await popupBounds());
  report.filtered.screenshot = await screenshot("filtered");

  // Clear back to empty, then type a no-match query.
  for (let i = 0; i < 3; i++) {
    await keyCode(51); // backspace
    await Bun.sleep(150);
  }
  await keystroke("qwerty");
  await Bun.sleep(900);
  report.zero_results = summarizeGeometry(await dialogState(), await popupBounds());
  report.zero_results.screenshot = await screenshot("zero-results");

  await keyCode(53); // escape
  await Bun.sleep(700);
  report.closed = { popupBounds: await popupBounds() };
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
