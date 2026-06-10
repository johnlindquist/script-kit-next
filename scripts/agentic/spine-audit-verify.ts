#!/usr/bin/env bun
/**
 * scripts/agentic/spine-audit-verify.ts
 *
 * Runtime verification for the 20260609 spine-behavior-audit implementation:
 *  1. fuzzy catalog filtering: "/rw" surfaces the /rewrite command row
 *  2. prompt-builder tail shows the "Ready to send" row for ".professional hello "
 *  3. honest tail: an unresolvable @token warns "Some context won't attach"
 *  4. A3 regression: "@fi" + Enter completes to "@file:" and never clears input
 *  5. postfix capture "todo; buy milk" opens the capture composer (form experience)
 *  6. '!' mode-exit really opens the Quick Terminal
 *
 * Scenarios 5 and 6 switch surfaces that protocol Escape cannot close, so
 * they run in their own driver sessions.
 *
 * Usage: bun scripts/agentic/spine-audit-verify.ts
 */

import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/spine-audit/script-kit-gpui");

const checks: Record<string, { pass: boolean; evidence: string[] }> = {};

function collectLabels(node: Json, out: string[]) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) return node.forEach((n) => collectLabels(n, out));
  const rec = node as Record<string, Json>;
  for (const key of ["label", "title", "text"]) {
    if (typeof rec[key] === "string") out.push(rec[key] as string);
  }
  for (const v of Object.values(rec)) collectLabels(v, out);
}

const has = (labels: string[], needle: string) =>
  labels.some((l) => l.toLowerCase().includes(needle.toLowerCase()));

async function withDriver(
  name: string,
  fn: (driver: Driver) => Promise<void>,
) {
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: name,
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  try {
    await fn(driver);
  } finally {
    await driver.close();
  }
}

async function labelsFor(driver: Driver, filter: string): Promise<string[]> {
  await driver.setFilterAndWait("");
  await Bun.sleep(150);
  await driver.setFilterAndWait(filter);
  await Bun.sleep(350);
  const elements = await driver.getElements();
  const labels: string[] = [];
  collectLabels(elements as Json, labels);
  return labels;
}

async function main() {
  await withDriver("spine-audit-list", async (driver) => {
    {
      const labels = await labelsFor(driver, "/rw");
      checks.fuzzySlashRewrite = {
        pass: has(labels, "rewrite"),
        evidence: labels.filter((l) => has([l], "rewrite")).slice(0, 4),
      };
    }
    {
      const labels = await labelsFor(driver, ".professional hello ");
      checks.tailReadyToSend = {
        pass: has(labels, "Ready to send"),
        evidence: labels.slice(0, 10),
      };
    }
    {
      const labels = await labelsFor(driver, "@selection @notes:groceries explain");
      checks.honestTailWarning = {
        pass:
          has(labels, "won't attach") ||
          labels.some((l) => l.includes("\u{26a0}")),
        evidence: labels.slice(0, 10),
      };
    }
    {
      await driver.setFilterAndWait("");
      await Bun.sleep(150);
      await driver.setFilterAndWait("@fi");
      await Bun.sleep(300);
      driver.simulateKey("enter");
      await Bun.sleep(700);
      const after = await driver.getState();
      checks.atFiEnterCompletes = {
        pass: after.inputValue === "@file:",
        evidence: [`after.inputValue=${after.inputValue}`],
      };
    }
  });

  await withDriver("spine-audit-capture", async (driver) => {
    await driver.setFilterAndWait("todo; buy milk");
    await Bun.sleep(500);
    const state = await driver.getState();
    const elements = await driver.getElements();
    const labels: string[] = [];
    collectLabels(elements as Json, labels);
    // The capture composer renders the capture target's form fields.
    checks.postfixCaptureComposer = {
      pass: has(labels, "Task") && has(labels, "Tags"),
      evidence: [`promptType=${state.promptType}`, ...labels.slice(0, 6)],
    };
  });

  await withDriver("spine-audit-bang", async (driver) => {
    await driver.setFilterAndWait("!");
    await Bun.sleep(500);
    const state = await driver.getState();
    checks.bangOpensQuickTerminal = {
      pass: state.promptType === "quickTerminal",
      evidence: [`promptType=${state.promptType}`],
    };
  });

  const allPass = Object.values(checks).every((c) => c.pass);
  console.log(JSON.stringify({ allPass, checks }, null, 2));
  process.exitCode = allPass ? 0 : 1;
}

main();
