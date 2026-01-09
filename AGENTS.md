# Script Kit GPUI

Script Kit GPUI is a rewrite of Script Kit using **GPUI** (app shell) + **bun** (script runner) + a new SDK. Goal: **backwards compatibility** for Script Kit scripts with a new architecture.

> Keep this file <40k chars (it’s read often).

---

## 0. Agent quick-start checklist (MANDATORY)

Do these, in order:

1) Read this file end-to-end before changing code  
2) Check `.hive/issues.jsonl` for tasks/context  
3) **TDD**: write failing test → implement → refactor (see §23)  
4) Update bead status when starting/completing work  
5) Include `correlation_id` in **all** log entries/spans  
6) **UI changes**: test via **stdin JSON protocol** (never CLI args)  
7) Before every commit: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`  

---

## 1. UI testing (CRITICAL)

### 1.1 Stdin JSON protocol (ONLY supported way)
**Never pass scripts as command line arguments.** The app accepts JSONL commands via stdin.

✅ Correct:
```bash
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-editor-height.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

❌ Wrong (does nothing):
```bash
./target/debug/script-kit-gpui tests/smoke/hello-world.ts
```

Always set `SCRIPT_KIT_AI_LOG=1` when testing (compact logs save ~70% tokens).

### 1.2 Verification gate (run before every commit)
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

### 1.3 Stdin commands
```json
{"type":"run","path":"/abs/path/to/script.ts"}
{"type":"show"}
{"type":"hide"}
{"type":"setFilter","text":"search term"}
{"type":"showGrid","showBounds":true}
{"type":"hideGrid"}
{"type":"openNotes"}
{"type":"openAi"}
```

---

## 2. Quick reference (things that break most often)

- **Layout chain order:** Layout (`flex*`) → Sizing (`w/h`) → Spacing (`px/gap`) → Visual (`bg/border`)  
- **Lists:** `uniform_list` (fixed height **52px**) + `UniformListScrollHandle`  
- **Theme colors:** use `theme.colors.*` (**never** `rgb(0x...)`)  
- **Focus colors:** use `theme.get_colors(is_focused)`; re-render on focus change  
- **State updates:** after render-affecting changes, **must** `cx.notify()`  
- **Keyboard:** use `cx.listener()`; coalesce rapid events (20ms)  
- **Window positioning:** `Bounds::centered(Some(display_id), size, cx)` for multi-monitor  
- **Errors:** `anyhow::Result` + `.context()`; `NotifyResultExt` for user toasts  
- **Logging:** `tracing` JSONL, typed fields, include `correlation_id`, `duration_ms`  
- **Beads:** `hive_start` → progress → `swarm_complete` (**not** `hive_close`)  
- **Tests:** `tests/smoke/`=E2E; `tests/sdk/`=SDK; `--features system-tests` for clipboard/accessibility  
- **SDK preload:** tests import `../../scripts/kit-sdk`; runtime uses embedded SDK extracted to `~/.scriptkit/sdk/`  
- **Arrow keys:** match both `"up"|"arrowup"`, `"down"|"arrowdown"`, `"left"|"arrowleft"`, `"right"|"arrowright"`  
- **Config-driven UI:** font sizes/padding from `~/.scriptkit/config.ts`; use `config.get_*()` helpers  
- **Secondary windows:** Notes in `src/notes/` (`openNotes`), AI in `src/ai/` (`openAi`)  

---

## 3. AI compact log mode (SCRIPT_KIT_AI_LOG=1)

stderr format: `SS.mmm|L|C|message`

- `L`: `i` INFO, `w` WARN, `e` ERROR, `d` DEBUG, `t` TRACE  
- `C` categories: `P` position, `A` app, `U` UI, `S` stdin, `H` hotkey, `V` visibility, `E` exec,
  `K` key, `F` focus, `T` theme, `C` cache, `R` perf, `W` window_mgr, `X` error, `M` mouse_hover,
  `L` scroll_state, `Q` scroll_perf, `D` design, `B` script, `N` config, `Z` resize.

Example:
- Standard: `... INFO ... Selected display origin=(0,0)`
- Compact: `13.150|i|P|Selected display origin=(0,0)`

Enable:
```bash
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

---

## 4. Script metadata (preferred) + legacy

### Preferred: global `metadata` (use for all new scripts/tests)
```ts
import '../../scripts/kit-sdk';

export const metadata = {
  name: "My Script",
  description: "Does something useful",
  shortcut: "cmd+shift+m",
  author: "Your Name",
  // more typed fields available
};
```

Why prefer global metadata (vs `// Name:` comments):
- Type safety (TS types)
- IDE autocomplete + validation
- More fields + easier extensibility
- Compile-time feedback (vs runtime parsing)

### Legacy: comment-based (supported for backwards compatibility)
```ts
// Name: My Script
// Description: Does something useful
// Shortcut: cmd+shift+m
import '../../scripts/kit-sdk';
```

---

## 5. Scriptlet bundle frontmatter (`~/.scriptkit/snippets/*.md`)

Optional YAML frontmatter (line 1 must be `---`, closed by `---` on its own line):
```md
---
name: My API Tools
description: Collection of useful API utilities
author: John Doe
icon: api
---
```

Fields: `name`, `description`, `author`, `icon` (all optional). Unknown fields ignored.

Backwards compatibility:
- Frontmatter optional; bundles without it still work
- Filename fallback: missing `name` uses filename (sans `.md`)
- Invalid frontmatter logs warnings but does not break parsing

Validation behavior:
- Missing closing `---` → warn (line #), skip frontmatter
- Invalid YAML → warn (error + line #), skip frontmatter
- Errors also surface to users via HUD notifications with line #

Default icon mapping (when no explicit `icon:`):
- `template` → `text-cursor-input`
- `tool` → `wrench`
- `snippet` → `code`
- `script` → `terminal`
- `prompt` → `message-circle`
- `action` → `zap`
- fallback → `file-text`

Icon resolution priority:
1) explicit frontmatter icon  
2) tool-type default  
3) bundle fallback `file-text`

```rust
pub fn resolve_scriptlet_icon(frontmatter_icon: Option<&str>, tool_type: Option<&str>) -> &'static str {
  frontmatter_icon.unwrap_or_else(|| tool_type_to_icon(tool_type))
}
```

Troubleshooting:
- Frontmatter not parsed → must start on line 1 with `---` and have closing `---` on its own line; fix YAML indentation/quotes
- Icon not showing → verify icon name exists; check `[CACHE]` logs for resolution; use default mapping names
- HUD validation error → use the reported line number; common fix: quote special chars
- Metadata not updating → refresh cache by touching file; check `~/.scriptkit/logs/script-kit-gpui.jsonl` for cache logs

---

## 6. Autonomous UI testing protocol (MANDATORY)

### 6.1 Build-test-iterate loop
```bash
cargo build
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-editor-height.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```
Non‑negotiable: do not ask the user to test; don’t skip.

Log filtering helpers:
```bash
echo '{"type":"run","path":".../test-editor-height.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'RESIZE|editor|height_for_view|700'
tail -50 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep -i resize
```
Editor height test “good signs” in logs often include:
- `height_for_view(EditorPrompt) = 700`
- `Resize: 501 -> 700`

### 6.2 Visual testing (screenshots)
Use SDK `captureScreenshot()` (captures **only the app window**). Save PNG to `.test-screenshots/`. **Read the PNG** to verify.

Blocked system screenshot tools (do not use): `screencapture`, `scrot`, `gnome-screenshot`, `flameshot`, `maim`, ImageMagick `import`.

Minimal pattern:
```ts
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

await div(`<div class="p-4 bg-blue-500 text-white rounded-lg">Test</div>`);
await new Promise(r => setTimeout(r, 500));

const shot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `shot-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);
process.exit(0);
```

Tailwind sanity test idea (one script):
- render 3 boxes: `bg-blue-500`, `bg-green-500`, `bg-red-500` (white text, rounded, flex column gap)
- screenshot + verify visually

### 6.3 Grid overlay + layout inspection
Grid overlay (stdin):
```bash
echo '{"type":"showGrid","showBounds":true}' | ./target/debug/script-kit-gpui
echo '{"type":"showGrid","showBounds":true,"showBoxModel":true,"showAlignmentGuides":true,"showDimensions":true}' | ./target/debug/script-kit-gpui
echo '{"type":"hideGrid"}' | ./target/debug/script-kit-gpui
```

Example workflow: show overlay then run a test:
```bash
(echo '{"type":"showGrid","showBounds":true,"showDimensions":true}'; \
 echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-layout.ts"}') | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

Options: `gridSize` (default 8), `showBounds`, `showBoxModel`, `showAlignmentGuides`, `showDimensions`,
`depth` ("prompts" | "all" | specific component names). Env alternative: `SCRIPT_KIT_DEBUG_GRID=1`.

Color coding: prompts red, inputs teal, buttons yellow, lists mint, headers plum, containers sky.

Programmatic layout: SDK `getLayoutInfo()`:
```ts
interface LayoutInfo {
  windowWidth: number; windowHeight: number;
  promptType: string; // "arg"|"div"|"editor"|"mainMenu"|...
  components: LayoutComponentInfo[];
  timestamp: string; // ISO
}
interface LayoutComponentInfo {
  name: string;
  type: "prompt"|"input"|"button"|"list"|"listItem"|"header"|"container"|"panel"|"other";
  bounds: { x:number; y:number; width:number; height:number };
  boxModel?: { padding?:{top:number;right:number;bottom:number;left:number}; margin?:{top:number;right:number;bottom:number;left:number}; gap?:number };
  flex?: { direction?:"row"|"column"; grow?:number; shrink?:number; basis?:string; alignItems?:string; justifyContent?:string };
  depth: number; parent?: string; children: string[];
  explanation?: string;
}
```

Combine layout + screenshot (typical debug):
- `await Promise.all([getLayoutInfo(), captureScreenshot()])`
- log key bounds
- save screenshot and verify

### 6.4 Anti-patterns (don’t do these)
- Running scripts via CLI args (must use stdin JSON)
- “I can’t test without manual interaction” (use stdin + logs + screenshots)
- Not using `SCRIPT_KIT_AI_LOG=1`
- Capturing screenshot but not reading/verifying PNG
- Guessing at layout (use `captureScreenshot()` / `getLayoutInfo()` / grid overlay)

Why this matters:
- Key events require a visible/active window; check `focus_handle.is_focused=true`
- Key names vary by platform; match both arrow-key variants
- Layout issues only appear at runtime; test the actual app window

---

## 7. GPUI keyboard key names (CRITICAL)

GPUI often sends **short** arrow keys on macOS: `"up"|"down"|"left"|"right"`. Always match both:
```rust
match key.as_str() {
  "up" | "arrowup" => self.move_up(),
  "down" | "arrowdown" => self.move_down(),
  "left" | "arrowleft" => self.move_left(),
  "right" | "arrowright" => self.move_right(),
  _ => {}
}
```

---

## 8. Layout system

Rule: chain in order → layout → sizing → spacing → visual → children.

Flex patterns:
```rust
div().flex().flex_row().items_center().gap_2();
div().flex().flex_col().w_full();
div().flex().items_center().justify_center();
div().flex_1(); // fill remaining space
```

Conditional rendering:
```rust
div().when(is_selected, |d| d.bg(selected)).when_some(desc, |d, s| d.child(s));
```

Transforms:
```rust
div().map(|d| if loading { d.opacity(0.5) } else { d })
```

---

## 9. List virtualization + scroll performance

Use `uniform_list` with fixed-height rows (~52px) and `UniformListScrollHandle`:

```rust
uniform_list("script-list", filtered.len(), cx.processor(|this, range, _w, _cx| {
  this.render_list_items(range)
}))
.h_full()
.track_scroll(&self.list_scroll_handle);
```

Scroll to item:
```rust
self.list_scroll_handle.scroll_to_item(selected_index, ScrollStrategy::Nearest);
```

Rapid-key coalescing (20ms window) to avoid freezes (shape, not exact impl):
```rust
enum ScrollDirection { Up, Down }

fn process_arrow(&mut self, dir: ScrollDirection, cx: &mut Context<Self>) {
  let now = Instant::now();
  if now.duration_since(self.last_scroll_time) < Duration::from_millis(20)
     && self.pending_dir == Some(dir) {
    self.pending_delta += 1;
    return;
  }
  self.flush_pending(cx);
  self.pending_dir = Some(dir);
  self.pending_delta = 1;
  self.last_scroll_time = now;
}
```

---

## 10. Theme system

Theme types (see `src/theme.rs`):
- `Theme { colors: ColorScheme, focus_aware: Option<...>, opacity, drop_shadow, vibrancy }`
- `ColorScheme { background, text, accent, ui }`

Correct usage (no hardcoded rgb):
```rust
let colors = &self.theme.colors;
div().bg(rgb(colors.background.main)).border_color(rgb(colors.ui.border));
```

Focus-aware:
- compute `is_focused = self.focus_handle.is_focused(window)`
- if changed: update state + `cx.notify()`
- use `let colors = self.theme.get_colors(is_focused);`

For closures: prefer copyable extracted structs like `colors.list_item_colors()` (returns `Copy`).

---

## 11. Events + focus

Use `cx.listener()` for handlers; implement `Focusable` for keyboard focus.

Focus basics:
```rust
let focus_handle = cx.focus_handle();
focus_handle.focus(window);
let is_focused = focus_handle.is_focused(window);
```

Keyboard events (keys vary; handle both arrow variants):
```rust
window.on_key_down(cx.listener(|this, e: &KeyDownEvent, window, cx| {
  let key = e.key.as_ref().map(|k| k.as_str()).unwrap_or("");
  match key {
    "up"|"arrowup" => this.move_up(cx),
    "down"|"arrowdown" => this.move_down(cx),
    "enter"|"Enter" => this.submit(cx),
    "escape"|"Escape" => this.cancel(cx),
    _ => {}
  }
}));
```

Mouse:
```rust
.on_click(cx.listener(|this, _e, _w, cx| this.handle_click(cx)))
.on_mouse_down(MouseButton::Right, cx.listener(|this, e, _w, cx| this.show_context_menu(e.position, cx)))
```

---

## 12. Window management

Multi-monitor positioning: pick display containing the mouse; use visible bounds for usable area; often center at an “eye-line” (upper third).

Useful display APIs:
- `cx.displays()`, `cx.primary_display()`, `cx.find_display(id)`
- `display.bounds()`, `display.visible_bounds()`, `bounds.contains(&point)`

macOS floating panel (call after `cx.activate(true)`):
```rust
#[cfg(target_os="macos")]
unsafe {
  let app: id = NSApp();
  let window: id = msg_send![app, keyWindow];
  if window != nil {
    let _: () = msg_send![window, setLevel:3i32]; // NSFloatingWindowLevel
    let _: () = msg_send![window, setCollectionBehavior:1u64]; // join all spaces
  }
}
```

---

## 13. State management

- After any state mutation that affects rendering: `cx.notify()`
- Shared state: `Arc<Mutex<T>>` or channels; for async work, use `mpsc` sender → UI receiver.

---

## 14. Error handling

- Application errors: `anyhow::Result`; add `.context()` at boundaries
- Domain/library errors: `thiserror` when callers match variants
- User-facing errors: `NotifyResultExt` → log first (`tracing::error!`) then toast (auto-dismiss ~5s; 10s for critical)

Best practices:
- Don’t `unwrap()`/`expect()`
- Add context at each level (“which file?”, “what operation?”)
- Use typed fields in logs (avoid interpolated strings as primary data)

---

## 15. Logging + observability

Use `tracing` + `tracing-subscriber`:
- JSONL to `~/.scriptkit/logs/script-kit-gpui.jsonl`
- optional pretty output for humans

JSONL line example:
```json
{"timestamp":"...","level":"INFO","target":"script_kit::executor","message":"Script executed","fields":{"script_name":"hello.ts","duration_ms":142,"exit_code":0}}
```

Spans + timing:
- `#[instrument]` where useful
- record `duration_ms`; warn if slow (e.g. >100ms)

Correlation IDs:
- generate UUID per user action/run
- attach to spans so nested logs inherit it

Log level guide:
- `error`: failure; `warn`: unexpected but handled; `info`: key events; `debug`: dev; `trace`: very verbose.

Filter by targets (module paths), e.g. `script_kit::ui`, `script_kit::executor`, `script_kit::theme`.

---

## 16. Dev workflow + debugging

Hot reload: `./dev.sh` (cargo-watch).  
Triggers:
- `.rs` → rebuild
- `~/.scriptkit/theme.json` → live reload
- `~/.scriptkit/scripts/` → live reload
- `~/.scriptkit/config.ts` → restart

Debug:
- Logs panel: `Cmd+L`
- Tags: `[UI] [EXEC] [KEY] [THEME] [FOCUS] [HOTKEY] [PANEL]`
- Perf tags: `[KEY_PERF] [SCROLL_TIMING] [PERF_SLOW]`

Perf tests:
- `bun run tests/sdk/test-scroll-perf.ts`
- `npx tsx scripts/scroll-bench.ts`
Thresholds: P95 key latency <50ms; single key <16.67ms; scroll op <8ms.

---

## 17. Common gotchas (real failures)

- UI not updating → forgot `cx.notify()`
- Theme not applying → hardcoded `rgb(0x...)`
- Laggy list scrolling → no coalescing
- Window on wrong monitor → use mouse display + `Bounds::centered(...)`
- Focus styling wrong → missing `Focusable` or focus-change re-render
- Spawn failures silent → match `Command::spawn()` and log errors
- Script doesn’t exit after finishing → SDK calls `process.stdin.resume()`; add `(process.stdin as any).unref?.()` after resume
- Script `console.error()` not visible live → GPUI may read stderr only on exit; add stderr reader thread forwarding lines to `logging::log("SCRIPT", ...)`

---

## 17b. GPUI Vibrancy Gotcha (CRITICAL - hard to debug)

**Problem:** App looks washed out / too transparent over white backgrounds, even after configuring `NSVisualEffectView` materials correctly.

**Root Cause:** GPUI intentionally hides the macOS `CAChameleonLayer` (the native tint layer that provides dark tinting on top of blur). In the GPUI source code (`~/.cargo/git/checkouts/zed-*/`):

```rust
// blurred_view_update_layer in GPUI hides the tint:
if class_name.isEqualToString("CAChameleonLayer") {
    // Remove the desktop tinting effect.
    let _: () = msg_send![layer, setHidden: YES];
}
```

This means:
- Changing `NSVisualEffectMaterial` (SIDEBAR, HUD_WINDOW, POPOVER, etc.) has **no visible effect**
- The material's tint color is never applied because the CAChameleonLayer is hidden
- Native macOS apps like Raycast/Spotlight get automatic dark tinting; we do not

**The Fix:** Provide our own dark tint via GPUI theme colors at **70-85% opacity**:

```rust
// src/theme/gpui_integration.rs
let main_bg = if vibrancy_enabled {
    // GPUI hides CAChameleonLayer, so we must add our own dark tint
    // Need 70-85% opacity to stay dark over light backgrounds
    let tint_alpha = opacity.main.clamp(0.70, 0.85);
    with_vibrancy(colors.background.main, tint_alpha)
} else {
    hex_to_hsla(colors.background.main) // Fully opaque when vibrancy disabled
};
```

**What doesn't work (we tried these):**
- Changing material from SIDEBAR → HUD_WINDOW → ULTRA_DARK → POPOVER
- Changing appearance from DarkAqua → VibrantDark
- Setting `window.backgroundColor = clearColor`
- Setting `window.isOpaque = false`
- Recursively configuring all NSVisualEffectViews in the view hierarchy

All of these are correct and should be applied, but they **won't produce visible tinting** because GPUI hides the tint layer.

**Key files:**
- `src/theme/gpui_integration.rs:49-54` - The fix (background alpha 70-85%)
- `src/platform.rs` - NSVisualEffectView configuration (still useful for blur)
- `expert-bundles/vibrancy-*.md` - Full documentation of this investigation

**Debug with:** `Cmd+Shift+M` to cycle through material/appearance combinations (won't fix the tint issue, but useful for understanding).

---

## 18. Repository structure (key modules)

`src/`
- `main.rs` app entry + window setup + main render loop + ErrorNotification UI  
- `error.rs` `ScriptKitError`, `ErrorSeverity`, `NotifyResultExt`  
- `theme.rs` theme system  
- `prompts.rs` ArgPrompt, DivPrompt, EditorPrompt  
- `actions.rs` ActionsDialog  
- `protocol.rs` stdin JSON protocol + `ParseResult`  
- `scripts.rs` script loading + execution instrumentation  
- `config.rs` config loading + defaults  
- `executor.rs` bun execution + timing spans + structured logging  
- `watcher.rs` file watchers  
- `panel.rs` macOS panel configuration  
- `perf.rs` perf timing utilities  
- `logging.rs` dual-output logging (JSONL + pretty/compact)  
- `lib.rs` exports  
- `utils.rs` shared utilities  
- `notes/` Notes window module  
- `ai/` AI chat window module

Logs: `~/.scriptkit/logs/script-kit-gpui.jsonl`

---

## 19. SDK deployment architecture

SDK source: `scripts/kit-sdk.ts`

Two-tier deployment:
1) **Build time (dev):** `build.rs` copies `scripts/kit-sdk.ts` to `~/.scriptkit/sdk/`  
2) **Compile time:** `executor.rs` embeds via `include_str!("../scripts/kit-sdk.ts")`  
3) **Runtime:** `ensure_sdk_extracted()` writes embedded SDK to `~/.scriptkit/sdk/kit-sdk.ts`  
4) **Execution:** `bun run --preload ~/.scriptkit/sdk/kit-sdk.ts <script>`

Tests import `../../scripts/kit-sdk` (repo path). Production scripts use runtime-extracted SDK.

tsconfig mapping:
```json
{ "compilerOptions": { "paths": { "@scriptkit/sdk": ["./sdk/kit-sdk.ts"] } } }
```

---

## 20. User configuration (`~/.scriptkit/config.ts`)

Example:
```ts
import type { Config } from "@scriptkit/sdk";
export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  padding: { top: 8, left: 12, right: 12 },
  editorFontSize: 16,
  terminalFontSize: 14,
  uiScale: 1.0,
  builtIns: { clipboardHistory: true, appLauncher: true },
  bun_path: "/opt/homebrew/bin/bun",
  editor: "code"
} satisfies Config;
```

Rust helpers (use these; they handle defaults):
- `config.get_editor_font_size()` (default 14)
- `config.get_terminal_font_size()` (default 14)
- `config.get_padding()` (top 8, left/right 12)
- `config.get_ui_scale()` (default 1.0)
- `config.get_builtins()` (clipboardHistory/appLauncher default true)
- `config.get_editor()` (default `"code"`)

Font sizing patterns:
- Editor: `line_height = font_size * 1.43`
- Terminal: cell dims scale with font; `line_height = font_size * 1.3`

---

## 21. Notes window (`src/notes/`)

Separate floating window. gpui-component + SQLite `~/.scriptkit/db/notes.sqlite`. Theme synced from `~/.scriptkit/theme.json`.

Files:
- `window.rs` NotesApp view + open/close + quick_capture
- `storage.rs` SQLite persistence + FTS5 search + delete/restore
- `model.rs` `NoteId`, `Note`, `ExportFormat`

Features: markdown toolbar, sidebar list + search + trash, FTS5, soft delete/trash restore, export (copies to clipboard), character count footer, hover icons.

Root wrapper (required):
```rust
let handle = cx.open_window(opts, |w, cx| {
  let view = cx.new(|cx| NotesApp::new(w, cx));
  cx.new(|cx| Root::new(view, w, cx))
})?;
```

Theme mapping: Script Kit colors → gpui-component `ThemeColor` (e.g. via `hex_to_hsla(...)`).

Testing:
- stdin `{"type":"openNotes"}`
- captureScreenshot() captures the **main** window; Notes testing is mainly log-based for now
- log filter: `grep -i 'notes|PANEL'`

Open methods: hotkey `Cmd+Shift+N` (configurable `notesHotkey`), tray menu, stdin.  
Single-instance: global `OnceLock<Mutex<Option<WindowHandle<Root>>>>`.

---

## 22. AI window (`src/ai/`)

Separate floating BYOK chat window. SQLite `~/.scriptkit/db/ai-chats.sqlite`. Theme synced from Script Kit theme.

Files: `window.rs`, `storage.rs`, `model.rs` (`Chat`, `Message`, `ChatId`, roles), `providers.rs` (Anthropic/OpenAI), `config.rs` (env detection).

Features: streaming responses, markdown rendering, model picker, chat history sidebar, multi-provider, BYOK.

API keys via env:
- `SCRIPT_KIT_ANTHROPIC_API_KEY`
- `SCRIPT_KIT_OPENAI_API_KEY`
- `SCRIPT_KIT_VERCEL_API_KEY` (for Vercel AI Gateway)
(set in shell profile or system env)

Testing: stdin `{"type":"openAi"}`; log filter `grep -i 'ai|chat|PANEL'`.  
Open methods: hotkey `Cmd+Shift+Space` (configurable `aiHotkey`), tray menu, stdin.  
Single-instance: `OnceLock<Mutex<Option<WindowHandle<Root>>>>`.

---

## 23. Testing infrastructure

Dirs:
- `tests/smoke/` E2E (run via stdin JSON protocol)
- `tests/sdk/` SDK tests (often `bun run ...`)

SDK preload in tests:
```ts
import '../../scripts/kit-sdk'; // globals: arg(), div(), editor(), fields(), captureScreenshot(), getLayoutInfo(), ...
```

Test output JSONL example:
```json
{"test":"arg-string-choices","status":"running","timestamp":"2024-..."}
{"test":"arg-string-choices","status":"pass","result":"Apple","duration_ms":45,"timestamp":"2024-..."}
```
Status values: `running | pass | fail | skip` (with `error`/`reason`).

Minimal test skeleton:
```ts
import '../../scripts/kit-sdk';

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const name = "my-test";
log(name, "running");
const start = Date.now();
try {
  const result = await arg("Pick", ["A", "B"]);
  log(name, "pass", { result, duration_ms: Date.now() - start });
} catch (e) {
  log(name, "fail", { error: String(e), duration_ms: Date.now() - start });
}
```

Run:
```bash
bun run scripts/test-runner.ts
bun run scripts/test-runner.ts tests/sdk/test-arg.ts
cargo build && echo '{"type":"run","path":"'"$(pwd)"'/tests/sdk/test-arg.ts"}' | ./target/debug/script-kit-gpui
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/hello-world.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
cargo test
```

System tests (side effects; macOS APIs): `cargo test --features system-tests`  
Run ignored interactive: `cargo test --features system-tests -- --ignored`

---

## 24. Hive / beads task management

`.hive/`:
- `issues.jsonl` task tracking
- `memories.jsonl` semantic learnings

issues.jsonl record example:
```json
{"id":"cell--...","title":"...","status":"open","priority":1,"issue_type":"task","created_at":"...","updated_at":"...","parent_id":null,"dependencies":[],"labels":[],"comments":[]}
```

Enums:
- `issue_type`: `epic|task|bug|feature|chore`
- `status`: `open|in_progress|blocked|closed`
- `priority`: 0 critical, 1 high, 2 medium, 3 low

MCP commands (don’t use CLI):
- query/next: `hive_query(...)`, `hive_ready()`
- create: `hive_create(...)`, `hive_create_epic(...)`
- update: `hive_start({id})`, `hive_update({id,...})`
- finish (required): `swarm_complete(...)` (**not** `hive_close()`)

Epic/subtask example:
```ts
hive_create_epic({
  epic_title: "Add search functionality",
  epic_description: "Implement fuzzy search for script list",
  subtasks: [
    { title: "Add search input UI", files: ["src/main.rs"], priority: 0 },
    { title: "Implement fuzzy matching", files: ["src/scripts.rs"], priority: 1 },
    { title: "Add keyboard navigation", files: ["src/main.rs"], priority: 1 }
  ]
});
```

Mandatory lifecycle:
`swarmmail_init()` → `hive_start()` → progress at 25/50/75 (`swarm_progress()`) → `swarm_complete()`.

Progress example:
```ts
swarm_progress({
  project_key: "/path/to/project",
  agent_name: "your-agent-name",
  bead_id: "cell--xxxxx",
  status: "in_progress",
  progress_percent: 50,
  message: "Completed X, now working on Y",
  files_touched: ["src/main.rs"]
});
```

---

## 25. Agent observability + queries

Required fields when relevant: `correlation_id`, `duration_ms`, `bead_id`, `agent_name`, `files_touched`.

Log queries:
```bash
grep '"correlation_id":"abc-123"' ~/.scriptkit/logs/script-kit-gpui.jsonl
grep '"duration_ms":' ~/.scriptkit/logs/script-kit-gpui.jsonl | jq 'select(.fields.duration_ms > 100)'
grep '"level":"ERROR"' ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -50
```

---

## 26. Agent anti-patterns + protocols

Don’t:
- skip `swarmmail_init()` (work not tracked)
- use `hive_close()` (reservation release breaks) → use `swarm_complete()`
- edit unreserved files → reserve first
- commit without verification gate
- skip 25/50/75 progress updates

Reserve files:
```ts
swarmmail_reserve({
  paths: ["src/main.rs", "src/theme.rs"],
  reason: "cell--xxxxx: Implement feature X",
  exclusive: true
});
```

When blocked: notify coordinator + mark bead blocked (include concrete reason).

Scope change: request permission; don’t silently expand beyond reserved files.

Pre-commit checklist:
- check / clippy / test pass
- only reserved files modified
- bead status updated
- progress reported
- correlation IDs present (where applicable)

---

## 27. Lessons learned (hard-won)

If you hit 100+ failures after a refactor:
1) stop, analyze  
2) count error types to find the single root cause:
```bash
cargo test 2>&1 | grep "error\\[E" | sort | uniq -c
```
3) make systematic helpers + bulk transforms; verify incrementally (`cargo check` first)

Type migration checklist (`T` → `Arc<T>`):
1) structs → 2) signatures → 3) return types → 4) test helpers → 5) transform test data →
6) fix field assigns → 7) `cargo check` → 8) fix stragglers → 9) full `cargo test`.

Example helper approach:
```rust
fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
  scripts.into_iter().map(Arc::new).collect()
}
```
Then bulk-edit call sites (automation > manual).

---

## 28. Hard-won learnings (frustration-extracted)

These patterns emerged from painful debugging sessions. They're non-obvious and easy to forget.

### 28.1 GPUI/Rust technical

**RefCell borrow conflicts during resize:**
Window resize/move callbacks can trigger while a RefCell is already borrowed. Fix: use `Window::defer()` to schedule the operation at the end of the current effect cycle:
```rust
// Don't call platform APIs directly during render:
// platform::resize_first_window_to_height(height); // WRONG - can panic

// Use Window::defer to run after borrows are unwound:
window.defer(cx, |_window, _cx| {
    platform::resize_first_window_to_height(height);
});
```
See `src/window_ops.rs` for a coalescing queue pattern that also prevents jitter from rapid updates.

**async_channel vs std::sync::mpsc:**
Use `async_channel` for event-driven behavior. `mpsc::try_recv()` polling wastes CPU; `async_channel` integrates with async runtimes and wakes only when messages arrive.

**Multi-monitor bounds accuracy:**
`cx.displays()` can return stale or incorrect bounds. For accurate multi-monitor positioning, use native NSScreen APIs directly:
```rust
#[cfg(target_os = "macos")]
fn get_mouse_display_bounds() -> Option<Bounds<Pixels>> {
    // Use CGMainDisplayID, NSScreen.screens, mouseLocation
    // Don't rely solely on cx.displays()
}
```

**Key event handling completeness:**
Beyond arrow keys, also match both variants for Enter (`"enter"|"Enter"`), Escape (`"escape"|"Escape"`), Tab (`"tab"|"Tab"`). GPUI normalization varies by platform and version.

### 28.2 Theming + vibrancy

**Vibrancy requires two things:**
1. `WindowBackgroundAppearance::Blurred` in window options
2. Semi-transparent background colors (70-85% alpha)

Missing either = no vibrancy effect. The window options enable the blur; the alpha lets it show through.

**Consistent opacity levels:**
- Selection highlight: ~50% alpha (`0x80` suffix, e.g., `0x3B82F680`)
- Hover highlight: ~25% alpha (`0x40` suffix, e.g., `0x3B82F640`)
- Disabled state: ~30% opacity via `.opacity(0.3)`

Using consistent levels prevents jarring visual transitions.

**SVG icon theming:**
Always use `stroke='currentColor'` (not hardcoded colors) so icons inherit the theme's text color. For fill-based icons, use `fill='currentColor'`.

**Focus-aware color extraction for closures:**
When passing colors to closures (e.g., in `uniform_list` renderers), extract a `Copy` struct first:
```rust
let item_colors = theme.colors.list_item_colors(); // Returns Copy struct
uniform_list(..., move |...| {
    div().bg(item_colors.background) // Works in closure
})
```

### 28.3 Agent workflow

**Autonomous testing is mandatory:**
Never say "the user should test this" or "please verify manually." Agents must:
1. Write a test script
2. Run it via stdin JSON protocol
3. Analyze logs/screenshots
4. Iterate until correct

**Visual verification requires reading the file:**
Capturing a screenshot is not enough. You must:
```ts
const shot = await captureScreenshot();
writeFileSync(path, Buffer.from(shot.data, 'base64'));
// THEN: Use Read tool to actually examine the PNG
```
"I captured a screenshot" without reading it = incomplete verification.

**Bead completion protocol:**
Use `swarm_complete()`, never `hive_close()`. The swarm_complete function:
- Releases file reservations
- Records metrics for implicit feedback scoring
- Properly closes the bead

`hive_close()` skips reservation cleanup → causes conflicts in subsequent work.

**Mass test failure triage:**
When a refactor causes 100+ test failures:
1. **Stop** - don't fix tests one by one
2. **Count error types:**
   ```bash
   cargo test 2>&1 | grep "error\[E" | sort | uniq -c
   ```
3. Find the single root cause (usually one type dominates)
4. Create systematic helpers for bulk fixes
5. Verify incrementally: `cargo check` before `cargo test`

**Feature-gated system tests:**
Tests that use clipboard, accessibility APIs, or other system resources must be behind `--features system-tests`:
```rust
#[cfg(feature = "system-tests")]
#[test]
fn test_clipboard_integration() { ... }
```
Run with: `cargo test --features system-tests`

---

## 29. GPUI Anti-Patterns (preloaded skill)

Common mistakes that cause bugs. Check this list when debugging.

### Forgetting `cx.notify()`
```rust
// WRONG - UI won't update
fn increment(&mut self) {
    self.count += 1;
}

// CORRECT
fn increment(&mut self, cx: &mut Context<Self>) {
    self.count += 1;
    cx.notify();  // Required for re-render
}
```

### Skipping Root wrapper
```rust
// WRONG - theming breaks
cx.open_window(opts, |_window, cx| {
    cx.new(|cx| MyView::new())
})

// CORRECT
cx.open_window(opts, |window, cx| {
    let view = cx.new(|cx| MyView::new(window, cx));
    cx.new(|cx| Root::new(view, window, cx))  // Required
})
```

### Missing `gpui_component::init()`
```rust
Application::new().run(|cx: &mut App| {
    gpui_component::init(cx);  // MUST call before opening windows
    cx.open_window(...);
});
```

### Creating entities in render()
```rust
// WRONG - creates new entity every frame, loses state
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = cx.new(|cx| TextInput::new(window, cx));  // BAD
        div().child(input)
    }
}

// CORRECT - create once in new(), store in struct
struct MyView { input: Entity<TextInput> }

impl MyView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self { input: cx.new(|cx| TextInput::new(window, cx)) }
    }
}
```

### Blocking the main thread
```rust
// WRONG - freezes UI
let data = reqwest::blocking::get(url).unwrap();

// CORRECT - use async
cx.spawn(|this, mut cx| async move {
    let data = reqwest::get(url).await.unwrap();
    this.update(&mut cx, |this, cx| {
        this.data = data;
        cx.notify();
    }).ok();
}).detach();
```

### Not detaching spawned tasks
```rust
// WRONG - task may never run (dropped immediately)
cx.spawn(|this, mut cx| async move { ... });

// CORRECT
cx.spawn(|this, mut cx| async move { ... }).detach();
```

**Debugging checklist:**
1. UI not updating? → Missing `cx.notify()`
2. Theming broken? → Missing `init()` or `Root`
3. State resetting? → Creating entities in `render()`
4. App freezing? → Blocking calls on main thread
5. Async not completing? → Task not detached

---

## 30. Rust Smart Pointers (preloaded skill)

Essential patterns for GPUI apps. Most bugs come from misusing these.

### Quick Decision Tree
```
Need multiple owners?
  NO  → Use regular ownership/references
  YES → Will it cross threads?
          NO  → Rc<T>
          YES → Arc<T>

Need interior mutability?
  Single-threaded → RefCell<T> (panics if misused!)
  Multi-threaded  → Mutex<T> or RwLock<T>

Need lazy initialization?
  → OnceLock<T> or LazyLock<T>
```

### Arc<Mutex<T>> - Shared mutable state across threads
```rust
// The standard pattern for thread-safe shared state
let state = Arc::new(Mutex::new(AppState::new()));
let state_clone = Arc::clone(&state);
thread::spawn(move || {
    let mut guard = state_clone.lock().unwrap();
    guard.update();
});
```

### OnceLock<Mutex<T>> - Global singletons
```rust
// The canonical pattern for global mutable singletons
static WINDOW_MANAGER: OnceLock<Mutex<WindowManager>> = OnceLock::new();

fn get_manager() -> &'static Mutex<WindowManager> {
    WINDOW_MANAGER.get_or_init(|| Mutex::new(WindowManager::new()))
}
```

### Arc vs Rc
- `Rc<T>` - Single-threaded only (UI callbacks that never leave main thread)
- `Arc<T>` - Thread-safe (callbacks that might cross async/thread boundaries)

```rust
// UI callbacks (single-threaded) - Rc is fine
on_click: Option<Rc<OnClickCallback>>,

// Hotkey handlers (cross threads) - must use Arc
pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;
```

### Common Anti-Patterns

**Deadlock from nested locks:**
```rust
// DEADLOCK - locking same mutex twice
let guard1 = data.lock().unwrap();
let guard2 = data.lock().unwrap();  // Blocks forever!

// FIX - drop first guard
{ let guard = data.lock().unwrap(); /* use */ }
{ let guard = data.lock().unwrap(); /* OK */ }
```

**RefCell borrow panics:**
```rust
// PANIC - overlapping borrows
let r = data.borrow();
data.borrow_mut().push(4);  // PANIC!

// FIX - use try_borrow_mut or ensure no overlap
if let Ok(mut r) = data.try_borrow_mut() { r.push(4); }
```

**Holding locks across await:**
```rust
// BAD - lock held during await
let mut guard = data.lock().unwrap();
do_async_work().await;  // Lock held!

// GOOD - copy, unlock, await
let value = { data.lock().unwrap().clone() };
do_async_work().await;
```

### Quick Reference Table

| Type | Thread-safe | Multiple owners | Mutable | Use case |
|------|-------------|-----------------|---------|----------|
| `Box<T>` | Yes* | No | Yes | Heap allocation, trait objects |
| `Rc<T>` | No | Yes | No | Single-threaded sharing |
| `Arc<T>` | Yes | Yes | No | Multi-threaded sharing |
| `RefCell<T>` | No | N/A | Yes | Interior mut (runtime check) |
| `Mutex<T>` | Yes | N/A | Yes | Thread-safe interior mut |
| `OnceLock<T>` | Yes | N/A | No | One-time initialization |

---

## 31. GPUI Testing Patterns (preloaded skill)

### Test File Organization
```
src/
  theme/
    mod.rs           # Implementation
    theme_tests.rs   # Tests (separate file)
```

Import in mod.rs:
```rust
#[cfg(test)]
mod theme_tests;
```

### Test Helpers (reduce boilerplate)
```rust
fn test_scriptlet(name: &str, tool: &str, code: &str) -> Scriptlet {
    Scriptlet { name: name.to_string(), tool: tool.to_string(), code: code.to_string(), ..Default::default() }
}

fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
    scripts.into_iter().map(Arc::new).collect()
}
```

### System Tests (feature-gated)
Tests that need OS permissions (clipboard, accessibility, hotkeys):
```rust
#[cfg(feature = "system-tests")]
#[test]
fn test_clipboard_integration() { ... }
```
Run with: `cargo test --features system-tests`

### Platform-Specific Tests
```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_specific() { ... }

#[cfg(unix)]
#[test]
fn test_unix_signals() { ... }
```

### Environment Variable Tests
```rust
#[test]
fn test_env_var() {
    // Save original
    let original = std::env::var("MY_VAR").ok();
    
    // Test
    std::env::set_var("MY_VAR", "value");
    assert!(check_var());
    
    // Restore (always!)
    match original {
        Some(v) => std::env::set_var("MY_VAR", v),
        None => std::env::remove_var("MY_VAR"),
    }
}
```

### Code Audit Tests
Enforce invariants about the codebase:
```rust
#[test]
fn test_no_direct_cx_hide() {
    let content = fs::read_to_string("src/app_execute.rs").unwrap_or_default();
    assert!(!content.contains("cx.hide()"), "Use close_and_reset_window() instead");
}
```

### Anti-Patterns
- **DON'T** use `cx.run()` in unit tests (needs running app)
- **DON'T** rely on global state between tests
- **DON'T** hardcode paths (`/Users/john/...`) - use temp dirs
- **DON'T** forget platform guards for OS-specific tests
- **DON'T** skip cleanup (env vars, temp files)

### Running Tests
```bash
cargo test                           # All tests
cargo test theme_tests               # Specific module
cargo test --features system-tests   # System tests
cargo test -- --nocapture            # Show output
cargo test test_default_config       # Single test
```

---

## 32. References
- GPUI docs: https://docs.rs/gpui/latest/gpui/
- Zed source (GPUI): https://github.com/zed-industries/zed/tree/main/crates/gpui
- Project research: `GPUI_RESEARCH.md`, `GPUI_IMPROVEMENTS_REPORT.md`
- Protocol reference: `docs/PROTOCOL.md`
- Roadmap: `docs/ROADMAP.md`
- Archived docs: `docs/archive/`

---

## 33. Landing the plane (session completion)

Work is not done until `git push` succeeds.

1) File issues for remaining work  
2) If code changed: run quality gates (check/clippy/test)  
3) Update issue status  
4) Push to remote (required):
```bash
git pull --rebase
bd sync
git push
git status
```
`git status` must show “up to date with origin”.

Rules:
- never stop before pushing
- never say “ready to push when you are” (you push)
- if push fails: resolve and retry until success

5) Clean up (stashes/branches)  
6) Verify everything committed + pushed  
7) Hand off context for next session
