#!/usr/bin/env bun

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const binary =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/layout-audit-final/script-kit-gpui";
const receiptPath = resolve(
  process.env.PROBE_RECEIPT ?? ".test-output/permissions-layout-frame-probe.json",
);

function component(layout: Json, name: string): Json | null {
  return ((layout.components ?? []) as Json[]).find((item) => item.name === name) ?? null;
}

function near(left: unknown, right: unknown, tolerance = 0.01): boolean {
  return Math.abs(Number(left) - Number(right)) <= tolerance;
}

mkdirSync(dirname(receiptPath), { recursive: true });
const receipt: Json = {
  schemaVersion: 1,
  tool: "permissions-layout-frame-probe",
  binary,
  pass: false,
  failures: [],
};

function check(name: string, pass: boolean, detail: Json = {}) {
  if (!pass) receipt.failures.push({ name, ...detail });
}

let driver: Driver | null = null;
try {
  driver = await Driver.launch({
    binary,
    sandboxHome: true,
    sessionName: "permissions-layout-frame",
    readyTimeoutMs: 60_000,
    defaultTimeoutMs: 10_000,
    env: {
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_STARTUP_PROFILE: "dev-fast",
    },
  });
  receipt.sessionDir = driver.sessionDir;

  // Drive the same launcher path a user takes. "Check Permissions" is the
  // first filtered result; "Set Up Permissions" is the second.
  await driver.setFilterAndWait("permissions", { timeoutMs: 10_000 });
  driver.simulateKey("down");
  driver.simulateKey("enter");
  const opened = await driver.waitForState(
    { promptType: "permissionsWizard" },
    { timeoutMs: 10_000 },
  );
  receipt.opened = opened;

  const state = (await driver.getState({ timeoutMs: 10_000 })) as Json;
  const elements = (await driver.getElements(
    { limit: 100 },
    { timeoutMs: 10_000 },
  )) as Json;
  const layout = (await driver.getLayoutInfo(
    {},
    { timeoutMs: 10_000 },
  )) as Json;
  const names = ((layout.components ?? []) as Json[]).map((item) => item.name);
  const semanticIds = ((elements.elements ?? []) as Json[]).map(
    (item) => item.semanticId,
  );
  const footerButtons = (state.activeFooter?.buttons ?? []) as Json[];

  const title = component(layout, "PermissionsTitle")?.bounds as Json | undefined;
  const introPanel = component(layout, "PermissionsIntroPanel")?.bounds as Json | undefined;
  const introText = component(layout, "PermissionsIntroText")?.bounds as Json | undefined;
  const introActions = component(layout, "PermissionsIntroActions")?.bounds as Json | undefined;
  const list = component(layout, "PermissionsList")?.bounds as Json | undefined;
  const firstRowText = component(layout, "PermissionsFirstRowText")?.bounds as Json | undefined;

  receipt.state = state;
  receipt.elements = elements;
  receipt.layout = layout;

  check("permissions_surface", state.promptType === "permissionsWizard", {
    promptType: state.promptType,
  });
  check(
    "permissions_specific_receipts",
    [title, introPanel, introText, introActions, list, firstRowText].every(Boolean) &&
      !names.includes("ScriptList") &&
      !names.includes("PreviewPanel"),
    { names },
  );
  check(
    "single_horizontal_frame",
    near(title?.x, introText?.x) &&
      near(title?.x, introActions?.x) &&
      near(title?.x, firstRowText?.x) &&
      near(introPanel?.x, list?.x) &&
      near(introPanel?.width, list?.width) &&
      Number(introPanel?.width) > 380,
    { title, introPanel, introText, introActions, list, firstRowText },
  );
  check(
    "permissions_semantics",
    semanticIds.includes("panel:permissions-wizard") &&
      semanticIds.includes("panel:permissions-intro") &&
      semanticIds.includes("list:permissions") &&
      semanticIds.some((id) => String(id).startsWith("permission-row:")) &&
      !semanticIds.includes("panel:current-view"),
    { semanticIds, warnings: elements.warnings ?? [] },
  );
  check(
    "single_native_footer_contract",
    state.activeFooter?.expectedSurface === "permissions_wizard" &&
      footerButtons.length === 2 &&
      footerButtons[0]?.action === "run" &&
      footerButtons[0]?.key === "↵" &&
      footerButtons[0]?.label === "Grant" &&
      footerButtons[1]?.action === "close" &&
      footerButtons[1]?.key === "Esc" &&
      footerButtons[1]?.label === "Done",
    { activeFooter: state.activeFooter },
  );

  receipt.pass = receipt.failures.length === 0;
} catch (error) {
  receipt.failures.push({
    name: "probe_exception",
    error: error instanceof Error ? error.stack ?? error.message : String(error),
  });
} finally {
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  await driver?.close();
}

console.log(JSON.stringify({ receiptPath, pass: receipt.pass, failures: receipt.failures }));
if (!receipt.pass) process.exitCode = 1;
