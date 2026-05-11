#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

const session = argValue("--session", "root-search-frame-stability");
const query = argValue("--query", "zzqxframeproof");
const timeoutMs = Number(argValue("--timeout", "10000"));
const pollMs = Number(argValue("--poll", "25"));

const fixtureResultPath = `/tmp/${query}-late-provider-result.txt`;
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
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
});

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(
      `session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`,
    );
  }
  let parsed: Json;
  try {
    parsed = JSON.parse(stdout);
  } catch (error) {
    throw new Error(`invalid JSON from session.sh: ${stdout}\n${String(error)}`);
  }
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(
      `session.sh ${args.join(" ")} failed: ${JSON.stringify(parsed)} stderr=${result.stderr.trim()}`,
    );
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  const envelope = runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]);
  if (envelope.status !== "ok") {
    throw new Error(`RPC failed: ${JSON.stringify(envelope)}`);
  }
  return envelope.response;
}

function send(command: Json): Json {
  return runSession([
    "send",
    session,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
}

function getState(tag: string): Json {
  const state = rpc(
    {
      type: "getState",
      requestId: `root-frame-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function getElements(tag: string): Json {
  const elements = rpc(
    {
      type: "getElements",
      requestId: `root-frame-elements-${tag}-${Date.now()}`,
    },
    "elementsResult",
  );
  if (elements.type !== "elementsResult") {
    throw new Error(`expected elementsResult, got ${JSON.stringify(elements)}`);
  }
  return elements;
}

function waitForInput(): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `root-frame-wait-input-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: query,
        },
      },
      timeout: timeoutMs,
      pollInterval: Math.max(25, Math.min(pollMs, 250)),
    },
    "waitForResult",
  );
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

function comparable(state: Json, tag: string): Json {
  const preflight = requirePreflight(state, tag);
  return {
    selectedIndex: preflight.selectedIndex,
    selectedResultKey: preflight.selectedResultKey ?? null,
    selectedResultRole: preflight.selectedResultRole,
    visibleResultKeyFingerprint: preflight.visibleResultKeyFingerprint,
    visibleRowFingerprint: preflight.visibleRowFingerprint,
    visibleResultCount: preflight.visibleResultCount,
    visibleResults: preflight.visibleResults,
    enterAction: preflight.enterAction,
    elementsFingerprint: elementsFingerprint(getElements(tag)),
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

async function sampleUntilRootFileSettled(
  baseline: Json,
  observedLoadingAtBaseline: boolean,
): Promise<{
  settled: Json;
  samples: Json[];
}> {
  const deadline = Date.now() + timeoutMs;
  const samples: Json[] = [];
  let observedLoading = observedLoadingAtBaseline;
  let last = getState("sample-start");

  while (Date.now() < deadline) {
    const status = last.rootFileSearch;
    if (status?.query === query && status?.mode === "GlobalQuery") {
      if (status.loading === true) {
        observedLoading = true;
      }

      const frame = comparable(last, `sample-${samples.length}`);
      samples.push({
        rootFileSearch: status,
        frame,
      });
      assertSameFrame(baseline, frame, `samples[${samples.length - 1}]`);

      if (status.loading === false) {
        if (observedLoading !== true) {
          throw new Error(
            `loading !== true before settle; status=${JSON.stringify(status)}`,
          );
        }
        if ((status.cacheResultCount ?? 0) <= 0) {
          throw new Error(
            `provider settled without warming cache; status=${JSON.stringify(status)}`,
          );
        }
        return { settled: last, samples };
      }
    }

    await Bun.sleep(Math.max(25, pollMs));
    last = getState(`sample-poll-${samples.length}`);
  }

  throw new Error(
    `root file search did not settle for ${JSON.stringify(query)}; last=${JSON.stringify(
      last.rootFileSearch,
    )}`,
  );
}

async function main() {
  if (!existsSync(sessionScript)) {
    throw new Error(`missing ${sessionScript}`);
  }

  runSession(["stop", session]);
  runSession(["start", session]);
  send({
    type: "setFilter",
    text: query,
    requestId: `root-frame-set-${Date.now()}`,
  });
  waitForInput();

  const before = getState("before");
  if (before.rootFileSearch?.query !== query) {
    throw new Error(
      `root file search did not track query ${JSON.stringify(query)}: ${JSON.stringify(
        before.rootFileSearch,
      )}`,
    );
  }
  if (before.rootFileSearch?.mode !== "GlobalQuery") {
    throw new Error(
      `expected GlobalQuery root file mode, got ${JSON.stringify(before.rootFileSearch)}`,
    );
  }
  if (before.rootFileSearch?.loading !== true) {
    throw new Error(
      `loading !== true for delayed provider baseline: ${JSON.stringify(before.rootFileSearch)}`,
    );
  }

  const baseline = comparable(before, "before");
  const { settled, samples } = await sampleUntilRootFileSettled(baseline, true);
  const settledFrame = comparable(settled, "settled");
  assertSameFrame(baseline, settledFrame, "settled");

  const receipt = {
    schemaVersion: 2,
    status: "pass",
    session,
    query,
    baseline: {
      inputValue: before.inputValue,
      rootFileSearch: before.rootFileSearch,
      mainWindowPreflight: baseline,
    },
    settled: {
      inputValue: settled.inputValue,
      rootFileSearch: settled.rootFileSearch,
      mainWindowPreflight: settledFrame,
    },
    samples,
  };

  process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
