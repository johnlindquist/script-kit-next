#!/usr/bin/env bun
/**
 * Runtime proof for Day Page deeplink activation:
 * - Cmd+. or a click on a kit://scripts link opens a read-only preview
 * - Escape closes the preview and returns to Day Page
 * - A click on plain text stays on Day Page
 */
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/note-deeplinks-iter3/script-kit-gpui");

type Obj = Record<string, any>;

const runId = `note-deeplinks-day-${Date.now()}`;
const receipt: Obj = {
  tool: "note-deeplinks-day-page-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
};

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) receipt.failures.push(name);
}

async function pollUntil(
  label: string,
  fn: () => Promise<boolean>,
  timeoutMs = 6000,
): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

async function setDayPageInput(driver: Driver, text: string) {
  return driver.batch([{ type: "setInput", text }], { timeoutMs: 8000 });
}

function stateSummary(state: Obj): Obj {
  return {
    promptType: state.promptType,
    windowVisible: state.windowVisible,
    inputValue: state.inputValue,
    dayPage: state.dayPage,
  };
}

async function getState(driver: Driver): Promise<Obj> {
  return asObj(await driver.getState({ timeoutMs: 8000 }));
}

async function getStateSummary(driver: Driver): Promise<Obj> {
  return stateSummary(await getState(driver));
}

async function dayPageKitResourcePreview(driver: Driver): Promise<Obj> {
  return asObj(asObj((await getState(driver)).dayPage).kitResourcePreview);
}

async function clickDayPageEditorFirstLine(driver: Driver) {
  const results = await driver.simulateGpuiClick(72, 104, {
    target: { type: "kind", kind: "main" },
    timeoutMs: 8000,
  });
  await Bun.sleep(150);
  return results.map(asObj);
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "note-deeplinks-day-page",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const opened = asObj(await openDayPage(driver, runId));
  check("day_page_opened", opened.promptType === "dayPage", {
    promptType: opened.promptType,
  });

  const kitPreviewSeed = asObj(await setDayPageInput(driver, "[scripts](kit://scripts)"));
  check("kit_scripts_link_seeded", kitPreviewSeed.success === true, { batch: kitPreviewSeed });
  await driver.simulateKey(".", ["cmd"]);
  const kitPreviewOpened = await pollUntil("kit-scripts-preview-open", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return (
      preview.active === true &&
      preview.uri === "kit://scripts" &&
      preview.readOnly === true
    );
  });
  const kitPreviewState = await dayPageKitResourcePreview(driver);
  check("kit_scripts_cmd_dot_opens_read_only_preview", kitPreviewOpened, {
    preview: kitPreviewState,
    state: await getStateSummary(driver),
  });
  await driver.simulateKey("escape");
  const kitPreviewClosed = await pollUntil("kit-scripts-preview-closed", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return preview.active === false;
  });
  check("escape_closes_kit_scripts_preview_and_returns_day_page", kitPreviewClosed, {
    state: await getStateSummary(driver),
    preview: await dayPageKitResourcePreview(driver),
  });

  const mousePlainSeed = asObj(await setDayPageInput(driver, "mouse plain text"));
  check("mouse_plain_text_seeded", mousePlainSeed.success === true, { batch: mousePlainSeed });
  const mousePlainClick = await clickDayPageEditorFirstLine(driver);
  check("mouse_plain_text_click_dispatches", mousePlainClick.every((result) => result.success), {
    click: mousePlainClick,
  });
  await Bun.sleep(200);
  const mousePlainState = await getStateSummary(driver);
  check("mouse_plain_text_click_stays_on_day_page", mousePlainState.promptType === "dayPage", {
    state: mousePlainState,
  });

  const mouseKitSeed = asObj(await setDayPageInput(driver, "[scripts](kit://scripts)"));
  check("mouse_kit_scripts_link_seeded", mouseKitSeed.success === true, { batch: mouseKitSeed });
  const mouseKitClick = await clickDayPageEditorFirstLine(driver);
  check("mouse_kit_scripts_click_dispatches", mouseKitClick.every((result) => result.success), {
    click: mouseKitClick,
  });
  const mouseKitPreviewOpened = await pollUntil("mouse-kit-scripts-preview-open", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return (
      preview.active === true &&
      preview.uri === "kit://scripts" &&
      preview.readOnly === true
    );
  });
  check("mouse_kit_scripts_opens_read_only_preview", mouseKitPreviewOpened, {
    preview: await dayPageKitResourcePreview(driver),
    state: await getStateSummary(driver),
  });
  await driver.simulateKey("escape");
  const mouseKitPreviewClosed = await pollUntil("mouse-kit-scripts-preview-closed", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return preview.active === false;
  });
  check("escape_closes_mouse_kit_scripts_preview", mouseKitPreviewClosed, {
    state: await getStateSummary(driver),
    preview: await dayPageKitResourcePreview(driver),
  });

  receipt.sessionDir = driver.sessionDir;
  receipt.pass = receipt.failures.length === 0;
  console.log(JSON.stringify(receipt, null, 2));
  if (!receipt.pass) process.exitCode = 1;
} finally {
  await driver.close();
}
