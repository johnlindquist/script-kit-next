/**
 * Probe: `.` style-selector rewrite flow footer contract.
 *
 * Proves three behaviors from the 2026-06-11 user bug report:
 * 1. Empty launcher filter → footer has the Agent ⌘↵ button (baseline).
 * 2. Filter "." (style selector, style-only parse) → the Enter button says
 *    "Rewrite" (not "Attach") and the Agent button is GONE.
 * 3. Filter "@file ." style is part of a larger composition → label falls
 *    back to "Attach" and the Agent button returns (only the style-only
 *    auto-submit state drops it).
 *
 * Run: bun scripts/agentic/probe-style-rewrite-footer.ts
 */
import { Driver } from "../devtools/driver";

const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/rewrite-flow/script-kit-gpui";

type FooterButton = { key?: string; label?: string; action?: string };

function footerButtons(state: any): FooterButton[] {
  return (state?.activeFooter?.buttons ?? []) as FooterButton[];
}

function summarize(state: any) {
  return footerButtons(state).map((b) => `${b.key ?? "?"} ${b.label ?? "?"}`);
}

const driver = await Driver.launch({
  sandboxHome: true,
  binary: BINARY,
  sessionName: "style-rewrite-footer",
});

const receipt: Record<string, unknown> = { binary: BINARY };
let failures: string[] = [];

function expect(cond: boolean, label: string) {
  if (!cond) failures.push(label);
}

try {
  // 1. Baseline: empty filter keeps the Agent button.
  await driver.setFilterAndWait("");
  const empty = await driver.getState();
  receipt.emptyFilterButtons = summarize(empty);
  expect(
    footerButtons(empty).some((b) => b.label === "Agent"),
    "baseline: Agent button present with empty filter",
  );

  // 2. Style selector: `.` is a style-only parse.
  await driver.setFilterAndWait(".");
  const styleOnly = await driver.getState();
  receipt.styleOnlyButtons = summarize(styleOnly);
  receipt.styleOnlySelectedValue = styleOnly?.selectedValue ?? null;
  expect(
    footerButtons(styleOnly).some((b) => b.label === "Rewrite"),
    "style-only: Enter button labeled Rewrite",
  );
  expect(
    !footerButtons(styleOnly).some((b) => b.label === "Attach"),
    "style-only: no Attach label",
  );
  expect(
    !footerButtons(styleOnly).some((b) => b.label === "Agent"),
    "style-only: Agent button removed",
  );

  // Also with a typed style query like ".prof".
  await driver.setFilterAndWait(".prof");
  const styleQuery = await driver.getState();
  receipt.styleQueryButtons = summarize(styleQuery);
  expect(
    footerButtons(styleQuery).some((b) => b.label === "Rewrite"),
    "style query .prof: Enter button labeled Rewrite",
  );
  expect(
    !footerButtons(styleQuery).some((b) => b.label === "Agent"),
    "style query .prof: Agent button removed",
  );

  // 3. Composed prompt: style inside a larger prompt keeps Attach + Agent.
  await driver.setFilterAndWait("@clipboard .prof");
  const composed = await driver.getState();
  receipt.composedButtons = summarize(composed);
  expect(
    footerButtons(composed).some((b) => b.label === "Agent"),
    "composed prompt: Agent button restored",
  );
  expect(
    !footerButtons(composed).some((b) => b.label === "Rewrite"),
    "composed prompt: style rows fall back to Attach (no Rewrite)",
  );
} finally {
  await driver.close();
}

receipt.failures = failures;
receipt.classification = failures.length === 0 ? "green" : "red";
console.log(JSON.stringify(receipt, null, 2));
process.exit(failures.length === 0 ? 0 : 1);
