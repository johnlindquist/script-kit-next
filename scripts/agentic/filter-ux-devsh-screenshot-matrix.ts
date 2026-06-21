#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY = process.env.PROBE_BINARY ?? "target/debug/script-kit-gpui";
const OUT_DIR =
  process.env.PROBE_OUT_DIR ?? ".artifacts/filter-ux-devsh-red";
const SANDBOX_HOME = process.env.PROBE_SANDBOX_HOME === "1";
const SESSION_NAME =
  process.env.PROBE_SESSION_NAME ??
  OUT_DIR.replace(/^.*\//, "").replace(/[^a-z0-9_-]/gi, "-");

const expectedHeadsT = ["type:", "tag:"];
const expectedTypeValues = [
  "type:script",
  "type:scriptlet",
  "type:skill",
  "type:builtin",
  "type:app",
  "type:window",
  "type:agent",
  "type:issue",
];

mkdirSync(OUT_DIR, { recursive: true });

function triggerTokens(elements: Json): string[] {
  return ((elements.elements ?? []) as Json[])
    .filter((element) => element.role === "menu-syntax-trigger-row")
    .map((element) => String(element.value ?? element.text ?? ""));
}

function visibleResultKeys(state: Json): string[] {
  return ((state.mainWindowPreflight?.visibleResults ?? []) as Json[]).map(
    (row) => String(row.stableKey ?? ""),
  );
}

function fallbackRowsVisible(state: Json): boolean {
  const keys = visibleResultKeys(state);
  return keys.some((key) => !key.startsWith("menu-syntax-trigger:"));
}

function equal(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

async function show(driver: Driver) {
  driver.send({ type: "show", requestId: "filter-ux-show" });
  await driver.waitForState(
    { windowVisible: true, promptType: "none" },
    { timeoutMs: 8000, pollIntervalMs: 25 },
  );
  await Bun.sleep(120);
}

async function capture(driver: Driver, slug: string, input?: string): Promise<Json> {
  if (input !== undefined) {
    await driver.setFilterAndWait(input, { timeoutMs: 8000 });
    await Bun.sleep(180);
  }
  await show(driver);
  const state = await driver.getState({ timeoutMs: 8000 });
  const elements = await driver.getElements({}, { timeoutMs: 8000 });
  const screenshotPath = join(OUT_DIR, `${slug}.png`);
  await driver.captureScreenshot({ savePath: screenshotPath, timeoutMs: 10000 });
  return {
    slug,
    screenshotPath,
    inputValue: state.inputValue,
    selectedValue: state.selectedValue ?? null,
    choiceCount: state.choiceCount,
    visibleChoiceCount: state.visibleChoiceCount,
    menuSyntaxKind: state.menuSyntaxMainHint?.kind ?? null,
    activeHead: state.menuSyntaxMainHint?.activeHead ?? null,
    activeHeadValuePartial:
      state.menuSyntaxMainHint?.activeHeadValuePartial ?? null,
    tokens: triggerTokens(elements),
    fallbackRowsVisible: fallbackRowsVisible(state),
    visibleResultKeys: visibleResultKeys(state).slice(0, 12),
  };
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: SANDBOX_HOME,
  sessionName: SESSION_NAME,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

const results: Record<string, Json> = {};
const checks: Record<string, boolean> = {};
let cleanup = "not-started";

try {
  results.colon = await capture(driver, "01-colon", ":");
  checks.colon_heads =
    results.colon.tokens.includes("type:") &&
    results.colon.tokens.includes("tag:") &&
    !results.colon.fallbackRowsVisible;

  results.colon_t = await capture(driver, "02-colon-t", ":t");
  checks.colon_t_heads_only =
    equal(results.colon_t.tokens, expectedHeadsT) &&
    !results.colon_t.tokens.includes("type:script") &&
    !results.colon_t.tokens.includes("todo:") &&
    !results.colon_t.tokens.includes("tabs:") &&
    !results.colon_t.fallbackRowsVisible;

  results.colon_ty = await capture(driver, "03-colon-ty", ":ty");
  checks.colon_ty_type_only =
    equal(results.colon_ty.tokens, ["type:"]) &&
    !results.colon_ty.fallbackRowsVisible;

  await driver.setFilterAndWait(":ty", { timeoutMs: 8000 });
  await Bun.sleep(120);
  driver.simulateKey("enter");
  await driver.waitForState({ inputValue: "type:" }, { timeoutMs: 8000 });
  await Bun.sleep(180);
  results.accept_type_from_colon_ty = await capture(
    driver,
    "04-accept-type-from-colon-ty",
  );
  checks.accept_type_from_colon_ty =
    results.accept_type_from_colon_ty.inputValue === "type:" &&
    equal(results.accept_type_from_colon_ty.tokens, expectedTypeValues) &&
    !results.accept_type_from_colon_ty.fallbackRowsVisible;

  results.type_direct = await capture(driver, "05-type-direct", "type:");
  checks.type_direct_values =
    equal(results.type_direct.tokens, expectedTypeValues) &&
    !results.type_direct.fallbackRowsVisible;

  driver.simulateKey("enter");
  await driver.waitForState({ inputValue: "type:script" }, { timeoutMs: 8000 });
  await Bun.sleep(120);
  results.type_enter_accepts_value = await capture(
    driver,
    "05b-type-enter-accepts-value",
  );
  checks.type_enter_accepts_value =
    results.type_enter_accepts_value.inputValue === "type:script" &&
    results.type_enter_accepts_value.tokens.length === 0;

  results.type_s = await capture(driver, "06-type-s", "type:s");
  checks.type_s_values =
    equal(results.type_s.tokens, [
      "type:script",
      "type:scriptlet",
      "type:skill",
    ]) && !results.type_s.fallbackRowsVisible;

  results.type_scr = await capture(driver, "07-type-scr", "type:scr");
  checks.type_scr_values =
    equal(results.type_scr.tokens, ["type:script", "type:scriptlet"]) &&
    !results.type_scr.fallbackRowsVisible;

  results.type_script_git = await capture(
    driver,
    "08-type-script-git",
    "type:script git",
  );
  checks.type_script_git_no_picker =
    results.type_script_git.inputValue === "type:script git" &&
    results.type_script_git.tokens.length === 0;

  await driver.setFilterAndWait("type:", { timeoutMs: 8000 });
  await Bun.sleep(120);
  driver.simulateKey("escape");
  await driver.waitForState({ inputValue: "" }, { timeoutMs: 8000 });
  results.escape_from_type = await capture(
    driver,
    "09-escape-from-type-after",
  );
  checks.escape_from_type_clears_once = results.escape_from_type.inputValue === "";

  await driver.close();
  cleanup = "driver.close completed";
} catch (error) {
  cleanup = "driver.close after error attempted";
  await driver.close().catch(() => {});
  throw error;
} finally {
  const report = {
    ok: Object.values(checks).every(Boolean),
    binary: BINARY,
    outDir: OUT_DIR,
    sandboxHome: SANDBOX_HOME,
    sessionDir: driver.sessionDir,
    cleanup,
    checks,
    results,
  };
  writeFileSync(join(OUT_DIR, "report.json"), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
}
