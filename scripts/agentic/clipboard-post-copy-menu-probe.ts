#!/usr/bin/env bun
/**
 * Clipboard sediment no-popup runtime probe.
 *
 * The historical filename is kept so existing checklists still find the probe,
 * but the expected behavior is now: copy keepable content -> brain sediment
 * records it -> no post-copy popup automation window appears.
 *
 *   bun scripts/agentic/clipboard-post-copy-menu-probe.ts
 */

import { readdir, readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/artifacts/t12-post-copy/script-kit-gpui",
);

const POPUP_AUTOMATION_ID = "clipboard-post-copy-menu";
const PROBE_URL = `https://example.com/script-kit-t12-${Date.now()}`;

async function findPostCopyPopup(driver: Driver) {
  const windows = await driver.listAutomationWindows();
  const list = (windows.windows ?? []) as Array<Record<string, any>>;
  return (
    list.find((w) => w.automationId === POPUP_AUTOMATION_ID) ??
    list.find((w) => w.semanticSurface === "clipboardPostCopyMenu") ??
    null
  );
}

async function readDayFiles(brainDaysDir: string) {
  const names = await readdir(brainDaysDir).catch(() => [] as string[]);
  const contents: string[] = [];
  for (const name of names.filter((name) => name.endsWith(".md"))) {
    contents.push(await readFile(join(brainDaysDir, name), "utf8"));
  }
  return contents;
}

async function waitForBrainUrl(brainDaysDir: string, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const contents = await readDayFiles(brainDaysDir);
    if (contents.some((content) => content.includes(PROBE_URL))) {
      return contents;
    }
    await Bun.sleep(100);
  }
  return readDayFiles(brainDaysDir);
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "clipboard-sediment-no-popup",
  sandboxHome: true,
});

const report: Record<string, any> = { probe_url: PROBE_URL };

try {
  await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
  await Bun.sleep(500);

  await Bun.$`printf ${PROBE_URL} | pbcopy`.quiet();

  const brainDir = join(driver.sandboxHome ?? "", ".scriptkit/brain/days");
  const contents = await waitForBrainUrl(brainDir);
  const joined = contents.join("\n");
  const occurrences = joined.split(PROBE_URL).length - 1;
  const popup = await findPostCopyPopup(driver);

  report.brain_day_page_contains_url = occurrences >= 1;
  report.url_occurrences = occurrences;
  report.post_copy_popup_present = Boolean(popup);
  report.post_copy_popup = popup;
  report.ok = report.brain_day_page_contains_url && !report.post_copy_popup_present;

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
