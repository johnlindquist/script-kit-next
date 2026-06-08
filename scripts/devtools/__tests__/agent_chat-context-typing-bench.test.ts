import { describe, expect, test } from "bun:test";
import { classifyBenchReceipt, summarizeBenchSteps } from "../agent_chat-mention";

function step(ms: number | null, success = true) {
  return {
    label: "set @",
    input: "@",
    success,
    traceTotalElapsedMs: ms,
    commandElapsedMs: ms,
    wallMs: ms ?? 0,
    effectiveElapsedMs: ms,
    inputTextAfter: "@",
    visibleCountAfter: 1,
    spine: {
      ownsList: true,
      activeSegmentKind: "contextMention",
      rowCount: 3,
      selectableRowCount: 3,
      selectedIndex: 0,
      rowFingerprint: "fnv1a64:0000000000000001",
      selectedRowFingerprint: "fnv1a64:0000000000000002",
      refreshElapsedMs: 1,
    },
    spineProof: { required: true, ok: true },
    target: { type: "id", id: "ai" },
  };
}

function scenario(name: string, p95: number, target: unknown = { type: "id", id: "ai" }) {
  const targetKind = name.startsWith("agent_chat") ? "agent_chat" as const : "main" as const;
  const steps = [step(p95 - 2), step(p95 - 1), step(p95)].map((entry) => {
    if (targetKind === "main") {
      return {
        ...entry,
        spine: null,
        spineProof: { required: false, ok: true },
      };
    }
    if (name === "agent_chat-file-subsearch") {
      return { ...entry, spine: { ...entry.spine, subsearchSource: "file" } };
    }
    if (name === "agent_chat-clipboard-subsearch") {
      return { ...entry, spine: { ...entry.spine, subsearchSource: "clipboard" } };
    }
    return entry;
  });
  return {
    name,
    targetKind,
    target,
    steps,
    summary: summarizeBenchSteps(steps),
  };
}

describe("agent_chat context typing benchmark receipt", () => {
  test("summarizes p95, missing timings, and failed steps", () => {
    const summary = summarizeBenchSteps([step(4), step(null), step(40, false)]);

    expect(summary.p50Ms).toBe(4);
    expect(summary.p95Ms).toBe(40);
    expect(summary.maxMs).toBe(40);
    expect(summary.over16Count).toBe(1);
    expect(summary.over32Count).toBe(1);
    expect(summary.missingTimingCount).toBe(1);
    expect(summary.missingTraceTimingCount).toBe(1);
    expect(summary.missingSpineCount).toBe(0);
    expect(summary.failedSpineProofCount).toBe(0);
    expect(summary.failedStepCount).toBe(1);
  });

  test("uses wall time when transaction timings are rounded to zero", () => {
    const summary = summarizeBenchSteps([{
      ...step(0),
      wallMs: 120,
      effectiveElapsedMs: 120,
    }]);

    expect(summary.p95Ms).toBe(120);
    expect(summary.over32Count).toBe(1);
  });

  test("uses Agent Chat spine row count as visible count after input", () => {
    const entry = {
      ...step(8),
      visibleCountAfter: 3,
    };

    expect(entry.visibleCountAfter).toBe(entry.spine?.rowCount);
  });

  test("classifies slow Agent Chat context typing as reproduced relative to main menu", () => {
    const classification = classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8),
      scenario("main-menu-spine-at-baseline", 9, { type: "id", id: "main" }),
      scenario("agent_chat-context-root", 28),
      scenario("agent_chat-file-subsearch", 12),
      scenario("agent_chat-clipboard-subsearch", 12),
    ]);

    expect(classification).toBe("reproduced");
  });

  test("classifies matched Agent Chat and main-menu timings as fixed", () => {
    const classification = classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      scenario("agent_chat-context-root", 9),
      scenario("agent_chat-file-subsearch", 10),
      scenario("agent_chat-clipboard-subsearch", 10),
    ]);

    expect(classification).toBe("fixed");
  });

  test("fails closed when a target or timing is missing", () => {
    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      scenario("agent_chat-context-root", 9, null),
      scenario("agent_chat-file-subsearch", 10),
      scenario("agent_chat-clipboard-subsearch", 10),
    ])).toBe("blocked-by-target-ambiguity");

    const missingTiming = {
      ...scenario("agent_chat-context-root", 9),
      steps: [step(null)],
      summary: summarizeBenchSteps([step(null)]),
    };
    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      missingTiming,
      scenario("agent_chat-file-subsearch", 10),
      scenario("agent_chat-clipboard-subsearch", 10),
    ])).toBe("blocked-by-missing-primitive");
  });

  test("fails closed when Agent Chat spine receipt is missing for @ input", () => {
    const missingSpine = {
      ...scenario("agent_chat-context-root", 9),
      steps: [{ ...step(9), spine: null, spineProof: { required: true, ok: false, reason: "missing-spine" } }],
    };
    missingSpine.summary = summarizeBenchSteps(missingSpine.steps);

    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      missingSpine,
      scenario("agent_chat-file-subsearch", 10),
      scenario("agent_chat-clipboard-subsearch", 10),
    ])).toBe("blocked-by-missing-primitive");
  });

  test("requires subsearch source for Agent Chat file and clipboard scenarios", () => {
    const badFile = scenario("agent_chat-file-subsearch", 10);
    badFile.steps = badFile.steps.map((entry) => ({
      ...entry,
      spine: entry.spine ? { ...entry.spine, subsearchSource: undefined } : null,
      spineProof: { required: true, ok: false, reason: "missing-file-subsearch-source" },
    }));
    badFile.summary = summarizeBenchSteps(badFile.steps);

    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      scenario("agent_chat-context-root", 9),
      badFile,
      scenario("agent_chat-clipboard-subsearch", 10),
    ])).toBe("blocked-by-missing-primitive");

    const badClipboard = scenario("agent_chat-clipboard-subsearch", 10);
    badClipboard.steps = badClipboard.steps.map((entry) => ({
      ...entry,
      spine: entry.spine ? { ...entry.spine, subsearchSource: undefined } : null,
      spineProof: { required: true, ok: false, reason: "missing-clipboard-subsearch-source" },
    }));
    badClipboard.summary = summarizeBenchSteps(badClipboard.steps);

    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      scenario("agent_chat-context-root", 9),
      scenario("agent_chat-file-subsearch", 10),
      badClipboard,
    ])).toBe("blocked-by-missing-primitive");
  });
});
