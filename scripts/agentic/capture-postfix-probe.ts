#!/usr/bin/env bun
/**
 * scripts/agentic/capture-postfix-probe.ts
 *
 * Runtime proof for the A4 decision (2026-06-09): capture targets use a
 * postfix `;` sigil.
 *
 *   1. Typing ";" as the first character lists the available capture targets.
 *   2. Typing ";to" then Enter converts the input to "todo; " (postfix).
 *   3. "todo; " hands the input to the capture form (fields visible,
 *      composer owns the input — search is suppressed).
 *   4. Typing "todo;" directly reaches the same composer state.
 *
 * Usage: bun scripts/agentic/capture-postfix-probe.ts
 */

import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/input-ambiguity/script-kit-gpui");

function collectLabels(node: Json, labels: string[]) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const child of node) collectLabels(child, labels);
    return;
  }
  const record = node as Record<string, Json>;
  for (const key of ["semanticId", "text", "value"]) {
    if (typeof record[key] === "string") labels.push(record[key] as string);
  }
  for (const value of Object.values(record)) collectLabels(value, labels);
}

async function labelsNow(driver: Driver, target?: string): Promise<string[]> {
  const labels: string[] = [];
  const extra: Json = target ? { target: { type: "kind", kind: target } } : {};
  try {
    collectLabels(await driver.getElements(extra), labels);
  } catch {
    // window kind may not exist; treat as empty
  }
  return labels;
}


async function main() {
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "capture-postfix",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  try {
    // Hidden-window probe: stdin simulateKey reaches spine enter handling
    // without stealing the user's focus.

    // Step 1: ";" lists capture targets (trigger popup window).
    await driver.setFilterAndWait(";");
    await Bun.sleep(500);
    const semicolonLabels = [
      ...(await labelsNow(driver)),
      ...(await labelsNow(driver, "promptPopup")),
    ];
    // The trigger-popup capture catalog: todo/note/link/snippet (+cal/social).
    const targetTitles = ["Todo", "Note", "Link", "Snippet"];
    const listedTargets = targetTitles.filter((t) =>
      semicolonLabels.some((l) => l.toLowerCase().includes(t.toLowerCase())),
    );

    // Step 2: ";to" + Enter converts to "todo; ".
    await driver.setFilterAndWait(";to");
    await Bun.sleep(300);
    driver.simulateKey("enter");
    await Bun.sleep(700);
    const afterEnter = await driver.getState();
    const afterEnterLabels = await labelsNow(driver);

    // Step 4: typing "todo;" directly reaches the composer too.
    await driver.setFilterAndWait("clear-first");
    await driver.setFilterAndWait("todo; Renew passport");
    await Bun.sleep(500);
    const direct = await driver.getState();
    const directLabels = await labelsNow(driver);

    const formMarkers = (labels: string[]) =>
      labels.filter((l) => /todo|body|tags|priority|due|capture/i.test(l)).slice(0, 15);

    console.log(
      JSON.stringify(
        {
          sessionDir: driver.sessionDir,
          step1: {
            listedTargets,
            pass: listedTargets.length === targetTitles.length,
          },
          step2: {
            inputValue: afterEnter.inputValue,
            promptType: afterEnter.promptType,
            formMarkers: formMarkers(afterEnterLabels),
            pass: (afterEnter.inputValue ?? "").startsWith("todo;"),
          },
          step4: {
            inputValue: direct.inputValue,
            visibleChoiceCount: direct.visibleChoiceCount,
            formMarkers: formMarkers(directLabels),
          },
          step2FormFieldIds: formMarkers(afterEnterLabels).filter((l) =>
            l.startsWith("handler-form:"),
          ),
          pass:
            listedTargets.length === targetTitles.length &&
            (afterEnter.inputValue ?? "").startsWith("todo;") &&
            afterEnterLabels.some((l) => l.startsWith("handler-form:todo:")),
        },
        null,
        2,
      ),
    );
  } finally {
    await driver.close();
  }
}

main();
