#!/usr/bin/env bun
/**
 * Theme Designer redesign probe.
 *
 * Proves, against the real app (sandbox HOME, hidden window, protocol only):
 *  1. `builtin/choose-theme` opens the unified catalog (list + panel-mode control).
 *  2. Arrow navigation live-previews a catalog entry (in-memory theme changes).
 *  3. Cmd+E toggles the right panel between Preview and Customize.
 *  4. setThemeControl drives appearance-mode / vibrancy-material / background-color.
 *  5. Typing a full hex into the filter live-previews that accent.
 *  6. Esc is a pure cancel: theme override file on disk is untouched.
 *  7. Enter commits: theme override file is written.
 */

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/theme-designer/script-kit-gpui";

type Check = { name: string; pass: boolean; detail: string };
const checks: Check[] = [];
function check(name: string, pass: boolean, detail: string) {
  checks.push({ name, pass, detail });
}

function elements(res: Json): Json[] {
  return (res.elements ?? []) as Json[];
}
function findEl(res: Json, semanticId: string): Json | undefined {
  return elements(res).find((e) => e.semanticId === semanticId || e.semantic_id === semanticId);
}
function elValue(res: Json, semanticId: string): string | null {
  return (findEl(res, semanticId)?.value as string | undefined) ?? null;
}

async function waitForThemeChooser(d: Driver, timeoutMs = 4000): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last: Json = {};
  while (Date.now() < deadline) {
    last = await d.getElements();
    if (findEl(last, "control:theme-chooser:panel-mode")) return last;
    await Bun.sleep(50);
  }
  throw new Error("Theme chooser never appeared in getElements");
}

async function waitForChooserGone(d: Driver, timeoutMs = 4000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const res = await d.getElements();
    if (!findEl(res, "control:theme-chooser:panel-mode")) return true;
    await Bun.sleep(50);
  }
  return false;
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "theme-designer-redesign",
});
const themeJsonPath = join(driver.sessionDir, "home", ".scriptkit", "theme.json");
const themeJsonFingerprint = () =>
  existsSync(themeJsonPath) ? readFileSync(themeJsonPath, "utf8") : null;

try {
  // --- 1. Open the Theme Designer through the real builtin path ------------
  driver.send({ type: "triggerBuiltin", builtinId: "builtin/choose-theme" });
  let els = await waitForThemeChooser(driver);

  const list = findEl(els, "list:theme-catalog");
  check(
    "open-unified-catalog",
    Boolean(list),
    list
      ? `list:theme-catalog present; text=${list.text}`
      : `missing; ids=${elements(els)
          .slice(0, 12)
          .map((e) => e.semanticId ?? e.semantic_id)
          .join(",")}`,
  );
  check(
    "panel-mode-defaults-preview",
    elValue(els, "control:theme-chooser:panel-mode") === "preview",
    `panel-mode=${elValue(els, "control:theme-chooser:panel-mode")}`,
  );
  const diskBeforeAnyPreview = themeJsonFingerprint();

  // --- 2. Live preview on arrow navigation ---------------------------------
  const accentBefore = elValue(els, "control:theme-chooser:accent-color");
  const bgBefore = elValue(els, "control:theme-chooser:background-color");
  let previewChanged = false;
  let stepsTaken = 0;
  for (let i = 0; i < 6 && !previewChanged; i++) {
    driver.simulateKey("down", []);
    stepsTaken += 1;
    await Bun.sleep(80);
    els = await driver.getElements();
    const accentNow = elValue(els, "control:theme-chooser:accent-color");
    const bgNow = elValue(els, "control:theme-chooser:background-color");
    previewChanged = accentNow !== accentBefore || bgNow !== bgBefore;
  }
  check(
    "arrow-navigation-live-previews",
    previewChanged,
    `accent ${accentBefore} -> ${elValue(els, "control:theme-chooser:accent-color")}, bg ${bgBefore} -> ${elValue(els, "control:theme-chooser:background-color")} after ${stepsTaken} downs`,
  );

  // --- 3. Cmd+E toggles Preview <-> Customize ------------------------------
  driver.simulateKey("e", ["cmd"]);
  await Bun.sleep(80);
  els = await driver.getElements();
  const modeAfterToggle = elValue(els, "control:theme-chooser:panel-mode");
  driver.simulateKey("e", ["cmd"]);
  await Bun.sleep(80);
  els = await driver.getElements();
  const modeAfterToggleBack = elValue(els, "control:theme-chooser:panel-mode");
  check(
    "cmd-e-toggles-panel-mode",
    modeAfterToggle === "customize" && modeAfterToggleBack === "preview",
    `preview -> ${modeAfterToggle} -> ${modeAfterToggleBack}`,
  );

  // --- 4. setThemeControl drives the new controls ---------------------------
  const batchRes = await driver.batch([
    { type: "setThemeControl", control: "panel-mode", value: "customize" },
    { type: "setThemeControl", control: "appearance-mode", value: "dark" },
    { type: "setThemeControl", control: "vibrancy-material", value: "hud" },
    { type: "setThemeControl", control: "background-color", value: "#102030" },
  ]);
  els = await driver.getElements();
  const batchOk =
    (batchRes.results as Json[] | undefined)?.every((r) => r.success) ?? false;
  check(
    "set-theme-control-batch",
    batchOk &&
      elValue(els, "control:theme-chooser:panel-mode") === "customize" &&
      elValue(els, "control:theme-chooser:appearance-mode") === "dark" &&
      elValue(els, "control:theme-chooser:vibrancy-material") === "hud" &&
      elValue(els, "control:theme-chooser:background-color") === "#102030",
    `batchOk=${batchOk} appearance=${elValue(els, "control:theme-chooser:appearance-mode")} material=${elValue(els, "control:theme-chooser:vibrancy-material")} bg=${elValue(els, "control:theme-chooser:background-color")}`,
  );

  // --- 5. Hex paste in the filter previews the accent -----------------------
  await driver.setFilterAndWait("#FF5500");
  await Bun.sleep(80);
  els = await driver.getElements();
  check(
    "hex-filter-previews-accent",
    elValue(els, "control:theme-chooser:accent-color") === "#FF5500",
    `accent=${elValue(els, "control:theme-chooser:accent-color")} after filter '#FF5500'`,
  );
  await driver.setFilterAndWait("");

  // --- 6. Esc is a pure cancel: no disk write -------------------------------
  driver.simulateKey("escape", []);
  const closedAfterEsc = await waitForChooserGone(driver);
  const diskAfterCancel = themeJsonFingerprint();
  check(
    "esc-cancel-does-not-persist",
    closedAfterEsc && diskAfterCancel === diskBeforeAnyPreview,
    `closedAfterEsc=${closedAfterEsc} theme.json before=${diskBeforeAnyPreview === null ? "absent" : `${diskBeforeAnyPreview.length}B`} after-cancel=${diskAfterCancel === null ? "absent" : `${diskAfterCancel.length}B`}`,
  );

  // --- 7. Enter commits: disk write happens ---------------------------------
  driver.send({ type: "triggerBuiltin", builtinId: "builtin/choose-theme" });
  els = await waitForThemeChooser(driver);
  await driver.batch([
    { type: "setThemeControl", control: "accent-color", value: "#22AA66" },
  ]);
  driver.simulateKey("enter", []);
  const closedAfterEnter = await waitForChooserGone(driver);
  await Bun.sleep(150);
  const diskAfterCommit = themeJsonFingerprint();
  check(
    "enter-commits-to-disk",
    closedAfterEnter &&
      diskAfterCommit !== null &&
      diskAfterCommit !== diskBeforeAnyPreview &&
      diskAfterCommit.toLowerCase().includes("22aa66"),
    `closedAfterEnter=${closedAfterEnter} theme.json after-commit=${diskAfterCommit === null ? "absent" : `${diskAfterCommit.length}B`} containsAccent=${diskAfterCommit?.toLowerCase().includes("22aa66") ?? false}`,
  );
} catch (error) {
  check("probe-error", false, String(error));
} finally {
  await driver.close();
}

const failed = checks.filter((c) => !c.pass);
console.log(
  JSON.stringify(
    {
      schemaVersion: 1,
      probe: "theme-designer-redesign",
      binary: BINARY,
      sessionDir: driver.sessionDir,
      status: failed.length === 0 ? "green" : "red",
      passed: checks.length - failed.length,
      failed: failed.length,
      checks,
    },
    null,
    2,
  ),
);
process.exit(failed.length === 0 ? 0 : 1);
