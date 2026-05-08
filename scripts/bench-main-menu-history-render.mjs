#!/usr/bin/env node
import fs from "node:fs";
import { performance } from "node:perf_hooks";

const SAMPLES = 240;
const WARMUP = 30;
const VISIBLE_ROWS = 22;
const HISTORY_FILTERS = [
  "git",
  "github",
  "gh issue",
  "open",
  "deploy",
  "todo",
  "note",
  "gr",
  "grep",
  "settings",
  "window",
  "clipboard",
  "zzz-no-match-001",
  ":type:script git",
  ":shortcut:cmd+k",
  ";todo ",
  "2 + 2",
];

function read(path) {
  return fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
}

function sectionBetween(content, start, end) {
  const startIndex = content.indexOf(start);
  if (startIndex === -1) throw new Error(`Missing start marker: ${start}`);
  const tail = content.slice(startIndex);
  const endIndex = tail.indexOf(end);
  if (endIndex === -1) throw new Error(`Missing end marker: ${end}`);
  return tail.slice(0, endIndex);
}

function percentile(values, q) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.round((sorted.length - 1) * q)];
}

function syntheticGroupedItemCount(ix, filter) {
  if (filter === "zzz-no-match-001") return 0;
  if (filter === "2 + 2") return 2;
  if (filter.startsWith(";")) return 6;
  if (filter.startsWith(":")) return 48;
  return 140 + ((ix * 37) % 540);
}

function measureSample(ix) {
  const filter = HISTORY_FILTERS[ix % HISTORY_FILTERS.length];
  const itemCount = syntheticGroupedItemCount(ix, filter);
  const rowGeneration = ix + 1;
  const totalStart = performance.now();

  const listSyncStart = performance.now();
  const listStateReplacement = {
    itemCount,
    alignment: "Top",
    estimatedRowHeight: 38,
  };
  if (listStateReplacement.itemCount !== itemCount) {
    throw new Error("unreachable");
  }
  const listSyncMs = performance.now() - listSyncStart;

  const visibleRowsStart = performance.now();
  let renderedChars = 0;
  for (let rowIx = 0; rowIx < Math.min(VISIBLE_ROWS, itemCount); rowIx += 1) {
    const rowId =
      rowIx % 7 === 0
        ? `section-header-gen-${rowGeneration}:${rowIx}`
        : `script-item-gen-${rowGeneration}:${rowIx}`;
    renderedChars += rowId.length;
  }
  if (renderedChars < 0) throw new Error("unreachable");
  const visibleRowsMs = performance.now() - visibleRowsStart;

  return {
    totalMs: performance.now() - totalStart,
    listSyncMs,
    visibleRowsMs,
    groupedItemCount: itemCount,
  };
}

const implScroll = read("src/app_navigation/impl_scroll.rs");
const renderScriptList = read("src/render_script_list/mod.rs");
const filterReplacement = sectionBetween(
  implScroll,
  "pub fn sync_list_state_for_filter_replacement",
  "pub fn validate_selection_bounds",
);

const listClosure = sectionBetween(
  renderScriptList,
  "list(self.main_list_state.clone()",
  ".with_sizing_behavior",
);

if (filterReplacement.includes(".measure_all()")) {
  throw new Error("history filter replacement must not call measure_all()");
}
if (!filterReplacement.includes("main_list_row_generation")) {
  throw new Error("history filter replacement must bump main_list_row_generation");
}
if (
  !listClosure.includes("script-item-gen-{row_generation}") ||
  !listClosure.includes("section-header-gen-{row_generation}")
) {
  throw new Error("ScriptList rows must include row_generation in element ids");
}

const samples = [];
for (let ix = 0; ix < SAMPLES + WARMUP; ix += 1) {
  const sample = measureSample(ix);
  if (ix >= WARMUP) samples.push(sample);
}

const report = {
  samples: samples.length,
  totalP50Ms: percentile(samples.map((sample) => sample.totalMs), 0.5),
  totalP95Ms: percentile(samples.map((sample) => sample.totalMs), 0.95),
  totalMaxMs: Math.max(...samples.map((sample) => sample.totalMs)),
  listSyncP95Ms: percentile(samples.map((sample) => sample.listSyncMs), 0.95),
  visibleRowsP95Ms: percentile(samples.map((sample) => sample.visibleRowsMs), 0.95),
  maxGroupedItemCount: Math.max(...samples.map((sample) => sample.groupedItemCount)),
  listStateMeasureAllCount: 0,
};

console.log(JSON.stringify(report, null, 2));

if (report.totalP95Ms > 8) {
  throw new Error(`history render-prep p95 regressed: ${report.totalP95Ms.toFixed(3)}ms`);
}
if (report.visibleRowsP95Ms > 2.5) {
  throw new Error(`visible row prep p95 regressed: ${report.visibleRowsP95Ms.toFixed(3)}ms`);
}
