#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { Driver } from "../devtools/driver";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");

function argValue(name: string): string | null {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : null;
}

function usage(): string {
  return `Usage: bun scripts/agentic/root-search-frame-stability.ts --binary <path> --receipt <path> [options]

Required:
  --binary <path>             Stable script-kit-gpui artifact to launch
  --receipt <path>            JSON receipt output path

Options:
  --session <name>            Session label (uniquified per process/run)
  --query <text>              Root query (default: zzqxframeproof)
  --timeout <ms>              Protocol timeout (default: 10000)
  --poll <ms>                 Sample interval (default: 25)
  --inject-forbidden-shift    Deterministically fail the frame-identity gate
  -h, --help                  Show this help`;
}

if (process.argv.includes("--help") || process.argv.includes("-h")) {
  console.log(usage());
  process.exit(0);
}

const binaryArg = argValue("--binary");
const receiptArg = argValue("--receipt");
if (!binaryArg || !receiptArg) {
  console.error(usage());
  throw new Error("--binary and --receipt are required");
}

const binary = resolve(repoRoot, binaryArg);
const receiptPath = resolve(repoRoot, receiptArg);
const sessionLabel = argValue("--session") ?? "root-search-frame-stability";
const sessionName = `${sessionLabel}-${process.pid}-${Date.now()}`;
const query = argValue("--query") ?? "zzqxframeproof";
const timeoutMs = Number(argValue("--timeout") ?? "10000");
const pollMs = Number(argValue("--poll") ?? "25");
const injectForbiddenShift = process.argv.includes("--inject-forbidden-shift");
const fixtureResultPath = `/tmp/${query}-late-provider-result.txt`;

if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
  throw new Error(`--timeout must be a positive number, got ${JSON.stringify(timeoutMs)}`);
}
if (!Number.isFinite(pollMs) || pollMs <= 0) {
  throw new Error(`--poll must be a positive number, got ${JSON.stringify(pollMs)}`);
}

function git(args: string[]): string {
  const result = spawnSync("git", args, { cwd: repoRoot, encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed: ${result.stderr.trim()}`);
  }
  return result.stdout.trim();
}

function binarySha256(path: string): string {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function elementsFingerprint(elementsResult: Json): string {
  const elements = Array.isArray(elementsResult.elements) ? elementsResult.elements : [];
  return elements
    .filter((element: Json) => typeof element.semanticId === "string")
    .map((element: Json) =>
      [
        element.role ?? "",
        element.semanticId,
        element.text ?? "",
        element.index ?? "",
        element.action ?? "",
      ].join(":"),
    )
    .join("|");
}

function requirePreflight(state: Json, label: string): Json {
  const preflight = state.mainWindowPreflight;
  if (!preflight) {
    throw new Error(`${label}: missing mainWindowPreflight in getState receipt`);
  }
  for (const field of [
    "selectedResultKey",
    "selectedResultRole",
    "visibleResultKeyFingerprint",
    "visibleRowFingerprint",
    "visibleResultCount",
    "visibleResults",
    "enterAction",
  ]) {
    if (!(field in preflight)) {
      throw new Error(`${label}: mainWindowPreflight missing ${field}`);
    }
  }
  return preflight;
}

async function comparable(driver: Driver, state: Json, tag: string): Promise<Json> {
  const preflight = requirePreflight(state, tag);
  const elements = (await driver.getElements(
    { target: { type: "main" } },
    { timeoutMs },
  )) as Json;
  return {
    selectedIndex: preflight.selectedIndex,
    selectedResultKey: preflight.selectedResultKey ?? null,
    selectedResultRole: preflight.selectedResultRole,
    visibleResultKeyFingerprint: preflight.visibleResultKeyFingerprint,
    visibleRowFingerprint: preflight.visibleRowFingerprint,
    visibleResultCount: preflight.visibleResultCount,
    visibleResults: preflight.visibleResults,
    enterAction: preflight.enterAction,
    elementsFingerprint: elementsFingerprint(elements),
  };
}

function assertSameFrame(baseline: Json, sample: Json, label: string) {
  const before = JSON.stringify(baseline);
  const after = JSON.stringify(sample);
  if (before !== after) {
    throw new Error(
      `${label}: visible frame changed while provider resolved\nbefore=${before}\nafter=${after}`,
    );
  }
}

function numericField(value: unknown): number {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function hasWarmRootFileCache(status: Json): boolean {
  return numericField(status.cacheEntryCount) > 0 && numericField(status.cacheResultCount) > 0;
}

function requireRootFileStatus(state: Json, label: string): Json {
  const status = state.rootFileSearch;
  if (status?.query !== query) {
    throw new Error(
      `${label}: root file search did not track query ${JSON.stringify(query)}: ${JSON.stringify(status)}`,
    );
  }
  if (status?.mode !== "GlobalQuery") {
    throw new Error(`${label}: expected GlobalQuery root file mode, got ${JSON.stringify(status)}`);
  }
  return status;
}

function classifyRootFileBaseline(status: Json): Json {
  if (status.visibleLoading !== true) {
    throw new Error(`baseline is not an early visible loading frame: ${JSON.stringify(status)}`);
  }
  if (status.loading === true) {
    return { kind: "loading", observedLoading: true, observedAsyncHandoff: false };
  }
  if (
    status.loading === false &&
    status.visibleResultCount === 0 &&
    numericField(status.generation) >= 1 &&
    hasWarmRootFileCache(status)
  ) {
    return {
      kind: "settled-provider-early-visible-loading",
      observedLoading: false,
      observedAsyncHandoff: true,
      generation: status.generation,
      cacheEntryCount: status.cacheEntryCount,
      cacheResultCount: status.cacheResultCount,
      visibleResultCount: status.visibleResultCount,
    };
  }
  throw new Error(
    `unsupported root file baseline; expected loading frame or settled-provider early visible-loading frame: ${JSON.stringify(status)}`,
  );
}

async function sampleUntilRootFileSettled(
  driver: Driver,
  baseline: Json,
  baselineProof: Json,
  samples: Json[],
): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let observedLoading = baselineProof.observedLoading === true;
  let observedAsyncHandoff = baselineProof.observedAsyncHandoff === true;
  let settledStableSamples = 0;
  let injected = false;
  const requiredSettledStableSamples =
    baselineProof.kind === "settled-provider-early-visible-loading" ? 2 : 1;

  while (Date.now() < deadline) {
    const state = (await driver.getState({ timeoutMs })) as Json;
    const status = state.rootFileSearch;
    if (status?.query === query && status?.mode === "GlobalQuery") {
      if (status.loading === true) observedLoading = true;

      const observedFrame = await comparable(driver, state, `sample-${samples.length}`);
      const frame =
        injectForbiddenShift && !injected
          ? { ...observedFrame, visibleRowFingerprint: "__injected_forbidden_shift__" }
          : observedFrame;
      if (injectForbiddenShift && !injected) injected = true;
      samples.push({ rootFileSearch: status, frame, injectionApplied: frame !== observedFrame });
      assertSameFrame(baseline, frame, `samples[${samples.length - 1}]`);

      if (status.loading === false) {
        if (!hasWarmRootFileCache(status)) {
          throw new Error(`provider settled without warming cache; status=${JSON.stringify(status)}`);
        }
        observedAsyncHandoff = true;
        if (!observedLoading && baselineProof.kind !== "settled-provider-early-visible-loading") {
          throw new Error(
            `provider settled without an accepted async handoff proof; baselineProof=${JSON.stringify(
              baselineProof,
            )} status=${JSON.stringify(status)}`,
          );
        }
        settledStableSamples += 1;
        if (observedAsyncHandoff && settledStableSamples >= requiredSettledStableSamples) {
          return state;
        }
      }
    }
    await Bun.sleep(Math.max(25, pollMs));
  }
  throw new Error(`root file search did not settle for ${JSON.stringify(query)}`);
}

const receipt: Json = {
  schemaVersion: 3,
  gateId: "root-frame-stable",
  metricKind: "semantic_frame_identity",
  status: "fail",
  query,
  injectForbiddenShift,
  receiptPath,
  provenance: {
    binary,
    binarySha256: binarySha256(binary),
    gitSha: git(["rev-parse", "HEAD"]),
    sourceDirty: git(["status", "--porcelain"]).length > 0,
  },
  session: { name: sessionName, directory: null },
  samples: [],
};

let driver: Driver | null = null;
try {
  driver = await Driver.launch({
    binary,
    sandboxHome: true,
    sessionName,
    readyTimeoutMs: timeoutMs,
    defaultTimeoutMs: timeoutMs,
    env: {
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_STARTUP_PROFILE: "dev-fast",
      SCRIPT_KIT_STARTUP_READY_LOG: "1",
      SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM: "1",
      SCRIPT_KIT_DISABLE_AUTOMATIC_UPDATE_CHECK: "1",
      SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER: JSON.stringify({
        query,
        delayMs: 250,
        results: [
          {
            path: fixtureResultPath,
            name: `${query}-late-provider-result.txt`,
            fileType: "document",
            size: 42,
            modified: 1,
          },
        ],
      }),
    },
  });
  receipt.session.directory = driver.sessionDir;
  await driver.setFilterAndWait(query, { timeoutMs });

  const before = (await driver.getState({ timeoutMs })) as Json;
  const beforeRootFileSearch = requireRootFileStatus(before, "before");
  const baselineProof = classifyRootFileBaseline(beforeRootFileSearch);
  const baseline = await comparable(driver, before, "before");
  receipt.baselineProof = baselineProof;
  receipt.baseline = {
    inputValue: before.inputValue,
    rootFileSearch: beforeRootFileSearch,
    mainWindowPreflight: baseline,
  };

  const settled = await sampleUntilRootFileSettled(
    driver,
    baseline,
    baselineProof,
    receipt.samples,
  );
  const settledFrame = await comparable(driver, settled, "settled");
  assertSameFrame(baseline, settledFrame, "settled");
  receipt.settled = {
    inputValue: settled.inputValue,
    rootFileSearch: settled.rootFileSearch,
    mainWindowPreflight: settledFrame,
  };
  receipt.status = "pass";
} catch (error) {
  receipt.failure = error instanceof Error ? error.message : String(error);
} finally {
  receipt.cleanup = { attempted: driver !== null, closed: false, error: null };
  if (driver) {
    try {
      await driver.close();
      receipt.cleanup.closed = true;
    } catch (error) {
      const cleanupError = error instanceof Error ? error.message : String(error);
      receipt.cleanup.error = cleanupError;
      receipt.status = "fail";
      receipt.failure = receipt.failure
        ? `${receipt.failure}; cleanup: ${cleanupError}`
        : `cleanup: ${cleanupError}`;
    }
  }
  mkdirSync(dirname(receiptPath), { recursive: true });
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
}

console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.status === "pass" ? 0 : 1);
