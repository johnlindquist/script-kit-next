import { expect, test } from "bun:test";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";

function createTempRoot(): string {
  const root = mkdtempSync(join(tmpdir(), "script-kit-wiki-ingest-"));
  mkdirSync(join(root, "wiki"), { recursive: true });
  return root;
}

function writeJson(path: string, value: unknown): void {
  writeFileSync(path, JSON.stringify(value, null, 2), "utf8");
}

function runIngest(root: string, snapshot = "abc123") {
  return spawnSync(
    process.execPath,
    [
      join(process.cwd(), "scripts/wiki/ingest.ts"),
      "--root",
      root,
      "--snapshot",
      snapshot,
      "--config",
      "wiki/sources.json",
    ],
    { encoding: "utf8" }
  );
}

test("re-ingest preserves page-owned content with fenced code blocks and keeps the index single-line", () => {
  const root = createTempRoot();
  try {
    writeFileSync(join(root, "CLAUDE.md"), "# Repo Contract\n", "utf8");
    writeJson(join(root, "wiki", "sources.json"), {
      sources: [
        {
          id: "claude",
          path: "CLAUDE.md",
          title: "Repository contract",
          description: "Primary repository contract.",
        },
      ],
      pages: [
        {
          slug: "example-page",
          title: "Example Page",
          summary: "Bootstrap summary",
          sourceIds: ["claude"],
          facts: ["Bootstrap fact"],
          related: ["other-page"],
        },
        {
          slug: "other-page",
          title: "Other Page",
          summary: "Other summary",
          sourceIds: ["claude"],
          facts: ["Other fact"],
          related: ["example-page"],
        },
      ],
    });

    mkdirSync(join(root, "wiki", "pages"), { recursive: true });
    writeFileSync(
      join(root, "wiki", "pages", "example-page.md"),
      `---
title: "Example Page"
slug: "example-page"
sourceSnapshot: "oldsha"
sourceDocuments:
  - "raw/oldsha/CLAUDE.md"
relatedPages:
  - "other-page"
generatedBy: "manual"
generatedAt: "2026-04-04T00:00:00.000Z"
---

# Example Page

First paragraph stays on the page.

Second paragraph also stays on the page.

## Key Facts

- Hand-edited fact that must survive re-ingest.

## Key Files

- stale file list

## Source Documents

- stale source list

## Related Pages

- [other-page](./other-page.md)

## Usage

\`\`\`ts
## not-a-real-heading
console.log("keep fenced code intact");
\`\`\`
`,
      "utf8"
    );

    const result = runIngest(root, "abc123");

    expect(result.status).toBe(0);
    expect(result.stdout).toContain('"ok": true');
    expect(result.stderr).toContain('"event":"wiki_ingest.page_updated"');
    expect(result.stderr).toContain(
      '"event":"wiki_ingest.index_summary_normalized"'
    );

    const page = readFileSync(
      join(root, "wiki", "pages", "example-page.md"),
      "utf8"
    );
    const index = readFileSync(join(root, "wiki", "index.md"), "utf8");
    const log = readFileSync(join(root, "wiki", "log.md"), "utf8");

    // Page preserves full summary
    expect(page).toContain("First paragraph stays on the page.");
    expect(page).toContain("Second paragraph also stays on the page.");
    // Page preserves hand-edited key facts
    expect(page).toContain("- Hand-edited fact that must survive re-ingest.");
    // Page preserves fenced code block with ## inside it
    expect(page).toContain(
      [
        "## Usage",
        "```ts",
        "## not-a-real-heading",
        'console.log("keep fenced code intact");',
        "```",
      ].join("\n")
    );
    // Page has updated snapshot
    expect(page).toContain('sourceSnapshot: "abc123"');
    expect(page).toContain(
      "- [raw/abc123/CLAUDE.md](../raw/abc123/CLAUDE.md)"
    );

    // Index uses only first paragraph, single-line
    expect(index).toContain(
      "- [Example Page](./pages/example-page.md) — First paragraph stays on the page."
    );
    expect(index).not.toContain("Second paragraph also stays on the page.");

    // Log records the run
    expect(log).toContain("snapshot abc123");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("ingest fails fast when a page references an unknown related slug", () => {
  const root = createTempRoot();
  try {
    writeFileSync(join(root, "CLAUDE.md"), "# Repo Contract\n", "utf8");
    writeJson(join(root, "wiki", "sources.json"), {
      sources: [
        {
          id: "claude",
          path: "CLAUDE.md",
          title: "Repository contract",
          description: "Primary repository contract.",
        },
      ],
      pages: [
        {
          slug: "example-page",
          title: "Example Page",
          summary: "Summary",
          sourceIds: ["claude"],
          facts: ["Fact"],
          related: ["missing-page"],
        },
      ],
    });

    const result = runIngest(root, "abc123");

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(
      '"event":"wiki_ingest.invalid_related_page"'
    );
    expect(result.stderr).toContain(
      "Page example-page references unknown related page: missing-page"
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
