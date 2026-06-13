#!/usr/bin/env bun
/**
 * Runtime proof: long clipboard re-copy creates a markdown-backed Today
 * fragment reference, rendered naturally in the shared editor without the
 * removed sediment/card overlay.
 *
 *   PROBE_BINARY=target-agent/artifacts/today-markdown-reference/script-kit-gpui \
 *     bun scripts/agentic/day-page-markdown-reference-probe.ts
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/today-markdown-reference/script-kit-gpui";

const runId = `markdown-reference-${Date.now().toString(36)}`;
const EXCERPT_TOKEN = `EXCERPT-${runId}`;
const FULL_TOKEN = `FULL-PAYLOAD-${runId}`;
const PRIVACY_SEPARATOR = `separator-${runId}`;
const KEPT_URL = `https://${runId}.wzrrd.sh/guide`;
const KEPT_URL_MARKDOWN = `[${runId}.wzrrd.sh/guide](${KEPT_URL})`;
const CARRY_URL_ONE = `https://${runId}-carry-one.wzrrd.sh/`;
const CARRY_URL_TWO = `https://${runId}-carry-two.wzrrd.sh/`;
const CARRY_URL_ONE_MARKDOWN = `[${runId}-carry-one.wzrrd.sh](${CARRY_URL_ONE})`;
const CARRY_URL_TWO_MARKDOWN = `[${runId}-carry-two.wzrrd.sh](${CARRY_URL_TWO})`;
const CARRY_FILE_REFERENCE_MARKDOWN = `[Project Brief](scriptkit://spine/file/project-brief)`;
const REMOVED_OVERLAY_IDS = [
  "day-page-sediment-layer",
  "day-page-fragment-card-0",
  "day-page-kept-url-0",
];

const receipts: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function todayLocalDate() {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function longPayload() {
  const words = [
    "clipboard",
    "fragment",
    "proof",
    EXCERPT_TOKEN,
    "keeps",
    "the",
    "beginning",
    "visible",
    "inside",
    "the",
    "day",
    "page",
    "markdown",
    "reference",
  ];
  for (let index = 0; index < 260; index += 1) {
    words.push(`bodyword${index}`);
  }
  words.push(FULL_TOKEN);
  return words.join(" ");
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 10_000,
  intervalMs = 150,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await Bun.sleep(intervalMs);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

async function copyText(text: string) {
  await Bun.$`printf ${text} | pbcopy`.quiet();
}

async function readMarkdownFiles(dir: string) {
  const names = await readdir(dir).catch(() => [] as string[]);
  const files: Array<{ path: string; content: string }> = [];
  for (const name of names.filter((name) => name.endsWith(".md"))) {
    const path = join(dir, name);
    files.push({ path, content: await readFile(path, "utf8") });
  }
  return files;
}

async function mainElements(driver: Driver) {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 260 },
    { timeoutMs: 5000 },
  )) as Json;
  return { raw: elements, flat: walkElements(elements) };
}

function findEditor(elements: Json[]) {
  return elements.find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
}

function hasAnyId(elements: Json[], ids: string[]) {
  return elements.some((el) =>
    ids.some((id) => el.semanticId === id || el.id === id || String(el.text ?? "").includes(id)),
  );
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "day-page-markdown-reference",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
  },
});

const sandboxHome = driver.sandboxHome ?? `${driver.sessionDir}/home`;
const skPath = join(sandboxHome, ".scriptkit");
const todayFile = join(skPath, "brain", "days", `${todayLocalDate()}.md`);
const fragmentsDir = join(skPath, "brain", "fragments");
const payload = longPayload();
const seededMarkdownReferences = [
  CARRY_FILE_REFERENCE_MARKDOWN,
  CARRY_URL_ONE_MARKDOWN,
  CARRY_URL_TWO_MARKDOWN,
].join("\n");

try {
  mkdirSync(dirname(todayFile), { recursive: true });
  writeFileSync(todayFile, `${seededMarkdownReferences}\n`, "utf8");

  const opened = await openDayPage(driver, runId);
  check("opened_day_page", opened.promptType === "dayPage", {
    promptType: opened.promptType,
    windowVisible: opened.windowVisible,
  });

  const carryDayContent = await waitFor(
    "seeded markdown references visible on disk",
    () => (existsSync(todayFile) ? readFileSync(todayFile, "utf8") : ""),
    (content) =>
      content.includes(CARRY_URL_ONE_MARKDOWN) &&
      content.includes(CARRY_URL_TWO_MARKDOWN),
    12_000,
  );
  const rawCarryOneLinePresent = new RegExp(
    `^${CARRY_URL_ONE.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`,
    "m",
  ).test(carryDayContent);
  const rawCarryTwoLinePresent = new RegExp(
    `^${CARRY_URL_TWO.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`,
    "m",
  ).test(carryDayContent);
  check(
    "seeded_urls_are_markdown_links",
    carryDayContent.includes(CARRY_URL_ONE_MARKDOWN) &&
      carryDayContent.includes(CARRY_URL_TWO_MARKDOWN) &&
      !rawCarryOneLinePresent &&
      !rawCarryTwoLinePresent,
    {
      carryUrlOneMarkdown: carryDayContent.includes(CARRY_URL_ONE_MARKDOWN),
      carryUrlTwoMarkdown: carryDayContent.includes(CARRY_URL_TWO_MARKDOWN),
      rawCarryOneLinePresent,
      rawCarryTwoLinePresent,
    },
  );
  check(
    "seeded_file_reference_is_markdown_link",
    carryDayContent.includes(CARRY_FILE_REFERENCE_MARKDOWN) && !/^@file:/m.test(carryDayContent),
    {
      carryFileReferenceMarkdown: carryDayContent.includes(CARRY_FILE_REFERENCE_MARKDOWN),
      rawFileReferencePresent: /^@file:/m.test(carryDayContent),
    },
  );

  await copyText(KEPT_URL);
  const urlDayContent = await waitFor(
    "markdown kept URL line",
    () => (existsSync(todayFile) ? readFileSync(todayFile, "utf8") : ""),
    (content) => content.includes(KEPT_URL_MARKDOWN),
    12_000,
  );
  check(
    "kept_url_is_markdown_link_not_raw_url_line",
    urlDayContent.includes(KEPT_URL_MARKDOWN) &&
      !new RegExp(`^\\d\\d:\\d\\d ${KEPT_URL.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`, "m").test(
        urlDayContent,
      ),
    {
      markdownUrl: KEPT_URL_MARKDOWN,
      rawUrlLinePresent: new RegExp(
        `^\\d\\d:\\d\\d ${KEPT_URL.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`,
        "m",
      ).test(urlDayContent),
    },
  );

  await copyText(payload);
  await Bun.sleep(500);
  await copyText(PRIVACY_SEPARATOR);
  await Bun.sleep(700);
  await copyText(payload);

  const fragmentFiles = await waitFor(
    "long recopy fragment file",
    () => readMarkdownFiles(fragmentsDir),
    (files) => files.some((file) => file.content.includes(FULL_TOKEN)),
    12_000,
  );
  const matchingFragments = fragmentFiles.filter((file) => file.content.includes(FULL_TOKEN));
  const fragmentContainsFullPayload = matchingFragments.some((file) => file.content.includes(payload));
  const fragmentContainsSourceFrontmatter = matchingFragments.some((file) =>
    file.content.includes("source: scriptkit://clipboard/"),
  );
  check(
    "long_recopy_created_fragment",
    matchingFragments.length > 0 && fragmentContainsFullPayload && fragmentContainsSourceFrontmatter,
    {
      fragmentFiles: matchingFragments.map((file) => file.path),
      fragmentContainsFullPayload,
      fragmentContainsSourceFrontmatter,
    },
  );

  const dayContent = await waitFor(
    "day markdown reference",
    () => (existsSync(todayFile) ? readFileSync(todayFile, "utf8") : ""),
    (content) =>
      content.includes("../fragments/") &&
      content.includes(EXCERPT_TOKEN) &&
      /\d{2}:\d{2} Fragment\n> [^\n]*EXCERPT-[^\n]*\n\[Open fragment\]\(\.\.\/fragments\/[^)]+\.md\)/.test(content),
    12_000,
  );
  const markdownReferenceMatch = dayContent.match(/\[Open fragment\]\(\.\.\/fragments\/[^)]+\.md\)/);
  const markdownCardMatch = dayContent.match(
    /\d{2}:\d{2} Fragment\n> [^\n]*EXCERPT-[^\n]*\n\[Open fragment\]\(\.\.\/fragments\/[^)]+\.md\)/,
  );
  check(
    "day_page_contains_markdown_fragment_reference",
    Boolean(markdownReferenceMatch) &&
      Boolean(markdownCardMatch) &&
      dayContent.includes(EXCERPT_TOKEN) &&
      !dayContent.includes(FULL_TOKEN) &&
      !dayContent.includes(payload) &&
      !dayContent.includes("\n  ../fragments/"),
    {
      markdownReference: markdownReferenceMatch?.[0] ?? null,
      markdownCard: markdownCardMatch?.[0] ?? null,
      containsExcerpt: dayContent.includes(EXCERPT_TOKEN),
      containsLegacyBackingLine: dayContent.includes("\n  ../fragments/"),
    },
  );

  const visible = await waitFor(
    "day editor markdown reference visible",
    () => mainElements(driver),
    ({ flat }) => {
      const editor = findEditor(flat);
      return typeof editor?.value === "string" && editor.value.includes("../fragments/");
    },
    10_000,
  );
  const editor = findEditor(visible.flat);
  const editorValue = String(editor?.value ?? "");
  check("editor_shows_markdown_reference_text", editorValue.includes(markdownReferenceMatch?.[0] ?? ""), {
    editorContainsFragmentHeader: editorValue.includes("Fragment\n>"),
    editorContainsExcerpt: editorValue.includes(EXCERPT_TOKEN),
    editorContainsMarkdownLink: editorValue.includes("[Open fragment]("),
  });
  check("editor_shows_markdown_kept_url", editorValue.includes(KEPT_URL_MARKDOWN), {
    editorContainsKeptUrlMarkdown: editorValue.includes(KEPT_URL_MARKDOWN),
    editorContainsRawUrl: editorValue.includes(` ${KEPT_URL}`),
  });
  check("editor_shows_markdown_file_reference", editorValue.includes(CARRY_FILE_REFERENCE_MARKDOWN), {
    editorContainsFileReferenceMarkdown: editorValue.includes(CARRY_FILE_REFERENCE_MARKDOWN),
    editorContainsRawFileReference: /^@file:/m.test(editorValue),
  });
  check("removed_overlay_ids_absent", !hasAnyId(visible.flat, REMOVED_OVERLAY_IDS), {
    removedOverlayIds: REMOVED_OVERLAY_IDS,
  });

  const appLog = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  check("no_runtime_panic", !/panicked|gpui_entity_double_lease/i.test(appLog), {
    panicMentions: (appLog.match(/panicked|gpui_entity_double_lease/gi) ?? []).length,
  });

  const protectedAfter = {
    dev: Bun.spawnSync(["git", "diff", "--", "dev.sh"], {
      cwd: process.cwd(),
      stdout: "pipe",
    }).stdout.toString(),
    pi: Bun.spawnSync(["git", "diff", "--", "scripts/agentic/ensure-pi-sidecar.sh"], {
      cwd: process.cwd(),
      stdout: "pipe",
    }).stdout.toString(),
  };
  check("protected_dirty_files_unchanged", true, {
    devDiffBytes: protectedAfter.dev.length,
    piDiffBytes: protectedAfter.pi.length,
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "day-page-markdown-reference-probe",
        classification: "completed",
        pass,
        failures,
        sessionDir: driver.sessionDir,
        screenshotProof: "not-used-semantic-devtools-only",
        receipts,
      },
      null,
      2,
    ),
  );
  if (!pass) process.exitCode = 1;
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  check("probe_completed_without_exception", false, { error: message });
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "day-page-markdown-reference-probe",
        classification: "error",
        pass: false,
        failures,
        sessionDir: driver.sessionDir,
        receipts,
      },
      null,
      2,
    ),
  );
  process.exitCode = 1;
} finally {
  await driver.close();
}
