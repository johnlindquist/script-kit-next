#!/usr/bin/env bun
/**
 * scripts/agentic/spine-deferred-verify.ts
 *
 * Runtime verification for the spine-audit deferred-work slices:
 *  1. main-window rich subsearch still resolves through the shared
 *     spine attach resolver after its move to src/spine/attach.rs:
 *     "@calendar:standup" + Enter resolves to a compact token (input
 *     keeps the prompt; no launcher fall-through)
 *  2. the `>home` cwd anchor shows up as a token in the input area
 *     (status:input-cwd-anchor element) after the segment text is
 *     stripped
 *
 * Calendar data is injected via SCRIPT_KIT_CALENDAR_JSON so the
 * sandboxed app has a deterministic provider source.
 *
 * Usage: bun scripts/agentic/spine-deferred-verify.ts
 */

import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/spine-audit/script-kit-gpui");

const CALENDAR_JSON = JSON.stringify({
  items: [
    { title: "Standup", subtitle: "9:30 AM Daily" },
    { title: "Design Review", subtitle: "2:00 PM" },
  ],
});

const checks: Record<string, { pass: boolean; evidence: string[] }> = {};

function flatten(node: Json, out: Record<string, string>[]) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) return node.forEach((n) => flatten(n, out));
  const rec = node as Record<string, Json>;
  const entry: Record<string, string> = {};
  for (const key of ["semanticId", "semantic_id", "text", "value"]) {
    if (typeof rec[key] === "string") entry[key] = rec[key] as string;
  }
  if (Object.keys(entry).length > 0) out.push(entry);
  for (const v of Object.values(rec)) flatten(v, out);
}

async function main() {
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "spine-deferred",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
    env: { SCRIPT_KIT_CALENDAR_JSON: CALENDAR_JSON },
  });

  try {
    // 1. Shared resolver regression: @calendar: subsearch attach on Enter.
    await driver.setFilterAndWait("@calendar:standup");
    await Bun.sleep(400);
    driver.simulateKey("enter");
    await Bun.sleep(700);
    const afterAttach = await driver.getState();
    const attachInput = String(afterAttach.inputValue ?? "");
    checks.calendarAttachResolvesToken = {
      pass:
        attachInput.startsWith("@calendar:Standup") &&
        afterAttach.promptType !== "scriptOutput",
      evidence: [
        `inputValue=${attachInput}`,
        `promptType=${afterAttach.promptType}`,
      ],
    };

    // 2. cwd anchor token in the input area after `>home` + Enter.
    await driver.setFilterAndWait("");
    await Bun.sleep(150);
    await driver.setFilterAndWait(">home");
    await Bun.sleep(400);
    driver.simulateKey("enter");
    await Bun.sleep(700);
    const elements = await driver.getElements();
    const flat: Record<string, string>[] = [];
    flatten(elements as Json, flat);
    const anchor = flat.find((e) =>
      Object.values(e).some((v) => v.includes("input-cwd-anchor")),
    );
    checks.cwdAnchorTokenInInputArea = {
      pass: anchor !== undefined && (anchor.text ?? "").length > 0,
      evidence: [JSON.stringify(anchor ?? flat.slice(0, 6))],
    };
  } finally {
    await driver.close();
  }

  let failed = false;
  for (const [name, { pass, evidence }] of Object.entries(checks)) {
    console.log(`${pass ? "PASS" : "FAIL"} ${name}`);
    for (const e of evidence) console.log(`       ${e}`);
    if (!pass) failed = true;
  }
  process.exit(failed ? 1 : 0);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
