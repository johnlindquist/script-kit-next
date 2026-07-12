#!/usr/bin/env bun
/**
 * Story fidelity proof — verify design/mockups/stories/* line up with the
 * REAL app driven through the same user paths via script-kit-devtools.
 *
 * Per story checkpoint:
 *   app side    Driver → drive to the checkpoint state → captureScreenshot
 *               (@2x, GPUI window) + getState/getElements/getLayoutInfo
 *   mockup side agent-browser → http://127.0.0.1:<port>/stories/<id>/
 *               index.html?autoplay=0&t=<ms> → freeze animations →
 *               screenshot the active surface iframe (DPR 2)
 *   gates       1. physical dimensions equal (hard)
 *               2. semantic parity: search text / selected row (hard)
 *               3. content-region pixel diff receipt (footer band cropped —
 *                  the native AppKit footer never paints into GPUI captures)
 *
 * Receipts land in .test-output/story-fidelity/<story>/<chapter>/ and a
 * summary at .test-output/story-fidelity/summary.json.
 *
 * Usage:
 *   bun scripts/agentic/story-fidelity-proof.ts               # all stories
 *   bun scripts/agentic/story-fidelity-proof.ts 01 03         # by prefix
 *   bun scripts/agentic/story-fidelity-proof.ts 01 --chapter arg,pick
 */
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Database } from "bun:sqlite";
import { Driver } from "../devtools/driver.ts";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname.replace(/\/$/, "");
const MOCKUPS_ROOT = join(PROJECT_ROOT, "design/mockups");
const OUT_ROOT = join(PROJECT_ROOT, ".test-output/story-fidelity");

// ─── CLI ────────────────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
const storyFilters: string[] = [];
let chapterFilter: string[] | null = null;
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--chapter") chapterFilter = (argv[++i] ?? "").split(",").filter(Boolean);
  else storyFilters.push(argv[i]);
}

// ─── Mockup HTTP server (file:// iframes are cross-origin in Chromium) ─────
function serveMockups(): { port: number; stop: () => void } {
  const server = Bun.serve({
    port: 0,
    async fetch(req) {
      const url = new URL(req.url);
      let path = decodeURIComponent(url.pathname);
      if (path.endsWith("/")) path += "index.html";
      const file = Bun.file(join(MOCKUPS_ROOT, path));
      if (!(await file.exists())) return new Response("not found", { status: 404 });
      return new Response(file);
    },
  });
  return { port: server.port, stop: () => server.stop(true) };
}

// ─── agent-browser helpers ──────────────────────────────────────────────────
const BROWSER_SESSION = `story-fidelity-${process.pid}`;

async function browser(args: string[]): Promise<string> {
  const proc = Bun.spawn(["agent-browser", "--session", BROWSER_SESSION, ...args], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  if (code !== 0) throw new Error(`agent-browser ${args[0]} exited ${code}: ${stderr || stdout}`);
  return stdout.trim();
}

async function browserEval(js: string): Promise<any> {
  const raw = await browser(["eval", js]);
  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
}

/** Freeze CSS animation/transition state in the top doc and every iframe. */
const FREEZE_JS = String.raw`(() => {
  const freeze = (doc) => {
    if (!doc || doc.getElementById("sk-fidelity-freeze")) return;
    const s = doc.createElement("style");
    s.id = "sk-fidelity-freeze";
    s.textContent = "*{animation:none !important;transition:none !important}";
    (doc.head || doc.documentElement).appendChild(s);
  };
  freeze(document);
  document.querySelectorAll("iframe").forEach((f) => {
    try { freeze(f.contentDocument); } catch (_) {}
  });
  return "frozen";
})()`;

/** Collect story digest + active-surface anatomy geometry (iframe-relative CSS px). */
const COLLECT_JS = String.raw`(() => {
  const api = window.__SK_STORY__;
  if (!api) throw new Error("story api missing");
  const digest = api.getSemanticDigest();
  const iframes = Array.from(document.querySelectorAll("iframe[data-story-surface]"));
  const active = iframes.filter((f) => !f.hidden);
  const out = { digest, surfaces: {} };
  for (const frame of active) {
    const id = frame.getAttribute("data-story-surface");
    let doc = null;
    try { doc = frame.contentDocument; } catch (_) {}
    if (!doc) { out.surfaces[id] = { error: "no-doc" }; continue; }
    const frameRect = frame.getBoundingClientRect();
    const rel = (el) => {
      if (!el) return null;
      const r = el.getBoundingClientRect();
      return { x: r.x, y: r.y, w: r.width, h: r.height };
    };
    const win = doc.querySelector(".sk-window") || doc.getElementById("window");
    const selRow = doc.querySelector('.sk-list-row[data-state="selected"]');
    const selName = selRow ? selRow.querySelector(".sk-list-row__name") : null;
    const search = doc.querySelector(".sk-search-text, .sk-arg-text, .sk-chat-input-text, .sk-agent-chat-composer__text");
    const visibleRows = Array.from(doc.querySelectorAll(".sk-list-row")).filter(
      (r) => !r.hidden && r.getAttribute("data-story-hidden") !== "true"
    );
    out.surfaces[id] = {
      frame: { x: frameRect.x, y: frameRect.y, w: frameRect.width, h: frameRect.height },
      window: rel(win),
      header: rel(doc.querySelector(".sk-header, .sk-arg-header, .sk-notes-titlebar")),
      list: rel(doc.querySelector(".sk-list, .sk-clipboard-rows")),
      footer: rel(doc.querySelector(".sk-footer-host, .sk-footer-rail")),
      selectedRow: rel(selRow),
      selectedName: selName ? selName.textContent.trim() : null,
      searchText: search ? search.textContent : "",
      footerButtons: Array.from(doc.querySelectorAll(".sk-footer-rail .sk-footer-action"))
        .filter((b) => !b.hidden)
        .map((b) => {
          const label = b.querySelector(".sk-footer-label");
          const keys = Array.from(b.querySelectorAll(".sk-keycap")).map((k) => k.textContent.trim());
          return { label: label ? label.textContent.trim() : "", key: keys.join("") };
        }),
      visibleRowNames: visibleRows.map((r) => {
        const n = r.querySelector(".sk-list-row__name");
        return n ? n.textContent.trim() : "";
      }),
    };
  }
  return JSON.stringify(out);
})()`;

async function captureMockup(opts: {
  port: number;
  storyId: string;
  surface: string;
  t: number;
  outPng: string;
}): Promise<any> {
  const url = `http://127.0.0.1:${opts.port}/stories/${opts.storyId}/index.html?autoplay=0&t=${opts.t}`;
  await browser(["open", url]);
  // Wait for story api + iframe readiness, then deterministic seek.
  for (let attempt = 0; attempt < 40; attempt++) {
    const ok = await browserEval(
      `(() => Boolean(window.__SK_STORY__) && Array.from(document.querySelectorAll("iframe[data-story-surface]")).every((f) => { try { return f.contentDocument && f.contentDocument.readyState === "complete"; } catch (_) { return false; } }))()`,
    );
    if (ok === true || ok === "true") break;
    await Bun.sleep(150);
  }
  await browserEval(`window.__SK_STORY__.pause(), window.__SK_STORY__.seek(${opts.t}), "ok"`);
  await Bun.sleep(120);
  await browserEval(FREEZE_JS);
  const receiptRaw = await browserEval(COLLECT_JS);
  const receipt = typeof receiptRaw === "string" ? JSON.parse(receiptRaw) : receiptRaw;
  await browser(["screenshot", `[data-story-surface="${opts.surface}"]`, opts.outPng]);
  return receipt;
}

// ─── image helpers ──────────────────────────────────────────────────────────
function pngDims(path: string): { w: number; h: number } {
  const b = readFileSync(path);
  return { w: b.readUInt32BE(16), h: b.readUInt32BE(20) };
}

async function imageDiff(opts: {
  app: string;
  mock: string;
  out: string;
  receiptOut: string;
  label: string;
  cropBottomPt?: number; // logical pt to trim from the bottom of BOTH images
}): Promise<any> {
  const a = pngDims(opts.app);
  const cropH = opts.cropBottomPt ? Math.max(0, a.h - Math.round(opts.cropBottomPt * 2)) : a.h;
  const m = pngDims(opts.mock);
  const cropArgs = opts.cropBottomPt
    ? [
        "--red-crop",
        `${a.w}x${cropH}+0+0`,
        "--green-crop",
        `${m.w}x${Math.max(0, m.h - Math.round(opts.cropBottomPt * 2))}+0+0`,
      ]
    : [];
  const proc = Bun.spawn(
    [
      "bun",
      join(PROJECT_ROOT, "scripts/devtools/image-diff.ts"),
      "compare",
      "--red",
      opts.app,
      "--green",
      opts.mock,
      "--out",
      opts.out,
      "--receipt-out",
      opts.receiptOut,
      "--label",
      opts.label,
      "--fuzz",
      "12%",
      ...cropArgs,
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  await proc.exited;
  try {
    return JSON.parse(readFileSync(opts.receiptOut, "utf8"));
  } catch {
    return { error: "diff-receipt-missing" };
  }
}

// ─── fixtures (mirrors design-reference-capture.ts — keep in sync) ─────────
const ARG_FIXTURE = {
  type: "arg",
  id: "story-fidelity-arg",
  placeholder: "Pick a fruit",
  choices: [
    { name: "Apple", value: "apple", description: "Crisp and sweet — the default pick" },
    { name: "Banana", value: "banana" },
    { name: "Cherry", value: "cherry" },
    { name: "Dragonfruit", value: "dragonfruit" },
    { name: "Elderberry", value: "elderberry" },
    { name: "Fig", value: "fig" },
  ],
};

const CLIPBOARD_FIXTURE = [
  { id: "fx-tokens", content: "Design tokens stay in sync with the Rust renderer.", type: "text", ageSec: 60, pinned: 0 },
  { id: "fx-notes", content: "Meeting notes — Q3 launch\n- pixel-perfect mockups\n- token exporter\n- publish gallery", type: "text", ageSec: 300, pinned: 0 },
  { id: "fx-url", content: "https://scriptkit.com/downloads", type: "link", ageSec: 900, pinned: 0 },
  { id: "fx-code", content: "const answer = 42;", type: "text", ageSec: 1800, pinned: 0 },
  { id: "fx-pin", content: "npm install -g mdflow@next", type: "text", ageSec: 3600, pinned: 1 },
  { id: "fx-color", content: "#FBBF24", type: "color", ageSec: 7200, pinned: 0 },
];

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

const DAY_PAGE_TZ = "America/Denver";
const DAY_PAGE_LINES = [
  "# Friday · ship the Day Page mockup",
  "09:12 sketched the Day Page fixture and token list",
  "09:31 - [ ] wire day-page tokens into export_design_tokens #design",
  "10:02 [Script Kit](https://scriptkit.com) landing refresh notes",
  "09:47 [Clipboard entry](kit://clipboard-history?id=day-page-mockup-seed)",
];

function dayPageDateStamp(): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: DAY_PAGE_TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date());
}

function seedClipboardHome(): string {
  const home = mkdtempSync(join(tmpdir(), "story-fidelity-clipboard-"));
  const kitDir = join(home, ".scriptkit");
  mkdirSync(join(kitDir, "db"), { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    'export default { clipboardHistorySecretRejection: { extraSecretPatterns: ["(?s)."] } };\n',
  );
  const db = new Database(join(kitDir, "db", "clipboard-history.sqlite"));
  db.run(`CREATE TABLE IF NOT EXISTS history (
    id TEXT PRIMARY KEY, content TEXT NOT NULL, content_hash TEXT,
    content_type TEXT NOT NULL DEFAULT 'text', timestamp INTEGER NOT NULL,
    pinned INTEGER DEFAULT 0, ocr_text TEXT, text_preview TEXT,
    image_width INTEGER, image_height INTEGER, byte_size INTEGER)`);
  const now = Date.now();
  const insert = db.prepare(
    `INSERT INTO history (id, content, content_hash, content_type, timestamp, pinned, text_preview, byte_size)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
  );
  for (const row of CLIPBOARD_FIXTURE) {
    insert.run(row.id, row.content, `fixture-${row.id}`, row.type, now - row.ageSec * 1000, row.pinned, row.content.split("\n")[0], row.content.length);
  }
  db.close();
  return home;
}

/** Sandbox-ish home with launcher scripts the story timeline expects. */
function seedLauncherHome(): string {
  const home = mkdtempSync(join(tmpdir(), "story-fidelity-launcher-"));
  const scripts = join(home, ".scriptkit", "plugins", "main", "scripts");
  mkdirSync(scripts, { recursive: true });
  writeFileSync(
    join(scripts, "fruit-picker.ts"),
    `// Name: Fruit Picker\n// Description: arg prompt demo · pick a fruit\nexport {};\n`,
  );
  writeFileSync(
    join(scripts, "framework-docs.ts"),
    `// Name: Framework Docs\n// Description: open kit docs\nexport {};\n`,
  );
  return home;
}

// ─── checkpoint matrix ──────────────────────────────────────────────────────
type Checkpoint = {
  chapter: string;
  t: number;
  surface: string;
  /** logical pt trimmed from the bottom before the content pixel diff
   *  (native AppKit footer band — not painted in GPUI captures) */
  footerBandPt?: number;
  /** expectations against the mockup digest / anatomy */
  expect?: { search?: string; selectedName?: string };
  /** hard-gate pixel dims app vs mockup */
  gateDims?: boolean;
  drive: (driver: Driver, ctx: StoryCtx) => Promise<Record<string, unknown> | void>;
  notes?: string;
};

type StoryDef = {
  id: string;
  launch: () => Promise<{ driver: Driver; ctx: StoryCtx }>;
  checkpoints: Checkpoint[];
};

type StoryCtx = { home?: string; extra?: Record<string, unknown> };

async function launchDriver(opts: {
  session: string;
  sandboxHome?: boolean;
  env?: Record<string, string>;
}): Promise<Driver> {
  return Driver.launch({
    sessionName: opts.session,
    sandboxHome: opts.sandboxHome ?? false,
    defaultTimeoutMs: 12_000,
    ...(opts.env ? { env: opts.env } : {}),
  });
}

async function showAndSettle(driver: Driver) {
  await driver.request({ type: "show" }, { timeoutMs: 2_000 }).catch(() => {});
  await driver.waitForSettle();
  await Bun.sleep(1_200);
}

const STORIES: StoryDef[] = [
  {
    id: "01-run-script-with-arg",
    launch: async () => {
      const home = seedLauncherHome();
      const driver = await launchDriver({
        session: "story-fid-01",
        env: { HOME: home, SK_PATH: join(home, ".scriptkit") },
      });
      await showAndSettle(driver);
      return { driver, ctx: { home } };
    },
    checkpoints: [
      {
        chapter: "rest",
        t: 0,
        surface: "main-menu",
        footerBandPt: 32,
        gateDims: true,
        notes: "launcher content is home-dependent; dims + anatomy only",
        drive: async () => {},
      },
      {
        chapter: "narrow",
        t: 2600,
        surface: "main-menu",
        footerBandPt: 32,
        gateDims: true,
        expect: { search: "fruit", selectedName: "Fruit Picker" },
        drive: async (driver) => {
          await driver.setFilterAndWait("fruit");
          await driver.waitForSettle();
        },
      },
      {
        chapter: "arg",
        t: 4500,
        surface: "arg-prompt",
        footerBandPt: 32,
        gateDims: true,
        expect: { selectedName: "Apple" },
        drive: async (driver) => {
          await driver.setFilterAndWait("");
          driver.send(ARG_FIXTURE as never);
          await driver.waitForSettle();
          await Bun.sleep(600);
        },
      },
      {
        chapter: "pick",
        t: 6900,
        surface: "arg-prompt",
        footerBandPt: 32,
        gateDims: true,
        expect: { selectedName: "Cherry" },
        drive: async (driver) => {
          driver.simulateKey("down");
          driver.simulateKey("down");
          await driver.waitForSettle();
        },
      },
    ],
  },
  {
    id: "03-clipboard-paste",
    launch: async () => {
      const home = seedClipboardHome();
      const driver = await launchDriver({
        session: "story-fid-03",
        env: { HOME: home, SK_PATH: join(home, ".scriptkit") },
      });
      await showAndSettle(driver);
      driver.send({ type: "triggerBuiltin", name: "clipboard-history" } as never);
      await driver.waitForSettle();
      await Bun.sleep(600);
      return { driver, ctx: { home } };
    },
    checkpoints: [
      {
        chapter: "open",
        t: 300,
        surface: "clipboard-history",
        footerBandPt: 32,
        gateDims: true,
        expect: { selectedName: "Design tokens stay in sync" },
        drive: async () => {},
      },
      {
        chapter: "pin",
        t: 3600,
        surface: "clipboard-history",
        footerBandPt: 32,
        gateDims: true,
        expect: { selectedName: "npm install -g mdflow@next" },
        drive: async (driver) => {
          for (let i = 0; i < 4; i++) driver.simulateKey("down");
          await driver.waitForSettle();
        },
      },
    ],
  },
  {
    id: "06-settings-theme",
    launch: async () => {
      const driver = await launchDriver({ session: "story-fid-06", sandboxHome: true });
      await showAndSettle(driver);
      driver.send({ type: "triggerBuiltin", name: "settings" } as never);
      await driver.waitForSettle();
      await Bun.sleep(600);
      return { driver, ctx: {} };
    },
    checkpoints: [
      {
        chapter: "open",
        t: 300,
        surface: "settings",
        footerBandPt: 32,
        gateDims: true,
        drive: async () => {},
      },
      {
        chapter: "select",
        t: 3000,
        surface: "settings",
        footerBandPt: 32,
        gateDims: true,
        expect: { search: "theme", selectedName: "Theme Designer" },
        drive: async (driver) => {
          await driver.setFilterAndWait("theme");
          await driver.waitForSettle();
        },
      },
    ],
  },
];

// ─── semantic extraction from the app ───────────────────────────────────────
async function appSemantics(driver: Driver): Promise<Record<string, unknown>> {
  const state = (await driver.getState().catch(() => ({}))) as Record<string, any>;
  const elements = (await driver.getElements().catch(() => ({}))) as Record<string, any>;
  const rows = ((elements.elements ?? []) as Array<Record<string, any>>).filter(
    (el) => el.type === "choice",
  );
  const footerButtons = (state.activeFooter?.buttons ?? []).map((b: any) => ({
    action: b.action,
    label: b.label,
    key: b.key,
  }));
  return {
    promptType: state.promptType ?? null,
    surfaceKind: state.surfaceContract?.surfaceKind ?? null,
    filter: state.inputValue ?? null,
    selectedIndex: state.selectedIndex ?? null,
    selectedRowText: state.selectedValue ?? null,
    visibleChoiceCount: state.visibleChoiceCount ?? null,
    footerButtons,
    visibleRowTexts: rows.map((r) => r.text ?? ""),
  };
}

// ─── main loop ──────────────────────────────────────────────────────────────
async function runStory(story: StoryDef, port: number, summary: any[]) {
  console.error(`\n━━ ${story.id} ━━`);
  const { driver, ctx } = await story.launch();
  try {
    for (const cp of story.checkpoints) {
      if (chapterFilter && !chapterFilter.includes(cp.chapter)) continue;
      const dir = join(OUT_ROOT, story.id, cp.chapter);
      mkdirSync(dir, { recursive: true });
      const appPng = join(dir, "app@2x.png");
      const mockPng = join(dir, "mock@2x.png");

      const driveReceipt = (await cp.drive(driver, ctx)) ?? {};
      await driver.waitForSettle();
      const shot = (await driver.captureScreenshot({
        hiDpi: true,
        target: { type: "kind", kind: "main" },
        savePath: appPng,
      })) as Record<string, any>;
      const layout = await driver.getLayoutInfo().catch(() => ({ error: "layout-failed" }));
      const semantics = await appSemantics(driver);

      const mockReceipt = await captureMockup({
        port,
        storyId: story.id,
        surface: cp.surface,
        t: cp.t,
        outPng: mockPng,
      });

      const appDims = pngDims(appPng);
      const mockDims = pngDims(mockPng);
      const dimsMatch = appDims.w === mockDims.w && appDims.h === mockDims.h;

      const diff = await imageDiff({
        app: appPng,
        mock: mockPng,
        out: join(dir, "diff.png"),
        receiptOut: join(dir, "diff-receipt.json"),
        label: `${story.id}/${cp.chapter}`,
        cropBottomPt: cp.footerBandPt,
      });

      const surf = mockReceipt.surfaces?.[cp.surface] ?? {};
      const gates: Record<string, any> = {
        dims: {
          pass: dimsMatch || !cp.gateDims,
          app: appDims,
          mock: mockDims,
        },
      };
      if (cp.expect?.search != null) {
        const appFilter = String(semantics.filter ?? "");
        gates.search = {
          pass: appFilter === cp.expect.search && (surf.searchText ?? "") === cp.expect.search,
          app: appFilter,
          mock: surf.searchText ?? "",
          expected: cp.expect.search,
        };
      }
      if (cp.expect?.selectedName != null) {
        const appSel = String(semantics.selectedRowText ?? "");
        const mockSel = String(surf.selectedName ?? "");
        gates.selection = {
          pass:
            appSel.includes(cp.expect.selectedName) &&
            mockSel.startsWith(cp.expect.selectedName),
          app: appSel,
          mock: mockSel,
          expected: cp.expect.selectedName,
        };
      }
      const appFooter = (semantics.footerButtons ?? []) as Array<{ label: string; key: string }>;
      const mockFooter = (surf.footerButtons ?? []) as Array<{ label: string; key: string }>;
      if (appFooter.length && mockFooter.length) {
        const norm = (list: Array<{ label: string; key: string }>) =>
          list.map((b) => `${b.label}[${(b.key ?? "").replace(/\s+/g, "")}]`).join(" ");
        gates.footer = {
          pass: norm(appFooter) === norm(mockFooter),
          app: norm(appFooter),
          mock: norm(mockFooter),
        };
      }
      const pass = Object.values(gates).every((g: any) => g.pass !== false);

      const receipt = {
        receipt: "story-fidelity-proof",
        story: story.id,
        chapter: cp.chapter,
        t: cp.t,
        surface: cp.surface,
        pass,
        gates,
        pixel: {
          changedPixelRatio: diff?.changedPixelRatio ?? diff?.metrics?.changedPixelRatio ?? null,
          footerBandCroppedPt: cp.footerBandPt ?? 0,
          note: "vibrancy backdrop + glyph raster drift are receipted divergences; ratio is a ratchet, not a hard gate",
        },
        app: { shot: { w: shot.width, h: shot.height, error: shot.error ?? null }, semantics, drive: driveReceipt },
        mock: { digest: mockReceipt.digest, surface: surf },
        notes: cp.notes ?? null,
        capturedAt: new Date().toISOString(),
      };
      writeFileSync(join(dir, "receipt.json"), JSON.stringify(receipt, null, 2));
      writeFileSync(join(dir, "layout.json"), JSON.stringify(layout, null, 2));
      summary.push({
        story: story.id,
        chapter: cp.chapter,
        pass,
        dims: `${appDims.w}x${appDims.h} vs ${mockDims.w}x${mockDims.h}`,
        changedPixelRatio: receipt.pixel.changedPixelRatio,
        gates: Object.fromEntries(Object.entries(gates).map(([k, v]: [string, any]) => [k, v.pass])),
      });
      console.error(
        `  ${pass ? "✓" : "✗"} ${cp.chapter}  dims ${appDims.w}x${appDims.h} vs ${mockDims.w}x${mockDims.h}  ratio ${receipt.pixel.changedPixelRatio}`,
      );
    }
  } finally {
    await driver.close();
  }
}

async function main() {
  mkdirSync(OUT_ROOT, { recursive: true });
  const { port, stop } = serveMockups();
  await browser(["set", "viewport", "1600", "1000", "2"]);
  const summary: any[] = [];
  try {
    const selected = STORIES.filter(
      (s) => storyFilters.length === 0 || storyFilters.some((f) => s.id.startsWith(f)),
    );
    if (selected.length === 0) {
      console.error(`no stories match ${storyFilters.join(", ")}; known: ${STORIES.map((s) => s.id).join(", ")}`);
      process.exit(2);
    }
    for (const story of selected) {
      await runStory(story, port, summary).catch((err) => {
        console.error(`  story ${story.id} failed: ${err}`);
        summary.push({ story: story.id, chapter: "*", pass: false, error: String(err) });
      });
    }
  } finally {
    stop();
    await browser(["close"]).catch(() => {});
  }
  writeFileSync(join(OUT_ROOT, "summary.json"), JSON.stringify(summary, null, 2));
  console.log(JSON.stringify(summary, null, 2));
  const failed = summary.filter((s) => !s.pass);
  process.exitCode = failed.length ? 1 : 0;
}

await main();
