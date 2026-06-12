#!/usr/bin/env bun
/**
 * Post-copy ⌘-tap quick menu runtime probe (T12).
 *
 * Simulates: copy text → bare ⌘ tap → menu visible → annotate → brain doc
 * contains why → reject path removes entry + day-page line.
 *
 * Requires accessibility permission for the CGEventTap lane and cliclick for
 * synthetic modifier events.
 *
 *   bun scripts/agentic/clipboard-post-copy-menu-probe.ts
 */

import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/artifacts/t12-post-copy/script-kit-gpui",
);

const MENU_AUTOMATION_ID = "clipboard-post-copy-menu";
const PROBE_TEXT = `t12-probe-${Date.now()}`;
const WHY_TEXT = "auth doc reference";

async function findMenuWindow(driver: Driver) {
  const windows = await driver.listAutomationWindows();
  const list = (windows.windows ?? []) as Array<Record<string, any>>;
  return (
    list.find((w) => w.automationId === MENU_AUTOMATION_ID) ??
    list.find((w) => w.semanticSurface === "clipboardPostCopyMenu") ??
    null
  );
}

async function waitForMenu(driver: Driver, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const menu = await findMenuWindow(driver);
    if (menu) return menu;
    await Bun.sleep(50);
  }
  throw new Error("post-copy menu window never appeared");
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "clipboard-post-copy-menu",
  sandboxHome: true,
});

const report: Record<string, any> = { probe_text: PROBE_TEXT };

try {
  await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
  await Bun.sleep(500);

  // Seed clipboard with keepable text (non-URL, single copy).
  await Bun.$`printf ${PROBE_TEXT} | pbcopy`.quiet();
  await Bun.sleep(800);

  // Bare ⌘ tap: press and release command without other keys.
  await Bun.$`cliclick kd:cmd`.quiet();
  await Bun.sleep(80);
  await Bun.$`cliclick ku:cmd`.quiet();

  const menu = await waitForMenu(driver);
  report.menu_appeared = true;
  report.menu_bounds = menu.bounds;

  const elements = await driver.getElements({ automationId: MENU_AUTOMATION_ID });
  report.menu_elements = elements?.elements?.length ?? 0;

  // Select Annotate row via keyboard (first row default).
  driver.simulateKey("enter", []);
  await Bun.sleep(300);

  for (const ch of WHY_TEXT) {
    driver.simulateKey(ch, []);
  }
  driver.simulateKey("enter", []);
  await Bun.sleep(500);

  const brainDir = join(driver.sandboxHome ?? "", ".scriptkit/brain/days");
  const dayFiles = await Array.fromAsync(
    new Bun.Glob("*.md").scan(brainDir).catch(() => [] as string[]),
  );
  let whyFound = false;
  for (const file of dayFiles) {
    const content = await Bun.file(join(brainDir, file)).text();
    if (content.includes(PROBE_TEXT) && content.includes(WHY_TEXT)) {
      whyFound = true;
      break;
    }
  }
  report.annotate_why_on_day_page = whyFound;

  // Re-copy and reject path.
  await Bun.$`printf ${PROBE_TEXT} | pbcopy`.quiet();
  await Bun.sleep(800);
  await Bun.$`cliclick kd:cmd`.quiet();
  await Bun.sleep(80);
  await Bun.$`cliclick ku:cmd`.quiet();
  await waitForMenu(driver);
  driver.simulateKey("down", []);
  driver.simulateKey("enter", []);
  await Bun.sleep(500);

  let dayStillHasProbe = false;
  for (const file of dayFiles) {
    const content = await Bun.file(join(brainDir, file)).text();
    if (content.includes(PROBE_TEXT)) {
      dayStillHasProbe = true;
    }
  }
  report.reject_removed_day_line = !dayStillHasProbe;

  report.ok =
    report.menu_appeared &&
    report.annotate_why_on_day_page &&
    report.reject_removed_day_line;

  console.log(JSON.stringify(report, null, 2));
  if (!report.ok) {
    process.exit(1);
  }
} catch (error) {
  report.error = String(error);
  console.log(JSON.stringify(report, null, 2));
  process.exit(1);
} finally {
  await driver.shutdown().catch(() => {});
}
