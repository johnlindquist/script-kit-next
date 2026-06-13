#!/usr/bin/env bun
/**
 * Runtime proof: qmd-style brain provenance resources are available through
 * live MCP resources/read and refresh canonical brain markdown before recall.
 *
 *   PROBE_BINARY=target-agent/artifacts/brain-qmd-provenance/script-kit-gpui \
 *     bun scripts/agentic/brain-qmd-provenance-resource-probe.ts
 */

import { Database } from "bun:sqlite";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/brain-qmd-provenance/script-kit-gpui";
const runId = `qmd-provenance-${Date.now().toString(36)}`;
const dayOne = "2026-06-15";
const dayTwo = "2026-06-16";
const dayToken = `${runId}-day-token`;
const chatSourceId = "thread x#2/part";
const encodedChatSourceId = "thread%20x%232%2Fpart";
const expectedChatCitationUri = "brain://chat_turn/thread%20x%232%2Fpart";
const blockedNeedles = ["/Users/", ".scriptkit/db/brain.sqlite"];
const outPath = ".test-output/brain-qmd-provenance-resource-probe.json";

type Check = { name: string; pass: boolean; detail?: Json };

const checks: Check[] = [];
const failures: string[] = [];

function check(name: string, pass: boolean, detail: Json = {}) {
  checks.push({ name, pass, detail });
  if (!pass) failures.push(name);
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 10_000,
  intervalMs = 100,
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

function readJson(path: string): Json {
  return JSON.parse(readFileSync(path, "utf8")) as Json;
}

async function mcp(serverJsonPath: string, method: string, params: Json): Promise<Json> {
  const discovery = readJson(serverJsonPath);
  const endpoint = String(discovery.url ?? "").endsWith("/rpc")
    ? String(discovery.url)
    : `${String(discovery.url ?? "").replace(/\/$/, "")}/rpc`;
  const token = String(discovery.token ?? "");
  if (!endpoint || !token) {
    throw new Error(`invalid MCP discovery at ${serverJsonPath}`);
  }
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      authorization: `Bearer ${token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `${runId}-${method}-${Date.now()}`,
      method,
      params,
    }),
  });
  const body = (await response.json()) as Json;
  if (!response.ok || body.error) {
    throw new Error(`MCP ${method} failed: ${JSON.stringify(body)}`);
  }
  return body.result as Json;
}

async function readResource(serverJsonPath: string, uri: string): Promise<{ mimeType: string; text: string }> {
  const result = await mcp(serverJsonPath, "resources/read", { uri });
  const first = result.contents?.[0] as Json | undefined;
  if (!first || typeof first.text !== "string") {
    throw new Error(`resources/read returned no text for ${uri}: ${JSON.stringify(result)}`);
  }
  return {
    mimeType: String(first.mimeType ?? ""),
    text: first.text,
  };
}

async function listResources(serverJsonPath: string): Promise<Json[]> {
  const result = await mcp(serverJsonPath, "resources/list", {});
  return (result.resources ?? []) as Json[];
}

function parseJsonResource(resource: { text: string }) {
  return JSON.parse(resource.text) as Json;
}

function seedCanonicalDayPages(brainDir: string) {
  const daysDir = join(brainDir, "days");
  mkdirSync(daysDir, { recursive: true });
  writeFileSync(
    join(daysDir, `${dayOne}.md`),
    [
      "# Day One",
      "line one should not be returned",
      `line two contains ${dayToken}`,
      "line three has range proof",
      "line four should not be returned",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    join(daysDir, `${dayTwo}.md`),
    ["# Day Two", `second document also mentions ${dayToken}`, ""].join("\n"),
    "utf8",
  );
}

function seedChatTurnDoc(dbPath: string) {
  const db = new Database(dbPath);
  db.run(
    `INSERT INTO brain_docs (source, source_id, title, content, content_hash, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)
     ON CONFLICT(source, source_id) DO UPDATE SET
       title = excluded.title,
       content = excluded.content,
       content_hash = excluded.content_hash,
       updated_at = excluded.updated_at`,
    [
      "chat_turn",
      chatSourceId,
      "Thread source ref",
      "chat turn body with encoded citation proof",
      `${runId}-chat-hash`,
      230,
    ],
  );
  db.close();
}

function containsAnyNeedle(text: string, needles: string[]) {
  return needles.filter((needle) => text.includes(needle));
}

async function findFreePort(): Promise<number> {
  return await new Promise((resolve, reject) => {
    const server = createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
        } else {
          resolve(port);
        }
      });
    });
  });
}

let driver: Driver | null = null;
let driverClosed = false;

try {
  const mcpPort = await findFreePort();
  driver = await Driver.launch({
    binary: BINARY,
    sessionName: "brain-qmd-provenance",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
    env: {
      MCP_PORT: String(mcpPort),
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_BRAIN_TZ: "UTC",
    },
  });

  const sandboxHome = join(driver.sessionDir, "home");
  const skPath = join(sandboxHome, ".scriptkit");
  const brainDir = join(skPath, "brain");
  const dbPath = join(skPath, "db", "brain.sqlite");
  const serverJsonPath = join(skPath, "server.json");

  await waitFor(
    "MCP server.json",
    () => existsSync(serverJsonPath),
    Boolean,
    12_000,
  );

  seedCanonicalDayPages(brainDir);

  const resources = await listResources(serverJsonPath);
  const brain = resources.find((resource) => resource.uri === "kit://brain");
  const brainDescription = String(brain?.description ?? "");
  const descriptionContains = {
    "format=json": brainDescription.includes("format=json"),
    "kit://brain/doc": brainDescription.includes("kit://brain/doc"),
    "kit://brain/docs": brainDescription.includes("kit://brain/docs"),
  };
  check(
    "resources_list_advertises_brain_provenance",
    Boolean(brain) && Object.values(descriptionContains).every(Boolean),
    {
      uri: brain?.uri ?? null,
      descriptionContains,
    },
  );

  const statusResource = await readResource(serverJsonPath, "kit://brain");
  const recallUri = `kit://brain/recall?q=${encodeURIComponent(dayToken)}&format=json`;
  const recallResource = await readResource(serverJsonPath, recallUri);
  const recallJson = parseJsonResource(recallResource);
  const dayHit = (recallJson.hits ?? []).find((hit: Json) => hit.source === "day_page");
  const recallDetail = {
    uri: recallUri,
    mimeType: recallResource.mimeType,
    schemaVersion: recallJson.schemaVersion,
    hitSource: dayHit?.source ?? null,
    hasSourceId: typeof dayHit?.sourceId === "string",
    hasCitationUri: typeof dayHit?.citationUri === "string",
    hasLineRange: Number.isInteger(dayHit?.lineStart) && Number.isInteger(dayHit?.lineEnd),
  };
  check(
    "recall_json_refreshes_file_sources",
    recallResource.mimeType === "application/json" &&
      recallDetail.schemaVersion === 1 &&
      recallDetail.hitSource === "day_page" &&
      recallDetail.hasSourceId &&
      recallDetail.hasCitationUri &&
      recallDetail.hasLineRange,
    recallDetail,
  );

  const docUri = `kit://brain/doc?source=day_page&sourceId=${dayOne}&lines=3-4&format=json`;
  const docResource = await readResource(serverJsonPath, docUri);
  const docJson = parseJsonResource(docResource);
  const docContent = String(docJson.doc?.content ?? "");
  const docRangeDetail = {
    uri: docUri,
    lineStart: docJson.doc?.lineStart ?? null,
    lineEnd: docJson.doc?.lineEnd ?? null,
    excludedLine1: !docContent.includes("line one should not be returned"),
    excludedLine4: !docContent.includes("line four should not be returned"),
    contentHasToken: docContent.includes(dayToken),
  };
  check(
    "single_doc_json_line_range",
    docResource.mimeType === "application/json" &&
      docRangeDetail.lineStart === 3 &&
      docRangeDetail.lineEnd === 4 &&
      docRangeDetail.excludedLine1 &&
      docRangeDetail.excludedLine4 &&
      docRangeDetail.contentHasToken,
    docRangeDetail,
  );

  const invalidRange = await readResource(
    serverJsonPath,
    `kit://brain/doc?source=day_page&sourceId=${dayOne}&lines=0-2&format=json`,
  )
    .then((resource) => ({ failedClosed: false, text: resource.text }))
    .catch((error) => ({ failedClosed: true, text: String(error) }));
  check("invalid_line_range_fails_closed", invalidRange.failedClosed && !invalidRange.text.includes(dayToken), {
    leakedBody: invalidRange.text.includes(dayToken),
  });

  const docsUri = `kit://brain/docs?refs=day_page:${dayTwo},day_page:${dayOne},day_page:missing`;
  const docsResource = await readResource(serverJsonPath, docsUri);
  const docsJson = parseJsonResource(docsResource);
  const docs = (docsJson.docs ?? []) as Json[];
  const docsDetail = {
    refs: docs.map((doc) => doc.ref),
    found: docs.map((doc) => doc.found),
    missingError: docs[2]?.error ?? null,
  };
  check(
    "multi_doc_preserves_order_and_missing_receipt",
    docsResource.mimeType === "application/json" &&
      docsDetail.refs.join("|") === `day_page:${dayTwo}|day_page:${dayOne}|day_page:missing` &&
      docsDetail.found.join("|") === "true|true|false" &&
      docsDetail.missingError === "not_found",
    docsDetail,
  );

  seedChatTurnDoc(dbPath);
  const chatUri = `kit://brain/doc?source=chat_turn&sourceId=${encodedChatSourceId}&format=json`;
  const chatResource = await readResource(serverJsonPath, chatUri);
  const chatJson = parseJsonResource(chatResource);
  const chatDetail = {
    sourceId: chatJson.doc?.sourceId ?? null,
    citationUri: chatJson.doc?.citationUri ?? null,
    sourceIdRaw: chatJson.doc?.sourceId === chatSourceId,
    citationUriEncoded: chatJson.doc?.citationUri === expectedChatCitationUri,
  };
  check(
    "citation_uri_is_encoded_source_id_is_raw",
    chatDetail.sourceIdRaw && chatDetail.citationUriEncoded,
    chatDetail,
  );

  const metadataText = [
    recallResource.text,
    JSON.stringify({
      ...docJson.doc,
      content: undefined,
    }),
    JSON.stringify({
      docs: docs.map((doc) =>
        doc.doc
          ? {
              ...doc,
              doc: {
                ...doc.doc,
                content: undefined,
              },
            }
          : doc,
      ),
    }),
    JSON.stringify({
      ...chatJson.doc,
      content: undefined,
    }),
  ].join("\n");
  check("no_private_storage_paths_in_metadata", containsAnyNeedle(metadataText, blockedNeedles).length === 0, {
    blockedNeedles,
    leakedNeedles: containsAnyNeedle(metadataText, blockedNeedles),
  });

  check("runtime_log_has_no_panic", !/panicked|gpui_entity_double_lease/i.test(readFileSync(driver.logPath, "utf8")), {
    appLog: driver.logPath,
  });

  await driver.close();
  driverClosed = true;
  check("driver_closed", true, {});

  const pass = failures.length === 0 && checks.every((item) => item.pass);
  const receipt = {
    schemaVersion: 1,
    tool: "brain-qmd-provenance-resource-probe",
    classification: pass ? "completed" : "failed",
    pass,
    failures,
    binary: BINARY,
    sandboxHome,
    serverJson: serverJsonPath,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
    fixtures: {
      dayPages: [dayOne, dayTwo],
      chatTurnSourceId: chatSourceId,
      tokens: [dayToken],
    },
    checks,
  };
  mkdirSync(".test-output", { recursive: true });
  writeFileSync(outPath, `${JSON.stringify(receipt, null, 2)}\n`, "utf8");
  console.log(JSON.stringify(receipt, null, 2));
  if (!pass) process.exit(1);
} catch (error) {
  if (driver && !driverClosed) {
    await driver.close().catch(() => {});
    driverClosed = true;
  }
  const receipt = {
    schemaVersion: 1,
    tool: "brain-qmd-provenance-resource-probe",
    classification: "error",
    pass: false,
    failures: ["probe_completed_without_exception"],
    error: error instanceof Error ? error.message : String(error),
    binary: BINARY,
    sessionDir: driver?.sessionDir ?? null,
    appLog: driver?.logPath ?? null,
    checks,
  };
  mkdirSync(".test-output", { recursive: true });
  writeFileSync(outPath, `${JSON.stringify(receipt, null, 2)}\n`, "utf8");
  console.log(JSON.stringify(receipt, null, 2));
  process.exit(1);
}
