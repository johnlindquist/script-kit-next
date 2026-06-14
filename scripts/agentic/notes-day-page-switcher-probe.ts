/**
 * Runtime proof: Notes Cmd+P stays in the Notes/windowed experience and never
 * exposes or opens the main-window Day Page surface, even when sandbox day
 * files exist.
 *
 * Contract under test:
 *   1. Cmd+P in Notes opens the note switcher.
 *   2. The switcher contains no `daypage_YYYY-MM-DD` rows and no "Day Pages"
 *      section for files under `~/.scriptkit/brain/days/*.md`.
 *   3. The main window remains outside promptType "dayPage" after the Notes
 *      switcher user path.
 *
 * Protocol-only (simulateGpuiEvent), so the proof runs hidden-window safe.
 * Pass criteria in the printed report: every `checks[*].pass` is true.
 */
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/today/script-kit-gpui";

function walkElements(json: any, out: Json[] = []): Json[] {
  if (!json || typeof json !== "object") return out;
  if (Array.isArray(json)) {
    for (const item of json) walkElements(item, out);
    return out;
  }
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

const checks: { name: string; pass: boolean; detail?: Json }[] = [];
function check(name: string, pass: boolean, detail?: Json) {
  checks.push({ name, pass, detail });
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "notes-day-page-switcher-no-day",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

function gpuiKey(
  kind: string,
  key: string,
  modifiers: string[] = [],
  text?: string,
) {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind }, event },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );
}

try {
  // Seed two day files in the sandbox brain. Notes Cmd+P should ignore both;
  // day-page browsing belongs to the main-window Day Page switcher.
  const daysDir = join(driver.sessionDir, "home", ".scriptkit", "brain", "days");
  mkdirSync(daysDir, { recursive: true });
  const seededDate = "2026-06-01";
  const seededLine = "switcher probe alpha entry";
  writeFileSync(join(daysDir, `${seededDate}.md`), `${seededLine}\nsecond line\n`);
  const today = new Date();
  const todayStr = [
    today.getFullYear(),
    String(today.getMonth() + 1).padStart(2, "0"),
    String(today.getDate()).padStart(2, "0"),
  ].join("-");
  writeFileSync(join(daysDir, `${todayStr}.md`), "today seed entry\n");

  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);

  // --- 1. Cmd+P opens the switcher and does not list day page rows ---
  await gpuiKey("notes", "p", ["cmd"]);
  await Bun.sleep(900);

  const windows = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  check(
    "switcher_window_open",
    (windows.windows ?? []).some((w: Json) => w.id === "actions-dialog"),
    { windowIds: (windows.windows ?? []).map((w: Json) => w.id) },
  );

  const dialogElements = (await driver.request(
    { type: "getElements", target: { type: "kind", kind: "actionsDialog" } },
    { expect: "elementsResult", timeoutMs: 5000 },
  )) as Json;
  const nodes = walkElements(dialogElements);
  const dayRows = nodes.filter(
    (el) =>
      String(el.semanticId ?? el.id ?? "").includes("daypage_") ||
      /^\d{4}-\d{2}-\d{2} · |^Today · /.test(String(el.label ?? "")),
  );
  check("switcher_has_no_day_page_rows", dayRows.length === 0, {
    dayRowCount: dayRows.length,
    sample: dayRows.slice(0, 3).map((el) => ({
      id: el.semanticId ?? el.id,
      label: el.label,
    })),
    allNodes: nodes.slice(0, 25).map((el) => ({
      id: el.semanticId ?? el.id,
      label: el.label,
    })),
  });

  check(
    "seeded_past_day_absent",
    !nodes.some((el) =>
      JSON.stringify({
        id: el.semanticId ?? el.id,
        label: el.label,
        value: el.value,
      }).includes(seededDate),
    ),
    { seededDate },
  );

  const mainState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("main_window_not_day_page", mainState.promptType !== "dayPage", {
    promptType: mainState.promptType,
  });

  const log = await Bun.file(`${driver.sessionDir}/app.log`).text();
  check(
    "no_notes_day_page_handoff_logged",
    !log.includes("notes_note_switcher_day_page_handoff"),
    {
      browsePanelLogLines: log
        .split("\n")
        .filter((line) => /open_browse_panel|day_page/i.test(line))
        .slice(-5),
    },
  );
} finally {
  const failed = checks.filter((c) => !c.pass);
  console.log(
    JSON.stringify(
      { pass: failed.length === 0, checks, binary: BINARY },
      null,
      2,
    ),
  );
  await driver.close();
}
