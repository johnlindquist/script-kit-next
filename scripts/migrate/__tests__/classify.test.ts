import { describe, expect, test } from "bun:test";
import { join } from "node:path";
import { classify, formatFindings, stripComments } from "../classify.ts";
import {
  extractCommentMetadata,
  extractEffectiveMetadata,
  extractTypedMetadata,
  metadataLosses,
} from "../metadata.ts";

const FIXTURES = join(import.meta.dir, "fixtures", "v1");

async function fixture(name: string): Promise<string> {
  return Bun.file(join(FIXTURES, name)).text();
}

describe("classify buckets", () => {
  test("clean prompt-only script is ready", async () => {
    const c = classify(await fixture("hello-world.ts"));
    expect(c.bucket).toBe("ready");
    expect(c.hasKitImport).toBe(false);
    expect(c.findings.filter((f) => f.status !== "supported")).toEqual([]);
  });

  test("db/get/toast script needs rewrite and flags the kit import", async () => {
    const c = classify(await fixture("save-note-db.ts"));
    expect(c.bucket).toBe("needs-rewrite");
    expect(c.hasKitImport).toBe(true);
    const apis = c.findings.map((f) => f.api);
    expect(apis).toContain("db");
    expect(apis).toContain("get");
    expect(apis).toContain("toast");
  });

  test("widget + keyboard.type + registerShortcut needs rewrite", async () => {
    const c = classify(await fixture("widget-dashboard.ts"));
    expect(c.bucket).toBe("needs-rewrite");
    const byApi = Object.fromEntries(c.findings.map((f) => [f.api, f]));
    expect(byApi["widget"].status).toBe("stub");
    expect(byApi["keyboard.type"].status).toBe("stub");
    expect(byApi["registerShortcut"].status).toBe("removed");
  });

  test("renamed-only script is needs-changes at worst when no removed APIs", () => {
    const c = classify(`// Name: t\nawait textarea("x");\nawait edit("/tmp/f");\n`);
    expect(c.bucket).toBe("needs-changes");
    expect(c.findings.every((f) => f.status === "renamed")).toBe(true);
  });
});

describe("scanner precision", () => {
  test("APIs mentioned only in comments are not flagged", async () => {
    const c = classify(await fixture("renamed-apis.ts"));
    expect(c.findings.map((f) => f.api)).not.toContain("db");
  });

  test("property access does not match bare globals (map.get is not axios get)", () => {
    const c = classify(`const m = new Map();\nm.get("k");\nconst r = await fetch(u);\n`);
    expect(c.findings.map((f) => f.api)).not.toContain("get");
  });

  test("$ tagged template is detected as the removed zx-style global", () => {
    const c = classify("const out = await $`ls -la`;\n");
    expect(c.findings.map((f) => f.api)).toContain("$");
  });

  test("Bun.$ property access is the v2 remedy, not the removed $ global", () => {
    const c = classify("const out = await Bun.$`ls -la`.text();\n");
    expect(c.findings.map((f) => f.api)).not.toContain("$");
  });

  test("explicit imports shadow same-named v1 globals (the compat map's own remedies)", () => {
    const bunDollar = classify('import { $ } from "bun";\nconst out = await $`ls`.text();\n');
    expect(bunDollar.findings.map((f) => f.api)).not.toContain("$");
    const dateFns = classify('import { formatDate } from "date-fns";\nformatDate(new Date());\n');
    expect(dateFns.findings.map((f) => f.api)).not.toContain("formatDate");
  });

  test("env() with a v1 options object is NOT flagged (SDK is v1-compatible)", () => {
    const withOpts = classify('const k = await env("API_KEY", { secret: true, hint: "sk_..." });\n');
    expect(withOpts.findings.every((f) => f.status === "supported")).toBe(true);
  });

  test("aliased import does NOT shadow the original name", () => {
    const c = classify('import { $ as sh } from "bun";\nawait $`ls`;\n');
    expect(c.findings.map((f) => f.api)).toContain("$");
  });

  test("unknown keyboard method still flags as stub via the root fallback", () => {
    const c = classify("await keyboard.config({ autoDelayMs: 0 });\n");
    const f = c.findings.find((x) => x.api === "keyboard.config");
    expect(f?.status).toBe("stub");
  });

  test("stripComments preserves line numbers", () => {
    const stripped = stripComments("// one\n/* two\nthree */\ncode()\n");
    expect(stripped.split("\n").length).toBe(5);
    expect(stripped).toContain("code()");
    expect(stripped).not.toContain("one");
  });

  test("finding lines point at the real source line", async () => {
    const source = await fixture("save-note-db.ts");
    const c = classify(source);
    const db = c.findings.find((f) => f.api === "db")!;
    expect(source.split("\n")[db.line - 1]).toContain("db(");
  });
});

describe("call-shape mismatches (v1↔v2 parity audit class)", () => {
  test("node path methods on the global crash in v2 and are flagged", () => {
    const c = classify('const p = path.join(home(), "notes");\nawait path();\n');
    const f = c.findings.find((x) => x.api === "path.<node-method>");
    expect(f?.status).toBe("stub");
    expect(c.bucket).toBe("needs-rewrite");
  });

  test("bare path() picker is not flagged", () => {
    const c = classify("const file = await path();\n");
    expect(c.findings.filter((f) => f.status !== "supported")).toEqual([]);
  });

  test("editor config object and helper methods are flagged", () => {
    const c = classify('await editor({ value: "x", language: "md" });\neditor.append("y");\n');
    const apis = c.findings.map((f) => f.api);
    expect(apis).toContain("editor({config})");
    expect(apis).toContain("editor.<method>");
  });

  test("editor with plain string is not flagged", () => {
    const c = classify('await editor("draft", "markdown");\n');
    expect(c.findings.filter((f) => f.status !== "supported")).toEqual([]);
  });

  test("arg config object warns but does not force needs-rewrite", () => {
    const c = classify('await arg({ placeholder: "x", hint: "h" });\n');
    const f = c.findings.find((x) => x.api === "arg({config})");
    expect(f?.status).toBe("caveat");
    expect(c.bucket).toBe("needs-changes");
  });

  test("missing clipboard methods are stubs; supported ones are not", () => {
    const c = classify('await clipboard.writeHTML("<b>x</b>");\nawait clipboard.writeText("x");\n');
    const stub = c.findings.find((f) => f.api === "clipboard.writeHTML");
    expect(stub?.status).toBe("stub");
    expect(c.findings.find((f) => f.api === "clipboard.writeText")).toBeUndefined();
  });

  test("memoryMap Map-only methods are flagged; get/set are not", () => {
    const c = classify('memoryMap.set("a", 1);\nif (memoryMap.has("a")) exit();\n');
    const f = c.findings.find((x) => x.api === "memoryMap.<Map-method>");
    expect(f?.status).toBe("stub");
  });

  test("compile and notify-with-options carry caveats", () => {
    const c = classify('const t = compile("{{#each xs}}{{.}}{{/each}}");\nnotify({ title: "t", sound: true });\n');
    expect(c.findings.find((x) => x.api === "compile(template)")?.status).toBe("caveat");
    expect(c.findings.find((x) => x.api === "notify({options})")?.status).toBe("caveat");
  });
});

describe("formatFindings", () => {
  test("includes the kit-import removal instruction and replacements", async () => {
    const c = classify(await fixture("save-note-db.ts"));
    const text = formatFindings(c);
    expect(text).toContain("@johnlindquist/kit");
    expect(text).toContain("db [removed]");
    expect(text).toContain("toast [renamed] → hud");
  });
});

describe("metadata extraction", () => {
  test("comment metadata parses the launcher keys", async () => {
    const meta = extractCommentMetadata(await fixture("hello-world.ts"));
    expect(meta.name).toBe("Hello World");
    expect(meta.shortcut).toBe("cmd shift h");
  });

  test("typed metadata wins over comments", () => {
    const meta = extractEffectiveMetadata(
      `// Name: Old Name\nconst metadata = { name: "New Name", shortcut: "cmd k" };\n`,
    );
    expect(meta.name).toBe("New Name");
    expect(meta.shortcut).toBe("cmd k");
  });

  test("typed extraction handles export const and nested braces", () => {
    const meta = extractTypedMetadata(
      `export const metadata: Metadata = {\n  name: "X",\n  extra: { nested: true },\n  description: "Y",\n};\n`,
    );
    expect(meta.name).toBe("X");
    expect(meta.description).toBe("Y");
  });

  test("metadataLosses reports drops and changes, tolerates upgrades to typed", () => {
    const orig = extractEffectiveMetadata("// Name: A\n// Shortcut: cmd a\n");
    const portedOk = extractEffectiveMetadata(
      `const metadata = { name: "A", shortcut: "cmd a" };\n`,
    );
    expect(metadataLosses(orig, portedOk)).toEqual([]);
    const portedBad = extractEffectiveMetadata("// Name: A\n");
    expect(metadataLosses(orig, portedBad)).toEqual(['lost shortcut: "cmd a"']);
  });
});
