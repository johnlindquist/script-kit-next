#!/usr/bin/env bun
/**
 * Runtime proof for the kit:// resource preview keyboard contract (Day Page):
 * - Cmd+C while the preview is open copies the active preview URI exactly.
 * - Plain Enter is swallowed for collection previews (no source target): the
 *   preview stays open and nothing leaks into the hidden editor.
 * - Escape closes the preview and returns to the Day Page editor.
 *
 * Mirrors the clickable footer hints rendered by
 * `src/components/resource_preview.rs`.
 */
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/day-kit-actions/script-kit-gpui");

type Obj = Record<string, any>;

const runId = `day-kit-preview-keyboard-${Date.now()}`;
const receipt: Obj = {
  tool: "day-page-kit-preview-keyboard-probe",
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
  timeoutMs = 7000,
): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

async function pbpaste(): Promise<string> {
  return await Bun.$`pbpaste`.text().catch((error) => `__PBPASTE_ERROR__ ${String(error)}`);
}

async function getState(driver: Driver): Promise<Obj> {
  return asObj(await driver.getState({ timeoutMs: 8000 }));
}

async function dayPagePreview(driver: Driver): Promise<Obj> {
  return asObj(asObj((await getState(driver)).dayPage).kitResourcePreview);
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "day-page-kit-preview-keyboard",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const opened = asObj(await openDayPage(driver, runId));
  check("day_page_opened", opened.promptType === "dayPage", { promptType: opened.promptType });

  const seed = asObj(
    await driver.batch([{ type: "setInput", text: "[scripts](kit://scripts)" }], {
      timeoutMs: 8000,
    }),
  );
  check("kit_scripts_link_seeded", seed.success === true, { batch: seed });

  await driver.simulateKey(".", ["cmd"]);
  const openedPreview = await pollUntil("kit-scripts-preview-open", async () => {
    const preview = await dayPagePreview(driver);
    return preview.active === true && preview.uri === "kit://scripts";
  });
  check("kit_scripts_preview_opened", openedPreview, { preview: await dayPagePreview(driver) });

  // Cmd+C copies the active preview URI (keyboard path, not the ⌘K menu).
  await Bun.$`pbcopy < /dev/null`.quiet();
  await driver.simulateKey("c", ["cmd"]);
  const copied = await pollUntil("cmd-c-copies-uri", async () =>
    (await pbpaste()).trim() === "kit://scripts",
  );
  check("cmd_c_copies_active_preview_uri", copied, { copied: (await pbpaste()).trim() });

  // Plain Enter on a collection preview (no source target) is swallowed:
  // preview stays open, editor stays hidden behind it.
  await driver.simulateKey("enter", []);
  await Bun.sleep(300);
  const previewAfterEnter = await dayPagePreview(driver);
  check("enter_swallowed_for_collection_preview", previewAfterEnter.active === true, {
    preview: previewAfterEnter,
  });

  // Escape closes the preview and returns to the Day Page editor.
  await driver.simulateKey("escape", []);
  const closed = await pollUntil("escape-closes-preview", async () => {
    const state = await getState(driver);
    const preview = asObj(asObj(state.dayPage).kitResourcePreview);
    return state.promptType === "dayPage" && preview.active === false;
  });
  check("escape_closes_preview", closed, {
    state: asObj(asObj((await getState(driver)).dayPage).kitResourcePreview),
  });

  receipt.pass = receipt.failures.length === 0;
} finally {
  await driver.close().catch(() => {});
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
