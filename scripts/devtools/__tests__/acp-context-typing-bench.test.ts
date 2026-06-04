import { describe, expect, test } from "bun:test";
import { classifyBenchReceipt, summarizeBenchSteps } from "../acp-mention";

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
    target: { type: "id", id: "ai" },
  };
}

function scenario(name: string, p95: number, target: unknown = { type: "id", id: "ai" }) {
  const steps = [step(p95 - 2), step(p95 - 1), step(p95)];
  return {
    name,
    targetKind: name.startsWith("acp") ? "acp" as const : "main" as const,
    target,
    steps,
    summary: summarizeBenchSteps(steps),
  };
}

describe("acp context typing benchmark receipt", () => {
  test("summarizes p95, missing timings, and failed steps", () => {
    const summary = summarizeBenchSteps([step(4), step(null), step(40, false)]);

    expect(summary.p50Ms).toBe(4);
    expect(summary.p95Ms).toBe(40);
    expect(summary.maxMs).toBe(40);
    expect(summary.over16Count).toBe(1);
    expect(summary.over32Count).toBe(1);
    expect(summary.missingTimingCount).toBe(1);
    expect(summary.missingTraceTimingCount).toBe(1);
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

  test("classifies slow ACP context typing as reproduced relative to main menu", () => {
    const classification = classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8),
      scenario("main-menu-spine-at-baseline", 9, { type: "id", id: "main" }),
      scenario("acp-context-root", 28),
      scenario("acp-file-subsearch", 12),
      scenario("acp-clipboard-subsearch", 12),
    ]);

    expect(classification).toBe("reproduced");
  });

  test("classifies matched ACP and main-menu timings as fixed", () => {
    const classification = classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      scenario("acp-context-root", 9),
      scenario("acp-file-subsearch", 10),
      scenario("acp-clipboard-subsearch", 10),
    ]);

    expect(classification).toBe("fixed");
  });

  test("fails closed when a target or timing is missing", () => {
    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      scenario("acp-context-root", 9, null),
      scenario("acp-file-subsearch", 10),
      scenario("acp-clipboard-subsearch", 10),
    ])).toBe("blocked-by-target-ambiguity");

    const missingTiming = {
      ...scenario("acp-context-root", 9),
      steps: [step(null)],
      summary: summarizeBenchSteps([step(null)]),
    };
    expect(classifyBenchReceipt([
      scenario("main-menu-search-baseline", 8, { type: "id", id: "main" }),
      scenario("main-menu-spine-at-baseline", 8, { type: "id", id: "main" }),
      missingTiming,
      scenario("acp-file-subsearch", 10),
      scenario("acp-clipboard-subsearch", 10),
    ])).toBe("blocked-by-missing-primitive");
  });
});
