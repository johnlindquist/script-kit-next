import { describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { portScript } from "../pipeline.ts";

const V1 = join(import.meta.dir, "fixtures", "v1");

async function portHelloWorld(noExec: boolean) {
  const outDir = mkdtempSync(join(tmpdir(), "sk-migrate-pipeline-test-"));
  try {
    return await portScript(join(V1, "hello-world.ts"), {
      outDir,
      dryRun: true,
      noExec,
    });
  } finally {
    rmSync(outDir, { recursive: true, force: true });
  }
}

describe("ready-script pipeline receipts", () => {
  test("no-exec dry run records skipped validation and a truthful verbatim note", async () => {
    const result = await portHelloWorld(true);

    expect(result.status).toBe("verified-with-warnings");
    expect(result.agentUsed).toBe(false);
    expect(result.portedPath).toBeUndefined();
    expect(result.attempts).toHaveLength(1);
    expect(result.attempts[0].verdicts).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ id: "smoke", outcome: "skipped" }),
        expect.objectContaining({ id: "walkthrough", outcome: "skipped" }),
      ]),
    );
    expect(result.note).toEqual({
      summary: "Copied verbatim; no migration changes were required.",
      behavior_changes: [],
      confidence: "medium",
    });
    expect(result.attempts[0].note).toEqual(result.note);
  }, 120_000);

  test("fully executed ready script earns a high-confidence verified note", async () => {
    const result = await portHelloWorld(false);

    expect(result.status).toBe("verified");
    expect(
      result.attempts[0].verdicts.every((verdict) => verdict.outcome === "pass"),
    ).toBe(true);
    expect(result.note).toEqual({
      summary: "Copied verbatim; no migration changes were required.",
      behavior_changes: [],
      confidence: "high",
    });
    expect(result.attempts[0].note).toEqual(result.note);
  }, 120_000);
});
