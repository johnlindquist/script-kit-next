import { describe, expect, test } from "bun:test";
import { join } from "node:path";
import { apiScan, metadataCheck, smoke, typecheck, walkthrough } from "../validators.ts";
import { extractBlock, parseJsonBlock } from "../agent.ts";

const PORTED = join(import.meta.dir, "fixtures", "ported");
const V1 = join(import.meta.dir, "fixtures", "v1");

async function read(dir: string, name: string): Promise<string> {
  return Bun.file(join(dir, name)).text();
}

describe("api-scan validator", () => {
  test("passes a clean v2 script", async () => {
    const v = apiScan(await read(PORTED, "hello-world-v2.ts"));
    expect(v.outcome).toBe("pass");
  });

  test("fails a port that still calls db()", async () => {
    const v = apiScan(await read(PORTED, "bad-still-v1.ts"));
    expect(v.outcome).toBe("fail");
    expect(v.detail).toContain("db is removed");
  });

  test("caveat APIs warn instead of failing", () => {
    const v = apiScan('await arg("x");\nawait menu("icon");\n');
    expect(v.outcome).toBe("warn");
    expect(v.summary).toContain("menu");
  });
});

describe("metadata validator", () => {
  test("passes when everything survives", async () => {
    const orig = await read(V1, "hello-world.ts");
    const ported = await read(PORTED, "hello-world-v2.ts");
    expect(metadataCheck(orig, ported).outcome).toBe("pass");
  });

  test("fails when the shortcut is dropped", async () => {
    const orig = await read(V1, "hello-world.ts");
    const v = metadataCheck(orig, "// Name: Hello World\nawait arg('x');\n");
    expect(v.outcome).toBe("fail");
    expect(v.detail).toContain("shortcut");
  });
});

describe("typecheck validator (runs real tsc against the v2 SDK)", () => {
  test("clean v2 script typechecks", async () => {
    const v = await typecheck(join(PORTED, "hello-world-v2.ts"));
    expect(v.outcome).toBe("pass");
  }, 120_000);

  test("script using an undeclared v1 global fails with a TS diagnostic", async () => {
    const v = await typecheck(join(PORTED, "bad-still-v1.ts"));
    expect(v.outcome).toBe("fail");
    expect(v.detail).toContain("db");
  }, 120_000);
});

describe("smoke + walkthrough validators (run real bun with the v2 SDK preloaded)", () => {
  test("prompt script emits a first protocol message", async () => {
    const v = await smoke(join(PORTED, "hello-world-v2.ts"));
    expect(v.outcome).toBe("pass");
    expect(v.summary).toContain('"arg"');
  }, 20_000);

  test("side-effect-only script passes smoke via clean exit", async () => {
    const v = await smoke(join(PORTED, "no-prompt-v2.ts"));
    expect(v.outcome).toBe("pass");
    expect(v.summary).toContain("completion");
  }, 20_000);

  test("script crashing before its first prompt fails smoke with the real error", async () => {
    const v = await smoke(join(PORTED, "bad-crashes-early.ts"));
    expect(v.outcome).toBe("fail");
    expect(v.detail?.toLowerCase()).toContain("db");
  }, 20_000);

  test("script crashing AFTER its first prompt passes smoke but fails walkthrough", async () => {
    // smoke stops at the first protocol message, so the later db() crash is
    // invisible to it — the auto-submit walkthrough is the validator that
    // must catch it.
    const smokeVerdict = await smoke(join(PORTED, "bad-still-v1.ts"));
    expect(smokeVerdict.outcome).toBe("pass");
    const walkVerdict = await walkthrough(join(PORTED, "bad-still-v1.ts"));
    expect(walkVerdict.outcome).toBe("fail");
    expect(walkVerdict.detail?.toLowerCase()).toContain("db");
  }, 60_000);

  test("auto-submit walkthrough completes the whole prompt flow", async () => {
    const v = await walkthrough(join(PORTED, "hello-world-v2.ts"));
    expect(v.outcome).toBe("pass");
  }, 30_000);
});

describe("agent output contract parsing", () => {
  const sample = [
    "some preamble the model should not have written",
    "===PORTED_SCRIPT===",
    '// Name: X',
    'await arg("hi");',
    "===END_PORTED_SCRIPT===",
    "===MIGRATION_NOTE===",
    '{"summary":"s","behavior_changes":["a"],"confidence":"high"}',
    "===END_MIGRATION_NOTE===",
  ].join("\n");

  test("extracts the script block", () => {
    const block = extractBlock(sample, "PORTED_SCRIPT");
    expect(block).toContain('await arg("hi");');
    expect(block).not.toContain("preamble");
  });

  test("unwraps an accidental code fence", () => {
    const fenced = "===PORTED_SCRIPT===\n```ts\nawait arg('x');\n```\n===END_PORTED_SCRIPT===";
    expect(extractBlock(fenced, "PORTED_SCRIPT")).toBe("await arg('x');\n");
  });

  test("parses the migration note JSON", () => {
    const note = parseJsonBlock<{ summary: string }>(sample, "MIGRATION_NOTE");
    expect(note?.summary).toBe("s");
  });

  test("missing block returns null instead of throwing", () => {
    expect(extractBlock("no blocks here", "PORTED_SCRIPT")).toBeNull();
  });
});
