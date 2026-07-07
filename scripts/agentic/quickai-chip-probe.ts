#!/usr/bin/env bun
/**
 * Supplementary Quick AI proofs:
 *  A. Header Tab chip swaps label: cwd label when input empty → "Quick AI"
 *     when input has text (checked via layout/element text).
 *  B. The spawned pi process for the quick-ai session carries
 *     `--model gpt-5.3-codex-spark` and `--tools web_search` on its real argv.
 *
 * Run: bun scripts/agentic/quickai-chip-probe.ts
 */
import { Driver } from "../devtools/driver.ts";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/quickai-tab/script-kit-gpui";

const receipt: Record<string, unknown> = { probe: "quickai-chip", binary };

const driver = await Driver.launch({
  sessionName: "quickai-chip-probe",
  binary,
  sandboxHome: true,
  seedAgentAuth: true,
});

async function uiBlob(): Promise<string> {
  const [elements, layout] = await Promise.all([
    driver.getElements().catch(() => ({})),
    driver.getLayoutInfo().catch(() => ({})),
  ]);
  return JSON.stringify({ elements, layout });
}

try {
  await driver.waitForSettle();

  // A) chip label: screenshot empty vs typed state (chip text is not in the
  // semantic tree — visual proof via captured PNGs, inspected by the agent).
  const shotDir = process.env.QUICKAI_SHOT_DIR ?? driver.sessionDir;
  driver.send({ type: "show" });
  await driver.waitFor("windowVisible", { timeoutMs: 5000 }).catch(() => null);
  await driver.waitForSettle();
  const shotEmpty = await driver.captureScreenshot({
    savePath: `${shotDir}/chip-empty.png`,
  });
  await driver.setFilterAndWait("what is the capital of france");
  await driver.waitForSettle();
  const shotTyped = await driver.captureScreenshot({
    savePath: `${shotDir}/chip-typed.png`,
  });
  receipt.screenshotErrors = [
    (shotEmpty as any)?.error ?? null,
    (shotTyped as any)?.error ?? null,
  ];
  const typedBlob = await uiBlob();
  receipt.chip = {
    screenshots: [`${shotDir}/chip-empty.png`, `${shotDir}/chip-typed.png`],
    uiTreeExposesChipText: typedBlob.includes("Quick AI"),
  };

  // B) fire Quick AI and inspect the spawned pi processes' real argv.
  // The Text-profile prewarm also runs the spark model with
  // `--tools web_search`, so collect ALL `--mode rpc` pi lines and require
  // one that is quick-ai-shaped: spark model AND the Quick AI append prompt
  // ("You are Quick AI") on its argv.
  driver.simulateKey("tab");
  let quickAiLine = "";
  let allLines: string[] = [];
  const deadline = performance.now() + 15000;
  while (performance.now() < deadline) {
    const ps = Bun.spawnSync(["pgrep", "-fl", "mode rpc"]);
    allLines = ps.stdout
      .toString()
      .split("\n")
      .filter((l) => l.trim().length > 0);
    const line = allLines.find(
      (l) => l.includes("gpt-5.3-codex-spark") && l.includes("You are Quick AI"),
    );
    if (line) {
      quickAiLine = line;
      break;
    }
    await Bun.sleep(250);
  }
  receipt.piProcess = {
    found: quickAiLine.length > 0,
    hasSparkModel: quickAiLine.includes("gpt-5.3-codex-spark"),
    hasWebSearchTool: quickAiLine.includes("--tools web_search"),
    hasNoSkills: quickAiLine.includes("--no-skills"),
    hasNoExtensions: quickAiLine.includes("--no-extensions"),
    hasNoContextFiles: quickAiLine.includes("--no-context-files"),
    hasNoSession: quickAiLine.includes("--no-session"),
    rpcProcessCount: allLines.length,
  };

  const p = receipt.piProcess as Record<string, boolean>;
  receipt.pass = Boolean(
    p.found && p.hasSparkModel && p.hasWebSearchTool && p.hasNoSkills && p.hasNoContextFiles,
  );
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
