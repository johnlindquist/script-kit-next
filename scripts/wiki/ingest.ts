#!/usr/bin/env bun
import {
  appendFileSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";

type SourceSpec = {
  id: string;
  path: string;
  title: string;
  description: string;
};

type PageSpec = {
  slug: string;
  title: string;
  summary: string;
  sourceIds: string[];
  facts: string[];
  related: string[];
};

type WikiConfig = {
  sources: SourceSpec[];
  pages: PageSpec[];
};

type Args = {
  root: string;
  snapshot: string;
  config: string;
};

type IngestResult = {
  snapshot: string;
  rawCopied: number;
  pagesWritten: number;
  indexPath: string;
  logPath: string;
};

function log(event: string, fields: Record<string, unknown> = {}): void {
  console.error(JSON.stringify({ level: "info", event, ...fields }));
}

function normalize(relPath: string): string {
  return relPath.replace(/\\/g, "/").replace(/^\.?\//, "");
}

function ensureDir(path: string): void {
  mkdirSync(path, { recursive: true });
}

function parseArgs(argv: string[]): Args {
  let root = ".";
  let snapshot = "";
  let config = "wiki/sources.json";

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--root") {
      root = argv[i + 1] ?? root;
      i += 1;
    } else if (arg === "--snapshot") {
      snapshot = argv[i + 1] ?? snapshot;
      i += 1;
    } else if (arg === "--config") {
      config = argv[i + 1] ?? config;
      i += 1;
    }
  }

  if (!snapshot.trim()) {
    throw new Error("Missing required --snapshot <git-sha>.");
  }

  return { root, snapshot, config };
}

function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(path, "utf8")) as T;
}

function assertUniqueConfig(config: WikiConfig): void {
  const sourceIds = new Set<string>();
  for (const source of config.sources) {
    if (sourceIds.has(source.id)) {
      throw new Error(`Duplicate source id: ${source.id}`);
    }
    sourceIds.add(source.id);
  }

  const pageSlugs = new Set<string>();
  for (const page of config.pages) {
    if (pageSlugs.has(page.slug)) {
      throw new Error(`Duplicate page slug: ${page.slug}`);
    }
    pageSlugs.add(page.slug);

    for (const sourceId of page.sourceIds) {
      if (!sourceIds.has(sourceId)) {
        throw new Error(
          `Page ${page.slug} references unknown source id: ${sourceId}`
        );
      }
    }
  }
}

function rawWikiPath(snapshot: string, sourcePath: string): string {
  return normalize(`raw/${snapshot}/${normalize(sourcePath)}`);
}

function copyImmutableRaw(
  root: string,
  snapshot: string,
  source: SourceSpec
): string {
  const sourcePath = join(root, normalize(source.path));
  if (!existsSync(sourcePath)) {
    throw new Error(`Missing source file: ${source.path}`);
  }

  const destPath = join(root, "wiki", rawWikiPath(snapshot, source.path));
  ensureDir(dirname(destPath));

  if (!existsSync(destPath)) {
    copyFileSync(sourcePath, destPath);
    log("wiki_ingest.raw_copied", {
      sourceId: source.id,
      sourcePath: source.path,
      rawPath: rawWikiPath(snapshot, source.path),
    });
  } else {
    log("wiki_ingest.raw_exists", {
      sourceId: source.id,
      rawPath: rawWikiPath(snapshot, source.path),
    });
  }

  return rawWikiPath(snapshot, source.path);
}

function yamlList(values: string[], indent = 0): string {
  const prefix = " ".repeat(indent);
  return values.map((value) => `${prefix}- "${value}"`).join("\n");
}

function renderPage(
  page: PageSpec,
  sources: SourceSpec[],
  snapshot: string,
  generatedAt: string
): string {
  const rawPaths = sources.map((source) =>
    rawWikiPath(snapshot, source.path)
  );

  const keyFiles = sources
    .map(
      (source) =>
        `- \`${source.path}\` — ${source.title}. ${source.description}`
    )
    .join("\n");

  const keyFacts = page.facts.map((fact) => `- ${fact}`).join("\n");

  const sourceLinks = rawPaths
    .map((path) => `- [${path}](../${path})`)
    .join("\n");

  const relatedLinks = page.related
    .map((slug) => `- [${slug}](./${slug}.md)`)
    .join("\n");

  const escapedTitle = page.title.replace(/"/g, '\\"');

  return [
    "---",
    `title: "${escapedTitle}"`,
    `slug: "${page.slug}"`,
    `sourceSnapshot: "${snapshot}"`,
    "sourceDocuments:",
    yamlList(rawPaths, 2),
    "relatedPages:",
    yamlList(page.related, 2),
    `generatedBy: "scripts/wiki/ingest.ts"`,
    `generatedAt: "${generatedAt}"`,
    "---",
    "",
    `# ${page.title}`,
    "",
    page.summary,
    "",
    "## Key Facts",
    keyFacts,
    "",
    "## Key Files",
    keyFiles,
    "",
    "## Source Documents",
    sourceLinks,
    "",
    "## Related Pages",
    relatedLinks,
    "",
  ].join("\n");
}

function renderIndex(
  config: WikiConfig,
  snapshot: string,
  generatedAt: string
): string {
  const pageLines = config.pages.map(
    (page) =>
      `- [${page.title}](./pages/${page.slug}.md) — ${page.summary}`
  );

  const sourceLines = config.sources.map(
    (source) =>
      `- [${source.title}](./${rawWikiPath(snapshot, source.path)}) — \`${source.path}\``
  );

  return [
    "# Script Kit GPUI Wiki",
    "",
    "This file is generated by `scripts/wiki/ingest.ts`.",
    "",
    `- Snapshot: \`${snapshot}\``,
    `- Generated at: \`${generatedAt}\``,
    "",
    "## Pages",
    ...pageLines,
    "",
    "## Raw Sources",
    ...sourceLines,
    "",
  ].join("\n");
}

function ensureLogHeader(logPath: string): void {
  if (existsSync(logPath)) {
    return;
  }
  ensureDir(dirname(logPath));
  writeFileSync(
    logPath,
    "# Wiki Ingest Log\n\nAppend-only history of wiki ingest runs.\n",
    "utf8"
  );
}

function appendLog(
  logPath: string,
  snapshot: string,
  config: WikiConfig,
  generatedAt: string
): void {
  ensureLogHeader(logPath);

  const entry = [
    "",
    `## ${generatedAt} — snapshot ${snapshot}`,
    "",
    `- Raw sources processed: ${config.sources.length}`,
    `- Pages written: ${config.pages.length}`,
    `- Page slugs: ${config.pages.map((page) => `\`${page.slug}\``).join(", ")}`,
    "",
  ].join("\n");

  appendFileSync(logPath, entry, "utf8");
  log("wiki_ingest.log_appended", {
    logPath: "wiki/log.md",
    snapshot,
    pageCount: config.pages.length,
  });
}

function writePages(
  root: string,
  config: WikiConfig,
  snapshot: string,
  generatedAt: string
): number {
  const pageDir = join(root, "wiki", "pages");
  ensureDir(pageDir);

  const sourceMap = new Map(
    config.sources.map((source) => [source.id, source] as const)
  );

  let written = 0;

  for (const page of config.pages) {
    const sources = page.sourceIds
      .map((id) => sourceMap.get(id))
      .filter((value): value is SourceSpec => value !== undefined);

    const pagePath = join(pageDir, `${page.slug}.md`);
    writeFileSync(
      pagePath,
      renderPage(page, sources, snapshot, generatedAt),
      "utf8"
    );
    written += 1;
    log("wiki_ingest.page_written", {
      slug: page.slug,
      pagePath: `wiki/pages/${page.slug}.md`,
      sourceCount: sources.length,
    });
  }

  return written;
}

function run(
  root: string,
  snapshot: string,
  configPath: string
): IngestResult {
  const resolvedConfigPath = join(root, normalize(configPath));
  const config = readJson<WikiConfig>(resolvedConfigPath);
  assertUniqueConfig(config);

  const generatedAt = new Date().toISOString();

  log("wiki_ingest.start", {
    snapshot,
    configPath: normalize(configPath),
    sourceCount: config.sources.length,
    pageCount: config.pages.length,
  });

  for (const source of config.sources) {
    copyImmutableRaw(root, snapshot, source);
  }

  const pagesWritten = writePages(root, config, snapshot, generatedAt);

  const indexPath = join(root, "wiki", "index.md");
  ensureDir(dirname(indexPath));
  writeFileSync(
    indexPath,
    renderIndex(config, snapshot, generatedAt),
    "utf8"
  );
  log("wiki_ingest.index_written", {
    indexPath: "wiki/index.md",
    pageCount: config.pages.length,
  });

  const logPath = join(root, "wiki", "log.md");
  appendLog(logPath, snapshot, config, generatedAt);

  log("wiki_ingest.complete", {
    snapshot,
    rawCopied: config.sources.length,
    pagesWritten,
  });

  return {
    snapshot,
    rawCopied: config.sources.length,
    pagesWritten,
    indexPath: "wiki/index.md",
    logPath: "wiki/log.md",
  };
}

function main(): void {
  const args = parseArgs(process.argv.slice(2));
  const result = run(args.root, args.snapshot, args.config);
  process.stdout.write(
    `${JSON.stringify(
      { ok: true, type: "wikiIngestResult", ...result },
      null,
      2
    )}\n`
  );
}

main();
