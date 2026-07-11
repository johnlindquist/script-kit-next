#!/usr/bin/env bun
// Layout-stability (CLS) eval for the main window — POLISH.md §2.
//
// Contracts enforced (hard failures):
//   A. persistent-leading-separator: every non-empty main-list state leads
//      with a section header row (even just "Results"); the header row never
//      vanishes as the query changes (memory: lists-lead-with-persistent-separator).
//   B. no-late-republish (fold-scoped): for a fixed query, the rows inside the
//      visible fold immediately after the keystroke are not reordered/replaced
//      after a settle beat (the query-frame latch). Appending sections below
//      the fold is allowed; mutating the fold or moving selection is not.
//   C. launcher round-trip coherence: main -> (type name, Enter) builtin ->
//      Escape returns to a coherent main list (empty filter, header-led row 0,
//      selection on the first selectable row). Row identity may reorder from
//      frecency; slot structure may not break.
//   D. rapid-hop coherence: hopping launcher->builtin->escape at speed and
//      landing on main leaves a coherent, header-led, selection-coerced list.
//
// Cross-view observations (reported, not failed): per-builtin zero-match
// behavior, chrome bounds deltas vs the main-menu baseline, and leading-header
// availability inside each builtin. These become fix tickets, and graduate to
// hard assertions once each surface exposes header rows to getElements.
//
// Requires the `getElements { includeHeaders: true }` primitive.
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const receiptPath = resolve(
  process.env.PROBE_RECEIPT ?? ".test-output/stability-cls-probe.json",
);
const binary = process.env.SCRIPT_KIT_GPUI_BINARY;

// ~9 rows fit the 422px content viewport at 44px/row; 12 covers fold + margin.
const FOLD_ROWS = 12;

const MAIN_QUERY_STATES = ["", "c", "cl", "cli", "clip", "zzqxv", ""];
const HOP_BUILTINS: Array<{ id: string; query: string; slug: string; expectedPromptType: string }> = [
  {
    id: "clipboard-history",
    query: "clipboard history",
    slug: "clipboard-history",
    expectedPromptType: "clipboardHistory",
  },
  {
    id: "emoji",
    query: "emoji picker",
    slug: "emoji",
    expectedPromptType: "emojiPicker",
  },
  {
    id: "process-manager",
    query: "process manager",
    slug: "process",
    expectedPromptType: "processManager",
  },
  {
    id: "settings",
    query: "script kit settings",
    slug: "settings",
    expectedPromptType: "settings",
  },
  {
    id: "dictation-history",
    query: "dictation history",
    slug: "dictation",
    expectedPromptType: "dictationHistory",
  },
];

interface RowInfo {
  semanticId: string;
  role: string | null;
  kind: string | null;
  text: string | null;
  selectable: boolean;
  selected: boolean;
}

function rowsOf(elementsResult: Json): RowInfo[] {
  const elements: Json[] = elementsResult.elements ?? [];
  return elements
    .filter((e) => {
      if (e.semanticId === "input:filter" || e.semanticId === "list:results") return false;
      if (typeof e.semanticId === "string" && e.semanticId.startsWith("handler-form:")) {
        return false;
      }
      if (e.type === "input" || e.type === "list") return false;
      if (e.role === "footer") return false;
      return true;
    })
    .map((e) => ({
      semanticId: String(e.semanticId ?? ""),
      role: e.role ?? null,
      kind: e.kind ?? null,
      text: typeof e.text === "string" ? e.text.slice(0, 60) : null,
      selectable: e.selectable === true,
      selected: e.selected === true,
    }));
}

function rowSignature(rows: RowInfo[]): string[] {
  return rows.map((r) => `${r.role ?? "?"}|${r.semanticId}`);
}

function chromeOf(layout: Json): Json {
  const pick = (name: string) =>
    (layout.components ?? []).find((c: Json) => c.name === name)?.bounds ?? null;
  return {
    header: pick("MainViewHeader"),
    input: pick("MainViewInput"),
    contextZone: pick("MainViewContextZone"),
    footer: pick("MainViewFooter"),
  };
}

function boundsEqual(a: Json | null, b: Json | null): boolean {
  if (!a || !b) return a === b;
  return a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height;
}

mkdirSync(dirname(receiptPath), { recursive: true });
const receipt: Json = {
  schemaVersion: 1,
  tool: "stability-cls-probe",
  binary: binary ?? "freshest local agent/dev binary",
  pass: false,
  failures: [],
  observations: [],
  scenarios: {},
};

const driver = await Driver.launch({
  ...(binary ? { binary } : {}),
  sandboxHome: true,
  sessionName: "stability-cls",
  readyTimeoutMs: 30_000,
  defaultTimeoutMs: 10_000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function getRows(): Promise<{ rows: RowInfo[]; raw: Json }> {
  const raw = await driver.getElements(
    { limit: 120, includeHeaders: true },
    { timeoutMs: 10_000 },
  );
  return { rows: rowsOf(raw), raw };
}

interface SurfaceProbe {
  promptType: string;
  surfaceKind: string;
  windowVisible: boolean;
}

// The main list reports promptType "none" (prompt_handler/mod.rs), so surface
// identity comes from surfaceContract.surfaceKind, not promptType.
async function surfaceProbe(): Promise<SurfaceProbe> {
  const state = await driver.getState({ timeoutMs: 10_000 });
  return {
    promptType: String(state.promptType ?? "unknown"),
    surfaceKind: String(state.surfaceContract?.surfaceKind ?? "unknown"),
    windowVisible: state.windowVisible === true,
  };
}

function onMainList(probe: SurfaceProbe): boolean {
  return probe.surfaceKind === "ScriptList";
}

async function waitForSurface(
  predicate: (probe: SurfaceProbe) => boolean,
  timeoutMs = 6_000,
): Promise<SurfaceProbe> {
  const start = performance.now();
  let current = await surfaceProbe();
  while (!predicate(current) && performance.now() - start < timeoutMs) {
    await Bun.sleep(80);
    current = await surfaceProbe();
  }
  return current;
}

/// Open a builtin the way a user does: type its name, submit the top row.
/// Returns the observed promptType, or null when the launcher ranked a
/// different row first (recorded as an observation, not a failure).
async function openBuiltinViaLauncher(hop: (typeof HOP_BUILTINS)[number]): Promise<string | null> {
  await driver.setFilterAndWait(hop.query, { timeoutMs: 5_000 });
  await driver.waitForSettle({ timeoutMs: 5_000 });
  const { rows } = await getRows();
  const first = rows.find((r) => r.selectable);
  // Guard against launching a lookalike (e.g. the macOS "System Settings"
  // app row): the target must be the built-in itself.
  if (!first || !first.semanticId.includes(hop.slug) || first.kind !== "built-in") {
    receipt.observations.push({
      name: "builtin_not_ranked_first",
      id: hop.id,
      query: hop.query,
      firstRow: first ?? null,
    });
    await driver.setFilterAndWait("", { timeoutMs: 5_000 });
    return null;
  }
  driver.simulateKey("enter");
  const probe = await waitForSurface((p) => p.promptType === hop.expectedPromptType);
  return probe.promptType;
}

function auditLeadingSeparator(tag: string, rows: RowInfo[]) {
  if (rows.length === 0) return;
  const first = rows[0];
  if (first.role !== "sectionHeader") {
    receipt.failures.push({
      name: "leading_separator_missing",
      scenario: tag,
      firstRow: first,
      contract: "POLISH.md §2: every list leads with a persistent section header",
    });
  }
}

/// Coherent main list: empty filter, row 0 is a header, selection sits on the
/// first selectable row.
async function auditMainCoherence(tag: string) {
  const state = await driver.getState({ timeoutMs: 10_000 });
  const inputValue = String(state.inputValue ?? "");
  if (inputValue !== "") {
    receipt.failures.push({ name: "main_filter_not_cleared", scenario: tag, inputValue });
  }
  const { rows } = await getRows();
  auditLeadingSeparator(tag, rows);
  const firstSelectable = rows.find((r) => r.selectable);
  const selected = rows.find((r) => r.selected);
  if (firstSelectable && selected && firstSelectable.semanticId !== selected.semanticId) {
    receipt.failures.push({
      name: "main_selection_not_on_first_row",
      scenario: tag,
      firstSelectable: firstSelectable.semanticId,
      selected: selected.semanticId,
    });
  }
  if (firstSelectable && !selected) {
    receipt.failures.push({ name: "main_selection_missing", scenario: tag });
  }
}

try {
  await driver.waitForSettle();

  // --- Scenario A + B: main-list query states -------------------------------
  const queryStates: Json[] = [];
  for (const query of MAIN_QUERY_STATES) {
    await driver.setFilterAndWait(query, { timeoutMs: 5_000 });
    const immediate = await getRows();
    await driver.waitForSettle({ timeoutMs: 5_000 });
    await Bun.sleep(350);
    const settled = await getRows();

    const tag = `main-query:${JSON.stringify(query)}`;
    auditLeadingSeparator(tag, immediate.rows);
    auditLeadingSeparator(`${tag}:settled`, settled.rows);

    // B: the immediate visible fold must survive settling untouched.
    const immediateSig = rowSignature(immediate.rows);
    const settledSig = rowSignature(settled.rows);
    const foldLen = Math.min(immediateSig.length, settledSig.length, FOLD_ROWS);
    const foldMutated =
      immediateSig.slice(0, foldLen).join("\n") !== settledSig.slice(0, foldLen).join("\n");
    const shrankBelowFold = settledSig.length < Math.min(immediateSig.length, FOLD_ROWS);
    if (foldMutated || shrankBelowFold) {
      receipt.failures.push({
        name: "late_republish_mutated_visible_fold",
        scenario: tag,
        immediate: immediateSig.slice(0, FOLD_ROWS),
        settled: settledSig.slice(0, FOLD_ROWS),
        shrankBelowFold,
        contract: "POLISH.md §2: late provider results must not republish/move visible rows",
      });
    }
    const selectedImmediate = immediate.rows.find((r) => r.selected)?.semanticId ?? null;
    const selectedSettled = settled.rows.find((r) => r.selected)?.semanticId ?? null;
    if (selectedImmediate && selectedSettled && selectedImmediate !== selectedSettled) {
      receipt.failures.push({
        name: "late_republish_moved_selection",
        scenario: tag,
        selectedImmediate,
        selectedSettled,
      });
    }
    queryStates.push({
      query,
      immediateRows: immediateSig.slice(0, FOLD_ROWS),
      settledRows: settledSig.slice(0, FOLD_ROWS),
      settledCount: settledSig.length,
    });
  }
  receipt.scenarios.mainQueryStates = queryStates;

  // Chrome baseline on the settled main menu.
  const baselineLayout = await driver.getLayoutInfo({}, { timeoutMs: 10_000 });
  const baselineChrome = chromeOf(baselineLayout);
  receipt.scenarios.baselineChrome = baselineChrome;

  // --- Scenario C + cross-view observations ---------------------------------
  const builtinReports: Json[] = [];
  for (const hop of HOP_BUILTINS) {
    const report: Json = { id: hop.id };
    const observedPromptType = await openBuiltinViaLauncher(hop);
    if (observedPromptType === null) {
      report.skipped = "not ranked first";
      builtinReports.push(report);
      continue;
    }
    await driver.waitForSettle({ timeoutMs: 5_000 });
    report.promptType = observedPromptType;
    if (observedPromptType !== hop.expectedPromptType) {
      receipt.failures.push({
        name: "builtin_did_not_open",
        id: hop.id,
        expected: hop.expectedPromptType,
        actual: observedPromptType,
      });
      // Try to recover to main for the next hop.
      for (let press = 0; press < 3; press += 1) driver.simulateKey("escape");
      await driver.waitForSettle({ timeoutMs: 5_000 });
      builtinReports.push(report);
      continue;
    }

    // Chrome bounds vs main baseline (computed model; divergence = declared shift).
    const layout = await driver.getLayoutInfo({}, { timeoutMs: 10_000 });
    const chrome = chromeOf(layout);
    const chromeDeltas: string[] = [];
    for (const key of ["header", "input", "contextZone", "footer"]) {
      if (!boundsEqual(chrome[key], baselineChrome[key])) {
        chromeDeltas.push(key);
      }
    }
    report.chromeDeltas = chromeDeltas;
    if (chromeDeltas.length > 0) {
      receipt.observations.push({
        name: "builtin_chrome_divergence",
        id: hop.id,
        deltas: chromeDeltas,
        chrome,
        baseline: baselineChrome,
      });
    }

    // Row structure inside the builtin: initial, filtered, zero-match.
    const initial = await getRows();
    report.initialRows = rowSignature(initial.rows).slice(0, 10);
    report.initialLeadingHeader = initial.rows[0]?.role === "sectionHeader";
    if (!report.initialLeadingHeader) {
      receipt.observations.push({
        name: "builtin_leading_header_not_observable",
        id: hop.id,
        firstRow: initial.rows[0] ?? null,
        note: "surface collector does not expose a leading section header; verify renderer + extend collector before asserting",
      });
    }

    await driver.setFilterAndWait("e", { timeoutMs: 5_000 });
    await driver.waitForSettle({ timeoutMs: 5_000 });
    const filtered = await getRows();
    report.filteredRows = rowSignature(filtered.rows).slice(0, 10);

    await driver.setFilterAndWait("zzqxvzz", { timeoutMs: 5_000 });
    await driver.waitForSettle({ timeoutMs: 5_000 });
    const zero = await getRows();
    report.zeroMatchRows = rowSignature(zero.rows).slice(0, 10);
    report.zeroMatchRowCount = zero.rows.length;
    if (zero.rows.length === 0) {
      receipt.observations.push({
        name: "builtin_zero_match_no_designed_row",
        id: hop.id,
        note: "no rows at zero-match; verify surface keeps list container + designed empty state (audit P1: list<->EmptyState swap)",
      });
    }

    // Escape back to main. Opened from the main menu, so Escape must return
    // there (first press may clear the typed filter).
    let back: SurfaceProbe = await surfaceProbe();
    let presses = 0;
    for (; presses < 3 && !onMainList(back); presses += 1) {
      driver.simulateKey("escape");
      back = await waitForSurface(onMainList, 2_500);
    }
    report.afterEscapeSurface = back;
    report.escapePresses = presses;
    builtinReports.push(report);
    if (!onMainList(back)) {
      receipt.failures.push({
        name: "escape_did_not_return_to_main",
        id: hop.id,
        surface: back,
      });
      continue;
    }
    // Two presses max: one clears the typed zero-match filter, one goes back.
    if (presses > 2) {
      receipt.failures.push({
        name: "escape_needed_extra_presses",
        id: hop.id,
        presses,
        contract: "POLISH.md §3: Escape unwinds exactly one layer per press",
      });
    }
    await driver.waitForSettle({ timeoutMs: 5_000 });
    await auditMainCoherence(`escape-back:${hop.id}`);
  }
  receipt.scenarios.builtins = builtinReports;

  // --- Scenario D: rapid hop at lightning speed ------------------------------
  for (let round = 0; round < 2; round += 1) {
    for (const hop of HOP_BUILTINS) {
      driver.setFilter(hop.query);
      await Bun.sleep(40);
      driver.simulateKey("enter");
      await Bun.sleep(60);
      driver.simulateKey("escape");
      await Bun.sleep(40);
      driver.simulateKey("escape");
      await Bun.sleep(40);
    }
  }
  await driver.waitForSettle({ timeoutMs: 8_000 });
  let finalSurface = await waitForSurface(onMainList, 4_000);
  for (let press = 0; press < 3 && !onMainList(finalSurface); press += 1) {
    driver.simulateKey("escape");
    finalSurface = await waitForSurface(onMainList, 2_500);
  }
  receipt.scenarios.rapidHopFinalSurface = finalSurface;
  if (!onMainList(finalSurface)) {
    receipt.failures.push({
      name: "rapid_hop_stranded_off_main",
      surface: finalSurface,
    });
  } else {
    await driver.setFilterAndWait("", { timeoutMs: 5_000 });
    await driver.waitForSettle({ timeoutMs: 5_000 });
    await auditMainCoherence("rapid-hop-final");
  }

  receipt.pass = receipt.failures.length === 0;
} catch (error) {
  receipt.failures.push({ name: "probe_error", error: String(error) });
} finally {
  await driver.close();
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
}

if (!receipt.pass) {
  console.error(JSON.stringify(receipt, null, 2));
  process.exit(1);
}

console.log(JSON.stringify(receipt, null, 2));
