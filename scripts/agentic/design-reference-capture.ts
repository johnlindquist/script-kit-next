#!/usr/bin/env bun
// Capture clean, unannotated hiDPI reference screenshots of real app surfaces
// for the design/mockups contract (design/mockups/screens/<screen>/reference/).
// Read-only: launch → show → (open surface) → settle → captureScreenshot → close.
//
// Most launcher-backed fixtures use the real user home because they mirror
// live launcher content. Privacy-sensitive or strict-raster fixtures use a
// prepared disposable HOME instead; Agent Chat disables vibrancy and animated
// background effects so its pixels can be compared deterministically.
//
// Usage:
//   bun scripts/agentic/design-reference-capture.ts [--screen main|actions|confirm|clipboard] [out.png]
//
// --screen clipboard is the exception to "real user home": clipboard history
// holds private data, so it launches against a PREPARED fixture HOME (seeded
// sqlite + a catch-all extraSecretPatterns config so the monitor rejects every
// live text capture) and fails closed if any non-seeded row is visible.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Database } from "bun:sqlite";
import { Driver } from "../devtools/driver.ts";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname;

const args = process.argv.slice(2);
let screen = "main";
const rest: string[] = [];
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--screen") screen = args[++i] ?? "main";
  else rest.push(args[i]);
}

const DEFAULT_OUT: Record<string, string> = {
  main: "design/mockups/screens/main-menu/reference/main-menu-default@2x.png",
  actions: "design/mockups/screens/actions-dialog/reference/actions-dialog@2x.png",
  confirm: "design/mockups/screens/confirm-popup/reference/confirm-popup@2x.png",
  clipboard: "design/mockups/screens/clipboard-history/reference/clipboard-history@2x.png",
  notes: "design/mockups/screens/notes/reference/notes@2x.png",
  settings: "design/mockups/screens/settings/reference/settings@2x.png",
  arg: "design/mockups/screens/arg-prompt/reference/arg-prompt@2x.png",
  "agent-chat": "design/mockups/screens/agent-chat/reference/agent-chat@2x.png",
  "day-page": "design/mockups/screens/day-page/reference/day-page@2x.png",
  chat: "design/mockups/screens/chat-prompt/reference/chat-prompt@2x.png",
  terminal: "design/mockups/screens/terminal-prompt/reference/terminal-prompt@2x.png",
};
const CAPTURE_TARGET: Record<string, { type: string; kind: string }> = {
  main: { type: "kind", kind: "main" },
  actions: { type: "kind", kind: "actionsDialog" },
  confirm: { type: "kind", kind: "main" },
  clipboard: { type: "kind", kind: "main" },
  notes: { type: "kind", kind: "notes" },
  settings: { type: "kind", kind: "main" },
  arg: { type: "kind", kind: "main" },
  "agent-chat": { type: "kind", kind: "main" },
  "day-page": { type: "kind", kind: "main" },
  chat: { type: "kind", kind: "main" },
  terminal: { type: "kind", kind: "main" },
};

const DAY_PAGE_TZ = "America/Denver";

function dayPageDateStamp(): string {
  // YYYY-MM-DD in the pinned brain timezone.
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: DAY_PAGE_TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date());
  return parts; // en-CA yields YYYY-MM-DD
}

const DAY_PAGE_FIXTURE = [
  "# Friday · ship the Day Page mockup",
  "09:12 sketched the Day Page fixture and token list",
  "09:31 - [ ] wire day-page tokens into export_design_tokens #design",
  "10:02 [Script Kit](https://scriptkit.com) landing refresh notes",
  "09:47 [Clipboard entry](kit://clipboard-history?id=day-page-mockup-seed)",
  "",
].join("\n");

// Must equal design/mockups/screens/notes/index.html content exactly (8 lines).
const NOTES_FIXTURE_MARKDOWN = [
  "# Design Contract Notes",
  "",
  "Track every painted value in",
  "the Notes window.",
  "",
  "- Titlebar 36 px",
  "- Editor 16 px mono, 20 px line",
  "- Footer buttons hug",
].join("\n");

// Deterministic clipboard fixture rows. Newest first; one pinned. Content is
// mirrored by design/mockups/screens/clipboard-history/index.html — keep in sync.
const CLIPBOARD_FIXTURE = [
  { id: "fx-tokens", content: "Design tokens stay in sync with the Rust renderer.", type: "text", ageSec: 60, pinned: 0 },
  { id: "fx-notes", content: "Meeting notes — Q3 launch\n- pixel-perfect mockups\n- token exporter\n- publish gallery", type: "text", ageSec: 300, pinned: 0 },
  { id: "fx-url", content: "https://scriptkit.com/downloads", type: "link", ageSec: 900, pinned: 0 },
  { id: "fx-code", content: "const answer = 42;", type: "text", ageSec: 1800, pinned: 0 },
  { id: "fx-pin", content: "npm install -g mdflow@next", type: "text", ageSec: 3600, pinned: 1 },
  { id: "fx-color", content: "#FBBF24", type: "color", ageSec: 7200, pinned: 0 },
];

function seedClipboardFixtureHome(): string {
  const home = mkdtempSync(join(tmpdir(), "design-capture-clipboard-"));
  const kitDir = join(home, ".scriptkit");
  const dbDir = join(kitDir, "db");
  mkdirSync(dbDir, { recursive: true });
  // Catch-all extra secret pattern: the monitor's first poll always captures
  // the CURRENT pasteboard (change_detection.rs first-call semantics), which
  // would leak real user clipboard into the reference. `(?s).` rejects every
  // live text capture; seeded rows below are the only text content.
  writeFileSync(
    join(kitDir, "config.ts"),
    'export default { clipboardHistorySecretRejection: { extraSecretPatterns: ["(?s)."] } };\n',
  );
  const db = new Database(join(dbDir, "clipboard-history.sqlite"));
  db.run(`CREATE TABLE IF NOT EXISTS history (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    content_hash TEXT,
    content_type TEXT NOT NULL DEFAULT 'text',
    timestamp INTEGER NOT NULL,
    pinned INTEGER DEFAULT 0,
    ocr_text TEXT,
    text_preview TEXT,
    image_width INTEGER,
    image_height INTEGER,
    byte_size INTEGER
  )`);
  const now = Date.now();
  const insert = db.prepare(
    `INSERT INTO history (id, content, content_hash, content_type, timestamp, pinned, text_preview, byte_size)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
  );
  for (const row of CLIPBOARD_FIXTURE) {
    insert.run(
      row.id,
      row.content,
      `fixture-${row.id}`,
      row.type,
      now - row.ageSec * 1000,
      row.pinned,
      row.content.split("\n")[0],
      row.content.length,
    );
  }
  db.close();
  return home;
}
if (!(screen in DEFAULT_OUT)) {
  console.error(`unknown --screen ${screen}; known: ${Object.keys(DEFAULT_OUT).join(", ")}`);
  process.exit(2);
}
const OUT = rest[0] ?? join(PROJECT_ROOT, DEFAULT_OUT[screen]);

async function main() {
  // Agent Chat has a stronger contract than the generic screenshot path: its
  // state, paint bounds, rendered-frame generation, and two OS captures must
  // all agree before the canonical PNG is replaced.
  if (screen === "agent-chat") {
    const canonicalOut = join(PROJECT_ROOT, DEFAULT_OUT["agent-chat"]);
    const delegatedReceipt = resolve(OUT) === resolve(canonicalOut)
      ? join(
        PROJECT_ROOT,
        "design/mockups/screens/agent-chat/reference/agent-chat-runtime-receipt.json",
      )
      : `${OUT}.receipt.json`;
    const delegated = Bun.spawnSync(
      ["bun", join(PROJECT_ROOT, "scripts/agentic/agent-chat-design-reference-receipt.ts")],
      {
        cwd: PROJECT_ROOT,
        env: {
          ...process.env,
          PROBE_SCREENSHOT: OUT,
          PROBE_RECEIPT: delegatedReceipt,
        },
        stdout: "inherit",
        stderr: "inherit",
      },
    );
    if (delegated.exitCode !== 0) process.exitCode = delegated.exitCode;
    return;
  }

  // clipboard: prepared fixture HOME (never the real one — privacy). The
  // driver wipes its sessionDir at launch, so the fixture home lives outside
  // it and is wired via env instead of sandboxHome.
  const fixtureHome = screen === "clipboard" ? seedClipboardFixtureHome() : null;
  // notes: sandbox DB env pins default 350×280 bounds (window_ops.rs:672-674)
  // and keeps the capture off the real notes database.
  const notesDbPath =
    screen === "notes"
      ? join(mkdtempSync(join(tmpdir(), "design-capture-notes-")), "notes.sqlite")
      : null;
  const driver = await Driver.launch({
    sessionName: `design-reference-capture-${screen}`,
    // settings/day-page: sandbox HOME gives deterministic default-config
    // content (settings census, seeded day file).
    sandboxHome: screen === "settings" || screen === "day-page",
    defaultTimeoutMs: 10_000,
    ...(fixtureHome
      ? { env: { HOME: fixtureHome, SK_PATH: join(fixtureHome, ".scriptkit") } }
      : {}),
    ...(notesDbPath ? { env: { SCRIPT_KIT_TEST_NOTES_DB_PATH: notesDbPath } } : {}),
    ...(screen === "day-page" ? { env: { SCRIPT_KIT_BRAIN_TZ: DAY_PAGE_TZ } } : {}),
  });
  try {
    await driver.request({ type: "show" }, { timeoutMs: 2_000 }).catch(() => {});
    await driver.waitForSettle();
    // Extra settle so first-frame async sections (brain inbox, flows) land.
    await Bun.sleep(1_200);

    if (screen === "actions") {
      await driver.request({
        type: "batch",
        commands: [{ type: "openActions" }],
      } as never, { timeoutMs: 5_000 }).catch(() => {});
      await driver.waitForSettle();
      await Bun.sleep(600);
    }

    if (screen === "clipboard") {
      // triggerBuiltin is fire-and-forget (no response envelope).
      driver.send({ type: "triggerBuiltin", name: "clipboard-history" } as never);
      await driver.waitForSettle();
      await Bun.sleep(600);

      // Fail closed: every visible choice row must come from the seeded fixture.
      const elements = (await driver.getElements()) as {
        elements?: Array<{ type?: string; text?: string; semanticId?: string }>;
      };
      const seededPreviews = CLIPBOARD_FIXTURE.map((row) => row.content.split("\n")[0]);
      const rows = (elements.elements ?? []).filter((el) => el.type === "choice");
      const foreign = rows.filter(
        (el) => !seededPreviews.some((preview) => (el.text ?? "").includes(preview)),
      );
      if (rows.length !== CLIPBOARD_FIXTURE.length || foreign.length > 0) {
        console.error(
          JSON.stringify({
            receipt: "design-reference-capture",
            screen,
            error: "clipboard rows do not exactly match the seeded fixture — refusing to capture",
            rowCount: rows.length,
            expected: CLIPBOARD_FIXTURE.length,
            foreigntexts: foreign.map((el) => el.text ?? "<untexted>"),
          }),
        );
        process.exitCode = 1;
        return;
      }
    }

    if (screen === "terminal") {
      // Deterministic PTY content: fixed banner + ANSI palette sample, then
      // hold the PTY open through the capture window.
      driver.send({
        type: "term",
        id: "design-term-fixture",
        command:
          "clear; printf 'SCRIPT KIT TERMINAL FIXTURE\\n'; printf 'ansi \\033[31mred\\033[0m \\033[32mgreen\\033[0m \\033[33myellow\\033[0m \\033[34mblue\\033[0m \\033[1mbold\\033[0m\\n'; printf '$ '; sleep 60",
      } as never);
      await driver.waitForSettle();
      await Bun.sleep(1_200);
    }

    if (screen === "chat") {
      // Fresh launches start Mini; close_and_reset flips to Full so the chat
      // opens at DivPrompt height (750×500) with full chrome.
      driver.send({ type: "simulateKey", key: "escape" } as never);
      await driver.waitForSettle();
      await driver.request({ type: "show" }, { timeoutMs: 2_000 }).catch(() => {});
      await driver.waitForSettle();
      driver.send({
        type: "chat",
        id: "design-chat-fixture",
        placeholder: "Ask follow-up...",
        saveHistory: false,
        useBuiltinAi: false,
        messages: [
          { role: "user", content: "How do I read the clipboard in a script?" },
          {
            role: "assistant",
            content:
              "Use the SDK clipboard helper, then show it in a prompt:\n\n```ts\nconst text = await clipboard.readText();\nawait div(md(text));\n```\n\nCall `clipboard.writeText(...)` to write back.",
          },
        ],
      } as never);
      await driver.waitForSettle();
      await Bun.sleep(600);
    }

    if (screen === "day-page") {
      // Seed the sandbox day file, then open via the real hold gesture.
      // Shelf stays COLLAPSED (rest state): there is no element-id click
      // primitive; expanding would need simulateGpuiEvent coordinates.
      const dayFile = join(
        driver.sessionDir,
        "home/.scriptkit/brain/days",
        `${dayPageDateStamp()}.md`,
      );
      mkdirSync(join(driver.sessionDir, "home/.scriptkit/brain/days"), { recursive: true });
      writeFileSync(dayFile, DAY_PAGE_FIXTURE);
      const { openDayPage } = await import("./day-page-open-helper.ts");
      await openDayPage(driver, `design-ref-${Math.floor(performance.now()).toString(36)}`);
      await driver.waitForSettle();
      await Bun.sleep(600);
    }

    if (screen === "arg") {
      // Deterministic arg prompt via the stdin protocol Message fallback.
      driver.send({
        type: "arg",
        id: "design-arg-fixture",
        placeholder: "Pick a fruit",
        choices: [
          { name: "Apple", value: "apple", description: "Crisp and sweet — the default pick" },
          { name: "Banana", value: "banana" },
          { name: "Cherry", value: "cherry" },
          { name: "Dragonfruit", value: "dragonfruit" },
          { name: "Elderberry", value: "elderberry" },
          { name: "Fig", value: "fig" },
        ],
        actions: [
          {
            name: "Inspect Fruit",
            description: "Design fixture action",
            value: "inspect",
            hasAction: false,
          },
        ],
      } as never);
      await driver.waitForSettle();
      await Bun.sleep(600);
    }

    if (screen === "settings") {
      driver.send({ type: "triggerBuiltin", name: "settings" } as never);
      await driver.waitForSettle();
      await Bun.sleep(600);
    }

    if (screen === "confirm") {
      // Deterministic fixture matching the canonical stdin_commands test JSON.
      await driver.request({
        type: "openConfirmPrompt",
        title: "Delete?",
        body: "This cannot be undone.",
        confirmText: "Delete",
        cancelText: "Keep",
      } as never, { timeoutMs: 5_000 }).catch(() => {});
      await driver.waitForSettle();
      await Bun.sleep(600);
    }

    if (screen === "notes") {
      driver.send({ type: "openNotes" } as never);
      const notesTarget = { type: "kind", kind: "notes", index: 0 };
      // Wait until the notes window is targetable.
      let layout: { error?: string; windowBounds?: { height?: number } } = { error: "pending" };
      for (let attempt = 0; attempt < 40; attempt++) {
        layout = (await driver
          .request({ type: "getLayoutInfo", target: notesTarget } as never, { timeoutMs: 2_000 })
          .catch(() => ({ error: "timeout" }))) as typeof layout;
        if (!layout.error) break;
        await Bun.sleep(250);
      }
      if (layout.error) throw new Error(`notes window never became targetable: ${layout.error}`);
      await driver.request(
        {
          type: "batch",
          target: notesTarget,
          commands: [{ type: "setInput", text: NOTES_FIXTURE_MARKDOWN }],
          options: { stopOnError: true, timeout: 8_000 },
        } as never,
        { timeoutMs: 10_000 },
      );
      // Autosize settle: two consecutive equal window heights.
      let lastHeight = -1;
      for (let attempt = 0; attempt < 20; attempt++) {
        const info = (await driver
          .request({ type: "getLayoutInfo", target: notesTarget } as never, { timeoutMs: 2_000 })
          .catch(() => null)) as { windowBounds?: { height?: number } } | null;
        const height = info?.windowBounds?.height ?? -2;
        if (height === lastHeight && height > 0) break;
        lastHeight = height;
        await Bun.sleep(250);
      }
      // Let the 1500ms saved-flash "✓" clear before capturing the rest state.
      await Bun.sleep(1_700);
    }

    const state = (await driver.getState()) as {
      currentView?: string;
      windowVisible?: boolean;
      actionsDialog?: unknown;
    };
    const shot = (await driver.captureScreenshot({
      hiDpi: true,
      target: CAPTURE_TARGET[screen],
      savePath: OUT,
    })) as { error?: string; width?: number; height?: number };

    console.log(
      JSON.stringify(
        {
          receipt: "design-reference-capture",
          screen,
          out: OUT,
          windowVisible: state.windowVisible ?? null,
          currentView: state.currentView ?? null,
          actionsDialogOpen: screen === "actions" ? Boolean(state.actionsDialog) : undefined,
          width: shot.width ?? null,
          height: shot.height ?? null,
          error: shot.error ?? null,
        },
        null,
        2,
      ),
    );
    if (shot.error) process.exitCode = 1;
  } finally {
    await driver.close();
  }
}

await main();
