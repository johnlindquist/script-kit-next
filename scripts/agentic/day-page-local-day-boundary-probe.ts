#!/usr/bin/env bun
/**
 * Runtime proof that Day Page binds to the configured local day.
 *
 * Runs separate sandbox sessions with explicit SCRIPT_KIT_BRAIN_TZ values,
 * writes a marker through the real Day Page editor, and verifies the marker
 * lands in brain/days/<local-date>.md rather than an always-UTC date.
 */
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage, tapMainHotkey } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/local-day/script-kit-gpui";
const zones = ["Pacific/Kiritimati", "Pacific/Honolulu"] as const;
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
const receipts: Record<string, Json> = {};
const failures: string[] = [];

function ymdForZone(date: Date, timeZone: string): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const byType = Object.fromEntries(parts.map((part) => [part.type, part.value]));
  return `${byType.year}-${byType.month}-${byType.day}`;
}

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
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

async function runZone(zone: (typeof zones)[number]) {
  const marker = `local-day-${zone.replaceAll("/", "-")}-${runId}`;
  const driver = await Driver.launch({
    binary: BINARY,
    sandboxHome: true,
    sessionName: `day-page-local-day-${zone.split("/").pop()?.toLowerCase()}`,
    defaultTimeoutMs: 8000,
    env: {
      SCRIPT_KIT_BRAIN_TZ: zone,
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    },
  });

  try {
    const state = await openDayPage(driver, `${runId}-${zone}`);
    check(`${zone}:opened_day_page`, state.promptType === "dayPage", {
      promptType: state.promptType,
    });

    const elements = await driver.getElements(
      { target: { type: "main" }, limit: 120 },
      { timeoutMs: 5000 },
    );
    const editor = walkElements(elements).find(
      (el) => el.semanticId === "input:day-page-editor",
    );
    check(`${zone}:dayPageEditorVisible`, Boolean(editor), {
      focusedSemanticId: (elements as Json).focusedSemanticId ?? null,
    });

    const content = `# Local day boundary\n\n${marker}`;
    const batch = (await driver.batch(
      [
        { type: "setInput", text: content },
        {
          type: "waitFor",
          condition: {
            type: "stateMatch",
            state: { promptType: "dayPage", inputValue: content },
          },
        },
      ],
      { timeoutMs: 5000 },
    )) as Json;
    check(`${zone}:set_marker`, batch.success === true, { batch });

    await driver.simulateKey("s", ["cmd"]);
    await Bun.sleep(900);

    const now = new Date();
    const expectedLocalDate = ymdForZone(now, zone);
    const utcDate = ymdForZone(now, "UTC");
    const daysDir = join(driver.sessionDir, "home", ".scriptkit", "brain", "days");
    const createdDayFiles = existsSync(daysDir)
      ? readdirSync(daysDir).filter((file) => file.endsWith(".md")).sort()
      : [];
    const expectedFile = join(daysDir, `${expectedLocalDate}.md`);
    const wrongUtcFile = join(daysDir, `${utcDate}.md`);
    const expectedContent = existsSync(expectedFile)
      ? readFileSync(expectedFile, "utf8")
      : "";
    const wrongUtcContent =
      utcDate !== expectedLocalDate && existsSync(wrongUtcFile)
        ? readFileSync(wrongUtcFile, "utf8")
        : "";

    check(`${zone}:markerFoundInExpectedFile`, expectedContent.includes(marker), {
      expectedLocalDate,
      utcDate,
      createdDayFiles,
      expectedFile,
    });
    check(
      `${zone}:markerAbsentFromUtcWrongFileWhenDifferent`,
      utcDate === expectedLocalDate || !wrongUtcContent.includes(marker),
      {
        expectedLocalDate,
        utcDate,
        wrongUtcFileExists: existsSync(wrongUtcFile),
      },
    );

    await tapMainHotkey(driver, `${runId}-${zone}`, "back-to-launcher");
    await driver.waitForState(
      { windowVisible: true, promptType: "none" },
      { timeoutMs: 8000 },
    );
    await tapMainHotkey(driver, `${runId}-${zone}`, "reopen-day-page");
    const reopened = (await driver.getState({ timeoutMs: 5000 })) as Json;
    check(`${zone}:mainWindowStillReopenable`, reopened.promptType === "dayPage", {
      promptType: reopened.promptType,
    });

    return {
      zone,
      expectedLocalDate,
      utcDate,
      createdDayFiles,
    };
  } finally {
    await driver.close();
  }
}

const zoneSummaries = [];
for (const zone of zones) {
  zoneSummaries.push(await runZone(zone));
}

const expectedDateByZone = Object.fromEntries(
  zoneSummaries.map((summary) => [summary.zone, summary.expectedLocalDate]),
);
const utcDates = new Set(zoneSummaries.map((summary) => summary.utcDate));
const hasZoneDifferentFromUtc = zoneSummaries.some(
  (summary) => summary.expectedLocalDate !== summary.utcDate,
);
check("at_least_one_zone_differs_from_utc", hasZoneDifferentFromUtc, {
  expectedDateByZone,
  utcDates: [...utcDates],
});

const pass = failures.length === 0;
console.log(
  JSON.stringify(
    {
      pass,
      failures,
      expectedDateByZone,
      createdDayFilesByZone: Object.fromEntries(
        zoneSummaries.map((summary) => [summary.zone, summary.createdDayFiles]),
      ),
      receipts,
    },
    null,
    2,
  ),
);
if (!pass) process.exitCode = 1;
