#!/usr/bin/env bun
// One-window keys probe: the header names what's focused and Space puts it in play; the footer's
// left corner names what just happened and ↑ puts it in play; ⌘K acts on the highlighted row;
// `?` opens help and a backtick opens the Day Page. Real GPUI key events into a sandboxed app.
// Receipt: one JSON summary on stdout, with each step's line, receipt, staged subject, and footer.
//
// usage: bun scripts/agentic/one-window-keys-probe.ts [--binary <path>]
import { Driver, type Json } from "../devtools/driver.ts";

const binaryFlag = process.argv.indexOf("--binary");
const binary = binaryFlag > 0 ? process.argv[binaryFlag + 1] : undefined;

type Snap = {
  prompt: unknown;
  visible: unknown;
  filter: unknown;
  placeholder: unknown;
  view: unknown;
  line: Json | null;
  recent: Json | null;
  staged: unknown;
  leftInfo: Json | null;
  popup: unknown;
};

const steps: { name: string; snap: Snap; checks: Record<string, boolean> }[] = [];
const driver = await Driver.launch({ sandboxHome: true, sessionName: "one-window-keys", ...(binary ? { binary } : {}) });

async function snap(): Promise<Snap> {
  const state = (await driver.getState()) as Json;
  const one = (state.oneWindow ?? {}) as Json;
  const footer = (state.activeFooter ?? {}) as Json;
  return {
    prompt: state.promptType,
    visible: state.windowVisible ?? state.isVisible ?? null,
    filter: state.inputValue,
    placeholder: one.placeholder ?? null,
    view: one.view ?? null,
    line: (one.line as Json) ?? null,
    recent: (one.recent as Json) ?? null,
    staged: one.staged ?? null,
    leftInfo: (footer.leftInfo as Json) ?? null,
    popup: state.activePopup ?? null,
  };
}

async function key(name: string, opts: { text?: string; modifiers?: string[] } = {}) {
  await driver.simulateGpuiKeyDown(name, opts);
  await driver.waitForSettle();
}

/** Bring the window back the way the hotkey does, after a surface or launch let it go. */
async function summon() {
  await driver.request({ type: "show" }).catch(() => {});
  await driver.waitForSettle();
}

async function step(name: string, checks: (s: Snap) => Record<string, boolean>) {
  const s = await snap();
  steps.push({ name, snap: s, checks: checks(s) });
  return s;
}

let fatal: string | undefined;
try {
  await summon();

  const home = await step("home", s => ({
    placeholderIsSearchOrAsk: s.placeholder === "Search or ask",
    lineIsFocused: s.line?.kind === "focused",
    lineHintIsSpace: JSON.stringify(s.line?.hints ?? []).includes("space"),
    noActCmdKHint: !JSON.stringify(s.line?.hints ?? []).includes("act on it"),
    nothingStaged: s.staged === null,
  }));
  const focusedTitle = home.line?.title as string | undefined;

  await key("space", { text: " " });
  await step("space-grabs-focused", s => ({
    inputStaysEmpty: s.filter === "" || s.filter === null || s.filter === undefined,
    stagedIsFocused: !focusedTitle || s.staged === focusedTitle,
    lineLit: !focusedTitle || s.line?.lit === true,
    lineHintIsLetGo: !focusedTitle || JSON.stringify(s.line?.hints ?? []).includes("esc"),
  }));

  await key("escape");
  await step("escape-lets-go", s => ({
    nothingStaged: s.staged === null,
    lineNotLit: !s.line?.lit,
  }));

  await summon();
  await key("down");
  await key("k", { modifiers: ["cmd"] });
  await step("cmd-k-acts-on-row", s => ({
    headerNotStaged: !s.line?.lit,
    receiptNotStaged: !s.recent?.lit,
    actionsOpened: s.popup !== null || s.staged !== null,
  }));
  await key("escape");
  await summon();

  await key("`", { text: "`" });
  await step("backtick-opens-day-page", s => ({
    dayPageOpen: s.prompt === "dayPage",
    backtickNotLeftInInput: s.filter !== "`",
  }));
  await key("escape");
  await summon();
  await step("back-home-after-day-page", s => ({ home: s.prompt === "none" && s.view === null }));

  await key("?", { text: "?" });
  await step("question-opens-help", s => ({
    leftTheMainList: s.prompt !== "none" || s.view !== null,
    questionNotLeftInInput: s.filter !== "?",
  }));
  await key("escape");
  await summon();
  await step("back-home-after-help", s => ({ home: s.prompt === "none" && s.view === null }));

  // Something just happened: open Finder from the list (it only brings Finder forward), then summon.
  await driver.setFilterAndWait("Finder");
  await key("enter");
  await summon();
  const afterRun = await step("after-running-a-row", s => ({
    receiptShows: s.recent !== null,
    footerLeftIsReceipt: (s.leftInfo?.action as string | undefined) === "recent",
  }));

  if (afterRun.recent) {
    const recentTitle = afterRun.recent.title as string;
    await key("up");
    await step("up-grabs-recent", s => ({
      stagedIsRecent: s.staged === recentTitle,
      receiptLit: s.recent?.lit === true,
      footerSelected: s.leftInfo?.selected === true,
      headerStillNamesFocused: !s.line?.lit,
    }));
    await key("escape");
    await step("escape-lets-recent-go", s => ({ nothingStaged: s.staged === null, receiptNotLit: !s.recent?.lit }));
  }

} catch (e) {
  fatal = String(e);
}
let closeNote: string | undefined;
try {
  await driver.close();
} catch (e) {
  closeNote = String(e);
}

const failed = steps.flatMap(s => Object.entries(s.checks).filter(([, ok]) => !ok).map(([check]) => `${s.name}.${check}`));
console.log(JSON.stringify({ ok: !fatal && failed.length === 0, fatal, closeNote, failed, steps }, null, 2));
process.exit(!fatal && failed.length === 0 ? 0 : 1);
