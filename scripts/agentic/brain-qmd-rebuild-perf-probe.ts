#!/usr/bin/env bun
/**
 * Runtime proof: qmd brain files are canonical and brain.sqlite is a
 * rebuildable derived index. The probe writes only markdown fixtures under a
 * sandbox ~/.scriptkit/brain, wakes the indexer through the real Day Page
 * save path, deletes the derived DB, rebuilds from files, then proves deleted
 * canonical files are forgotten.
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/brain-qmd-rebuild-perf/script-kit-gpui \
 *     SCRIPT_KIT_BRAIN_TZ=America/Denver \
 *     bun scripts/agentic/brain-qmd-rebuild-perf-probe.ts
 */
import { Database } from "bun:sqlite";
import { Driver, type Json } from "../devtools/driver.ts";
import { openDayPage } from "./day-page-open-helper.ts";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { dirname, join } from "node:path";

type SourceKind = "note" | "day_page" | "fragment";

type Fixture = {
  source: SourceKind;
  sourceId: string;
  path: string;
  title: string;
  derivedContent: string;
  token: string;
  canonicalHash: string;
};

type DriverPass = {
  driver: Driver;
  sessionDir: string;
  appLog: string;
};

const binary =
  process.env.PROBE_BINARY ??
  process.argv[2] ??
  "target-agent/artifacts/brain-qmd-rebuild-perf/script-kit-gpui";
const timezone = process.env.SCRIPT_KIT_BRAIN_TZ || "America/Denver";
const runId = `qmd-proof-${Date.now().toString(36)}`;
const sandboxRoot = `/tmp/sk-brain-qmd-rebuild-perf-${process.pid}-${Date.now().toString(36)}`;
const sandboxHome = join(sandboxRoot, "home");
const skPath = join(sandboxHome, ".scriptkit");
const brainRoot = join(skPath, "brain");
const dbPath = join(skPath, "db", "brain.sqlite");
const noteCount = 24;
const fragmentCount = 6;
const budgets = {
  allDocsIndexedMs: 30000,
  forgetMs: 30000,
  getStateP95Ms: 250,
  getElementsP95Ms: 500,
};

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function sha256(text: string): string {
  return createHash("sha256").update(text).digest("hex");
}

function localDateFor(date: Date, timeZone: string): string {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const part = (type: string) =>
    parts.find((entry) => entry.type === type)?.value ?? "";
  return `${part("year")}-${part("month")}-${part("day")}`;
}

function addDays(date: Date, days: number) {
  const next = new Date(date);
  next.setUTCDate(next.getUTCDate() + days);
  return next;
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 30000,
  intervalMs = 250,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await sleep(intervalMs);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

function isoFor(offsetSeconds: number) {
  return new Date(Date.UTC(2026, 5, 13, 12, 0, offsetSeconds)).toISOString();
}

function noteFrontmatter(id: string, index: number, source: string) {
  const created = isoFor(index);
  const lines = [
    "---",
    `id: ${id}`,
    `created: ${created}`,
    `updated: ${created}`,
    "tags: [qmd, proof]",
    `aliases: [QMD Proof ${index}]`,
    `source: ${source}`,
  ];
  if (index % 7 === 0) {
    lines.push("pinned: true");
  }
  lines.push("---", "");
  return lines.join("\n");
}

function writeFile(path: string, content: string) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

function writeCanonicalFixtures(): Fixture[] {
  const fixtures: Fixture[] = [];
  const notesDir = join(brainRoot, "notes");
  const daysDir = join(brainRoot, "days");
  const fragmentsDir = join(brainRoot, "fragments");
  mkdirSync(notesDir, { recursive: true });
  mkdirSync(daysDir, { recursive: true });
  mkdirSync(fragmentsDir, { recursive: true });

  for (let index = 0; index < noteCount; index += 1) {
    const id = randomUUID();
    const slug = `${runId}-note-${String(index).padStart(2, "0")}`;
    const token = `${runId}-note-token-${String(index).padStart(2, "0")}`;
    const title = `QMD proof note ${index}`;
    const body = [
      `# ${title}`,
      "",
      `This canonical note proves ${token} is indexed from markdown only.`,
      `It also includes the shared run token ${runId}.`,
    ].join("\n");
    const raw = `${noteFrontmatter(id, index, `scriptkit://qmd-proof/${slug}`)}${body}\n`;
    const path = join(notesDir, `${slug}.md`);
    writeFile(path, raw);
    fixtures.push({
      source: "note",
      sourceId: id,
      path,
      title,
      derivedContent: `${body}\n`,
      token,
      canonicalHash: sha256(`${body}\n`),
    });
  }

  for (let index = 0; index < 2; index += 1) {
    const date = localDateFor(addDays(new Date(), -index - 1), timezone);
    const token = `${runId}-day-token-${index}`;
    const content = [
      `# ${date}`,
      "",
      `Daily qmd proof entry ${index}.`,
      `The canonical day token is ${token}.`,
      `Shared run token: ${runId}.`,
      "",
    ].join("\n");
    const path = join(daysDir, `${date}.md`);
    writeFile(path, content);
    fixtures.push({
      source: "day_page",
      sourceId: date,
      path,
      title: `Day Page ${date}`,
      derivedContent: content,
      token,
      canonicalHash: sha256(content),
    });
  }

  const longBody = Array.from({ length: 280 }, (_, word) => `fragment${word}`).join(" ");
  for (let index = 0; index < fragmentCount; index += 1) {
    const id = randomUUID();
    const fragmentId = `2026-06-13-12${String(index).padStart(2, "0")}-${runId}-fragment-${String(index).padStart(2, "0")}`;
    const token = `${runId}-fragment-token-${String(index).padStart(2, "0")}`;
    const source = `scriptkit://qmd-proof/fragment-${index}`;
    const body = [
      `Fragment fixture ${index} carries ${token}.`,
      `Shared run token: ${runId}.`,
      longBody,
    ].join("\n\n");
    const raw = `${noteFrontmatter(id, 100 + index, source)}${body}\n`;
    const path = join(fragmentsDir, `${fragmentId}.md`);
    writeFile(path, raw);
    fixtures.push({
      source: "fragment",
      sourceId: fragmentId,
      path,
      title: `Fragment: ${source}`,
      derivedContent: `${body}\n\n\nProvenance: ${source}`,
      token,
      canonicalHash: sha256(`${body}\n\n\nProvenance: ${source}`),
    });
  }

  return fixtures;
}

function removeBrainDb() {
  for (const suffix of ["", "-wal", "-shm"]) {
    const path = `${dbPath}${suffix}`;
    if (existsSync(path)) unlinkSync(path);
  }
}

function readBrainDocs() {
  if (!existsSync(dbPath)) return [];
  const db = new Database(dbPath, { readonly: true });
  try {
    return db
      .query(
        `SELECT source, source_id, title, content, content_hash, updated_at
         FROM brain_docs
         WHERE content LIKE ?1 OR title LIKE ?1 OR source_id LIKE ?1
         ORDER BY source, source_id`,
      )
      .all(`%${runId}%`) as Array<{
      source: SourceKind;
      source_id: string;
      title: string;
      content: string;
      content_hash: string;
      updated_at: number;
    }>;
  } catch {
    return [];
  } finally {
    db.close();
  }
}

function readSourceCounts() {
  const counts: Record<string, number> = {};
  for (const row of readBrainDocs()) {
    counts[row.source] = (counts[row.source] ?? 0) + 1;
  }
  return counts;
}

function fixtureKey(fixture: Pick<Fixture, "source" | "sourceId">) {
  return `${fixture.source}:${fixture.sourceId}`;
}

function docsByKey() {
  return new Map(readBrainDocs().map((doc) => [`${doc.source}:${doc.source_id}`, doc]));
}

function fixtureRowFingerprints(fixtures: Fixture[]) {
  const byKey = docsByKey();
  return fixtures
    .map((fixture) => {
      const key = fixtureKey(fixture);
      const doc = byKey.get(key);
      return `${key}|${doc?.title ?? ""}|${doc?.content_hash ?? ""}`;
    })
    .sort();
}

function derivedProof(fixtures: Fixture[], startedAt: number) {
  const byKey = docsByKey();
  const missing = fixtures.filter((fixture) => !byKey.has(fixtureKey(fixture)));
  const mismatched = fixtures.filter((fixture) => {
    const doc = byKey.get(fixtureKey(fixture));
    return doc == null || doc.content !== fixture.derivedContent;
  });
  const docs = readBrainDocs();
  return {
    sourceCounts: readSourceCounts(),
    rows: docs.length,
    allUniqueTokensFound:
      missing.length === 0 && fixtures.every((fixture) => byKey.get(fixtureKey(fixture))?.content.includes(fixture.token)),
    contentMatchesCanonical: mismatched.length === 0,
    missing: missing.map(fixtureKey),
    mismatched: mismatched.map(fixtureKey),
    timeToAllDocsMs: Date.now() - startedAt,
  };
}

async function launchPass(label: string): Promise<DriverPass> {
  const driver = await Driver.launch({
    sessionName: `brain-qmd-rebuild-perf-${label}`,
    sandboxHome: false,
    binary,
    readyTimeoutMs: 15000,
    defaultTimeoutMs: 10000,
    env: {
      HOME: sandboxHome,
      SK_PATH: skPath,
      SCRIPT_KIT_BRAIN_TZ: timezone,
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    },
  });
  return {
    driver,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
  };
}

async function triggerIndexerViaDayPage(driver: Driver, label: string) {
  const state = await openDayPage(driver, `${runId}-${label}`);
  const localDate = localDateFor(new Date(), timezone);
  const token = `${runId}-wake-${label}`;
  const content = [`# ${localDate}`, "", `Brain qmd wake token ${token}.`, ""].join("\n");
  const batch = await driver.batch([{ type: "setInput", text: content }], {
    timeoutMs: 10000,
  });
  driver.simulateKey("s", ["cmd"]);
  const dayPath = join(brainRoot, "days", `${localDate}.md`);
  await waitFor(
    `day page wake save ${label}`,
    () => (existsSync(dayPath) ? readFileSync(dayPath, "utf8") : ""),
    (value) => value.includes(token),
    10000,
  );
  return {
    openedDayPage: state.promptType === "dayPage",
    cmdSSent: true,
    dayPageSaveObserved: true,
    batchSuccess: batch.success === true,
    wakeToken: token,
  };
}

async function waitForDerivedDocs(fixtures: Fixture[], timeoutMs: number) {
  const startedAt = Date.now();
  return waitFor(
    "derived docs",
    () => derivedProof(fixtures, startedAt),
    (proof) => proof.allUniqueTokensFound && proof.contentMatchesCanonical,
    timeoutMs,
    500,
  );
}

function deleteCanonicalSubset(fixtures: Fixture[]) {
  const bySource = new Map<SourceKind, number>();
  const deleted: Fixture[] = [];
  for (const fixture of fixtures) {
    const count = bySource.get(fixture.source) ?? 0;
    const limit = fixture.source === "note" ? 3 : 1;
    if (count < limit) {
      bySource.set(fixture.source, count + 1);
      if (existsSync(fixture.path)) {
        unlinkSync(fixture.path);
        deleted.push(fixture);
      }
    }
  }
  return deleted;
}

async function waitForForgottenDocs(deleted: Fixture[], remaining: Fixture[]) {
  const startedAt = Date.now();
  return waitFor(
    "forgotten docs",
    () => {
      const byKey = docsByKey();
      const stale = deleted.filter((fixture) => byKey.has(fixtureKey(fixture)));
      const missingRemaining = remaining.filter((fixture) => !byKey.has(fixtureKey(fixture)));
      return {
        deletedCanonicalFiles: deleted.length,
        staleDerivedRowsRemoved: stale.length === 0,
        remainingRowsPreserved: missingRemaining.length === 0,
        stale: stale.map(fixtureKey),
        missingRemaining: missingRemaining.map(fixtureKey),
        timeToForgetMs: Date.now() - startedAt,
      };
    },
    (proof) => proof.staleDerivedRowsRemoved && proof.remainingRowsPreserved,
    budgets.forgetMs,
    500,
  );
}

function percentile(values: number[], p: number) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1);
  return sorted[index];
}

async function sampleDevtoolsResponsiveness(driver: Driver) {
  const getStateMs: number[] = [];
  const getElementsMs: number[] = [];
  for (let index = 0; index < 20; index += 1) {
    const started = Date.now();
    await driver.getState({ timeoutMs: 5000 });
    getStateMs.push(Date.now() - started);
  }
  for (let index = 0; index < 10; index += 1) {
    const started = Date.now();
    await driver.getElements({}, { timeoutMs: 5000 });
    getElementsMs.push(Date.now() - started);
  }
  return {
    getStateSamples: getStateMs.length,
    getStateP50Ms: percentile(getStateMs, 50),
    getStateP95Ms: percentile(getStateMs, 95),
    getElementsSamples: getElementsMs.length,
    getElementsP95Ms: percentile(getElementsMs, 95),
  };
}

function protectedDiff(path: string) {
  try {
    return Bun.spawnSync(["git", "diff", "--", path], {
      cwd: process.cwd(),
      stdout: "pipe",
      stderr: "pipe",
    }).stdout.toString();
  } catch {
    return "";
  }
}

rmSync(sandboxRoot, { recursive: true, force: true });
mkdirSync(skPath, { recursive: true });

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "brain-qmd-rebuild-perf-probe",
  classification: "blocked",
  seededDb: false,
  screenshotProof: "not-used-semantic-devtools-only",
  binary,
  timezone,
  sandboxHome,
  derivedDbPath: dbPath,
  protectedDirtyFiles: ["dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
  budgets,
  pass: false,
  failures: [] as string[],
};

let activeDriver: Driver | null = null;

try {
  const protectedBefore = {
    dev: protectedDiff("dev.sh"),
    pi: protectedDiff("scripts/agentic/ensure-pi-sidecar.sh"),
  };
  const fixtures = writeCanonicalFixtures();
  const expectedBySource = fixtures.reduce<Record<string, number>>((counts, fixture) => {
    counts[fixture.source] = (counts[fixture.source] ?? 0) + 1;
    return counts;
  }, {});
  receipt.canonicalFixtureProof = {
    notesWritten: fixtures.filter((fixture) => fixture.source === "note").length,
    dayPagesWritten: fixtures.filter((fixture) => fixture.source === "day_page").length,
    fragmentsWritten: fixtures.filter((fixture) => fixture.source === "fragment").length,
    hashes: fixtures.slice(0, 8).map((fixture) => `sha256:${fixture.canonicalHash}`),
  };
  receipt.preIndexProof = {
    brainDbExistedBeforeLaunch: existsSync(dbPath),
    preWriteDerivedRows: readBrainDocs().length,
  };

  const pass1 = await launchPass("initial");
  activeDriver = pass1.driver;
  receipt.sessionDir = pass1.sessionDir;
  receipt.appLog = pass1.appLog;
  receipt.wakeProof = await triggerIndexerViaDayPage(pass1.driver, "initial");
  const derived = await waitForDerivedDocs(fixtures, budgets.allDocsIndexedMs);
  const initialFingerprints = fixtureRowFingerprints(fixtures);
  receipt.derivedIndexProof = {
    ...derived,
    rowFingerprints: initialFingerprints,
    expectedSourceCounts: expectedBySource,
  };
  receipt.responsivenessProof = await sampleDevtoolsResponsiveness(pass1.driver);
  await pass1.driver.close();
  activeDriver = null;

  const beforeDeleteRows = readBrainDocs().length;
  removeBrainDb();
  const dbDeletedBetweenPasses = !existsSync(dbPath);
  const pass2 = await launchPass("rebuild");
  activeDriver = pass2.driver;
  receipt.rebuildSessionDir = pass2.sessionDir;
  receipt.rebuildAppLog = pass2.appLog;
  receipt.rebuildWakeProof = await triggerIndexerViaDayPage(pass2.driver, "rebuild");
  const rebuilt = await waitForDerivedDocs(fixtures, budgets.allDocsIndexedMs);
  const rebuiltFingerprints = fixtureRowFingerprints(fixtures);
  receipt.rebuildFromFilesProof = {
    dbDeletedBetweenPasses,
    rowsBeforeDbDelete: beforeDeleteRows,
    rebuiltSourceCountsMatch:
      JSON.stringify(rebuilt.sourceCounts) === JSON.stringify(derived.sourceCounts),
    rebuiltRowsMatchInitial:
      JSON.stringify(rebuiltFingerprints) === JSON.stringify(initialFingerprints),
    rebuiltContentMatchesCanonical: rebuilt.contentMatchesCanonical,
    rebuiltAllUniqueTokensFound: rebuilt.allUniqueTokensFound,
    timeToAllDocsMs: rebuilt.timeToAllDocsMs,
    sourceCounts: rebuilt.sourceCounts,
  };

  const deleted = deleteCanonicalSubset(fixtures);
  const deletedKeys = new Set(deleted.map(fixtureKey));
  const remaining = fixtures.filter((fixture) => !deletedKeys.has(fixtureKey(fixture)));
  await pass2.driver.close();
  activeDriver = null;

  const pass3 = await launchPass("forget");
  activeDriver = pass3.driver;
  receipt.forgetSessionDir = pass3.sessionDir;
  receipt.forgetAppLog = pass3.appLog;
  receipt.deleteWakeProof = await triggerIndexerViaDayPage(pass3.driver, "forget");
  receipt.forgetProof = {
    ...(await waitForForgottenDocs(deleted, remaining)),
    proofMode: "fresh-app-cycle-after-canonical-delete",
  };
  await pass3.driver.close();
  activeDriver = null;

  const responsiveness = receipt.responsivenessProof as Json;
  const derivedProof = receipt.derivedIndexProof as Json;
  const rebuildProof = receipt.rebuildFromFilesProof as Json;
  const forgetProof = receipt.forgetProof as Json;
  const protectedAfter = {
    dev: protectedDiff("dev.sh"),
    pi: protectedDiff("scripts/agentic/ensure-pi-sidecar.sh"),
  };
  receipt.protectedDirtyFilesUnchanged =
    protectedBefore.dev === protectedAfter.dev && protectedBefore.pi === protectedAfter.pi;
  const failures: string[] = [];
  if (!derivedProof.allUniqueTokensFound) failures.push("derived index missed fixture tokens");
  if (!derivedProof.contentMatchesCanonical) failures.push("derived content mismatched canonical markdown");
  if (!rebuildProof.dbDeletedBetweenPasses) failures.push("derived DB was not deleted between passes");
  if (!rebuildProof.rebuiltSourceCountsMatch) failures.push("rebuilt source counts did not match initial pass");
  if (!rebuildProof.rebuiltRowsMatchInitial) failures.push("rebuilt row fingerprints did not match initial pass");
  if (!rebuildProof.rebuiltContentMatchesCanonical) failures.push("rebuilt content mismatched canonical markdown");
  if (!forgetProof.staleDerivedRowsRemoved) failures.push("deleted canonical files were not forgotten");
  if (!forgetProof.remainingRowsPreserved) failures.push("remaining canonical rows were lost during forget sync");
  if (responsiveness.getStateP95Ms > budgets.getStateP95Ms) failures.push("getState p95 exceeded budget");
  if (responsiveness.getElementsP95Ms > budgets.getElementsP95Ms) failures.push("getElements p95 exceeded budget");
  if (!receipt.protectedDirtyFilesUnchanged) failures.push("protected dirty files changed during probe");
  receipt.failures = failures;
  receipt.pass = failures.length === 0;
  receipt.classification = failures.length === 0 ? "completed" : "failed";
} catch (error) {
  receipt.error = String(error);
  receipt.failures = [...((receipt.failures as string[]) ?? []), String(error)];
  if (activeDriver) {
    try {
      await activeDriver.close();
    } catch {
      // best effort cleanup
    }
  }
} finally {
  if (activeDriver) {
    try {
      await activeDriver.close();
    } catch {
      // best effort cleanup
    }
  }
}

console.log(JSON.stringify(receipt, null, 2));
