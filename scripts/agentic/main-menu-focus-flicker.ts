#!/usr/bin/env bun

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const repoRoot = resolve(import.meta.dir, "../..");
const outArgIndex = process.argv.indexOf("--out");
const durationArgIndex = process.argv.indexOf("--duration-ms");
const outDir =
  outArgIndex >= 0 && process.argv[outArgIndex + 1]
    ? resolve(process.argv[outArgIndex + 1])
    : join(repoRoot, ".test-output", "main-menu-focus-flicker");
const durationMs =
  durationArgIndex >= 0 && process.argv[durationArgIndex + 1]
    ? Number(process.argv[durationArgIndex + 1])
    : 160;

const homeDir = join(outDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const scriptsDir = join(kitDir, "plugins", "main", "scripts");

function seedFixtures() {
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(scriptsDir, { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    files: { enabled: false, globalSearch: false, recentFiles: false, directoryBrowse: false },
    notes: { enabled: false },
    clipboardHistory: { enabled: false },
    dictationHistory: { enabled: false },
    agent_chatHistory: { enabled: false },
    aiVault: { enabled: false },
    browserTabs: { enabled: false },
    browserHistory: { enabled: false },
  },
};
`,
  );

  for (const [name, description] of [
    ["flicker-alpha", "Flicker Alpha description proves selected rows expose details."],
    ["flicker-beta", "Flicker Beta description proves selected rows expose details."],
    ["flicker-gamma", "Flicker Gamma description proves selected rows expose details."],
  ]) {
    writeFileSync(
      join(scriptsDir, `${name}.ts`),
      `// Name: ${name.replace("-", " ")}
// Description: ${description}
console.log(${JSON.stringify(name)});
`,
    );
  }
}

function elementsFromReceipt(receipt: Json): Json[] {
  const candidates = [receipt.elements, receipt.nodes, receipt.elementSnapshot?.nodes];
  for (const candidate of candidates) {
    if (Array.isArray(candidate)) return candidate as Json[];
  }
  return [];
}

function selectedChoices(elements: Json[]): Json[] {
  return elements.filter(
    (element) =>
      element?.elementType === "choice" ||
      element?.type === "choice" ||
      element?.role === "row",
  ).filter((element) => element?.selected === true || element?.selected === "true");
}

async function sample(driver: Driver, label: string, t0: number): Promise<Json> {
  const [state, elementsReceipt] = await Promise.all([
    driver.getState({ timeoutMs: 5000 }),
    driver.getElements({ limit: 40 }, { timeoutMs: 5000 }),
  ]);
  const elements = elementsFromReceipt(elementsReceipt);
  const selected = selectedChoices(elements);
  return {
    label,
    tMs: Math.round(performance.now() - t0),
    inputValue: state.inputValue ?? null,
    promptType: state.promptType ?? null,
    selectedIndex: state.selectedIndex ?? null,
    visibleChoiceCount: state.visibleChoiceCount ?? null,
    selectedChoices: selected.map((choice) => ({
      semanticId: choice.semanticId ?? choice.semantic_id ?? null,
      text: choice.text ?? null,
      value: choice.value ?? null,
      index: choice.index ?? null,
    })),
  };
}

function assertStable(samples: Json[]) {
  const failures: Json[] = [];
  for (const sample of samples) {
    if (sample.inputValue !== "flicker") continue;
    if (Number(sample.visibleChoiceCount ?? 0) <= 0) continue;
    if (sample.selectedChoices.length !== 1) {
      failures.push({ reason: "expected exactly one selected choice", sample });
      continue;
    }
    const selected = sample.selectedChoices[0];
    if (typeof selected.value !== "string" || !selected.value.includes("description proves")) {
      failures.push({ reason: "selected choice did not expose description value", sample });
    }
  }
  if (failures.length > 0) {
    throw new Error(`main menu focus flicker samples failed: ${JSON.stringify(failures)}`);
  }
}

async function main() {
  seedFixtures();
  const binary =
    process.env.SCRIPT_KIT_GPUI_BINARY ??
    join(repoRoot, "target-agent", "artifacts", "main-menu-focus-flicker", "script-kit-gpui");
  const driver = await Driver.launch({
    binary,
    sessionName: "main-menu-focus-flicker",
    sessionDir: join(outDir, "driver"),
    sandboxHome: false,
    env: {
      HOME: homeDir,
      SK_PATH: kitDir,
      SCRIPT_KIT_AGENTIC_RUST_LOG:
        "info,script_kit::selection=debug,script_kit::scroll=debug,gpui=warn",
    },
    readyTimeoutMs: 15_000,
    defaultTimeoutMs: 5_000,
  });

  const samples: Json[] = [];
  try {
    await driver.setFilterAndWait("");
    await driver.waitForState({ promptType: "scriptList" }, { timeoutMs: 5000 });
    await driver.setFilterAndWait("gamma");
    await Bun.sleep(50);
    driver.simulateKey("down");
    await Bun.sleep(50);

    const t0 = performance.now();
    const setFilter = driver.setFilterAndWait("flicker", { timeoutMs: 5000 });
    while (performance.now() - t0 < durationMs) {
      samples.push(await sample(driver, "post-replacement", t0));
      await Bun.sleep(8);
    }
    await setFilter;
    samples.push(await sample(driver, "settled", t0));
    assertStable(samples);

    const receipt = {
      status: "pass",
      durationMs,
      binary,
      outDir,
      sessionDir: driver.sessionDir,
      logPath: driver.logPath,
      samples,
      stats: driver.stats,
    };
    writeFileSync(join(outDir, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
    console.log(JSON.stringify(receipt, null, 2));
  } finally {
    await driver.close();
  }
}

await main();
