# Terminal Prompt — design-contract brief (draft)

Screen slice: `design/mockups/screens/terminal-prompt/`
Owner surface: SDK terminal prompt — protocol `term` message → `AppView::TermPrompt`
(`src/protocol/message/variants/prompts_media.rs:400-408`, `src/prompt_handler/mod.rs:2199-2262`).
NOT the QuickTerminalView (harness terminal) — that is a separate fixture
(edge-to-edge, native footer, `quick_terminal` surface) and should be its own
screen slice later.

Every claim below is tagged **MEASURED-from-source** (file:line) or **GUESS**.
Nothing here was runtime-verified — capturing the reference PNG is the next step.

---

## 1. Surface anatomy

### Window
- Width 750 — MEASURED-from-source: `MAIN_WINDOW_WIDTH` `src/window_resize/mod.rs:291`.
- Height 700 — MEASURED-from-source: `ViewType::TermPrompt → max_height`
  `src/window_resize/mod.rs:640`; `layout::MAX_HEIGHT = px(DEFAULT_LAYOUT_MAX_HEIGHT)`
  `src/window_resize/mod.rs:543`; `DEFAULT_LAYOUT_MAX_HEIGHT = 700.0`
  `src/config/defaults.rs:20`. Resize is deferred and id-guarded
  (`src/prompt_handler/mod.rs:2238-2262`). User `layout.max_height` config can
  override (`runtime_layout_config`, `src/window_resize/mod.rs:567-573`) — the
  checked-in contract uses the default 700.
- Window chrome (radius 22, vibrancy material/tint, borders) — shared with
  main menu; existing generated tokens (`--sk-window-*`).

### Context header (above the terminal)
- `AppView::TermPrompt` is NOT in `uses_shared_main_view_header`
  (`src/main_sections/app_view_state.rs:984-1016`), so the root wraps it:
  header + clipped flex-1 content box — MEASURED-from-source:
  `src/main_sections/render_impl.rs:896-919`.
- Header renderer: `render_clickable_main_view_context_header`
  (`src/app_impl/ui_window.rs:1776-1786`) →
  `render_main_view_context_header(ctx, HEADER_PADDING_X)`
  (`src/components/main_view_chrome.rs:310-322`): `px(16)` / `py(8)`
  (`HEADER_PADDING_X` `src/window_resize/mod.rs:100`, `HEADER_PADDING_Y` :102).
  Context row height 22 (existing `--sk-main-menu-context-height`) → header
  band 38. Same anatomy the confirm mockup already locked
  (`--sk-confirm-header-*`).

### Terminal wrapper (`render_prompts/term.rs::render_term_prompt`)
- Fixed-height flex column, `h(content_height)` where SDK terminals use
  `window_resize::layout::MAX_HEIGHT` (700) — MEASURED-from-source:
  `src/render_prompts/term.rs:238-242,392-399`.
- No background when vibrancy is on (`get_vibrancy_background` → None) —
  `src/render_prompts/term.rs:229-230,396`. No rounded corners, no shadow
  (comments at :391,397).
- Entity flags set every render: `escape_cancels = !is_quick_terminal`,
  `edge_to_edge = is_quick_terminal` (false here) —
  `src/render_prompts/term.rs:218-227`.
- Children: terminal content child (`flex_1`, `min_h(0)`, `overflow_hidden`,
  :408-429) then the footer hint strip via `main_window_footer_slot`
  (:434-447).

### TermPrompt entity (`src/term_prompt/mod.rs`)
- Explicit height 670 = `MAX_HEIGHT − FOOTER_HEIGHT(30)` — MEASURED-from-source:
  `src/prompt_handler/mod.rs:2219-2231`; `FOOTER_HEIGHT = 30.0`
  `src/window_resize/mod.rs:527`. Entities don't inherit flex sizing, so the
  container applies `.h(670)` (`src/term_prompt/mod.rs:1695-1700`).
- Padding: `effective_padding()` = config `ContentPadding` (top 8, left 12,
  right 12), bottom = top (8) for SDK terminals — MEASURED-from-source:
  `src/term_prompt/mod.rs:274-295`; defaults `src/config/defaults.rs:6-8`;
  getters `src/config/types.rs:2143-2145`.
- Font: `FONT_MONO` = "JetBrains Mono" — MEASURED-from-source:
  `src/term_prompt/mod.rs:608-611`, `src/list_item/mod.rs:612`.
- Font size: 14 — `get_terminal_font_size()` `src/config/types.rs:2153-2157`,
  `DEFAULT_TERMINAL_FONT_SIZE = 14.0` `src/config/defaults.rs:12`.
- **Cell height 18.2 = 14 × 1.3** — the terminal OVERRIDES GPUI's φ default
  line height (14pt φ would be 23px): `LINE_HEIGHT_MULTIPLIER = 1.3`
  `src/term_prompt/mod.rs:27-29`, `cell_height()` :649-651, applied as
  `.line_height(px(cell_height))` :1193 and per-row `h(px(cell_height))` :1199.
- Cell width 8.43: measured `em_advance` of FONT_MONO at 14pt (documented
  8.4287) rounded UP to the next hundredth — `conservative_cell_width`
  `src/term_prompt/mod.rs:61-68`, `ensure_measured_cell_width` :634-646, test
  receipt :2131-2143. First-frame fallback before measurement: 8.5 scaled
  (`BASE_CELL_WIDTH` :34, `cell_width()` :625-628).
- Grid size at the fixture geometry — MEASURED-from-source arithmetic
  (`calculate_terminal_size_with_cells` :789-815, floor + MIN_COLS 20 /
  MIN_ROWS 5 :51-52):
  cols = floor((750−12−12)/8.43) = **86**; rows = floor((670−8−8)/18.2) = **35**.
- Colors (`render_content` :1161-1324 + container :1572-1583):
  - default fg = `theme.colors.text.primary` (:1167, container text color :1581).
  - default cell bg = **transparent** (vibrancy shows through; :1160,1285-1294).
  - cursor cell: bg `accent.selected`, fg inverts to the cell bg (:1164,1262-1264).
  - selection: `(accent.selected_subtle << 8) | 0x0f` ≈ 6% alpha (:1166,1265-1274).
  - explicit ANSI cells: raw cell RGB from the emulator palette (below).
  - bold → `FontWeight::BOLD` (:1306-1308); underline → `text_decoration_1` (:1309-1311).
- Bell flash: `border_2` in `accent.selected` for 150ms (:53-54,1686-1692) — not
  in the fixture.
- Scrollback indicator (only when scrolled): `bottom_2 right_2 px_2 py_1`,
  bg `background.title_bar`, text `text.secondary`, `text_xs`, `rounded_sm`
  (:1703-1728) — not in the fixture (offset 0).

### ANSI palette mapping (terminal theme)
- Adapter: `ThemeAdapter::from_theme` — MEASURED-from-source:
  `src/terminal/theme_adapter/impls.rs:15-59`:
  fg = `terminal.foreground.unwrap_or(text.primary)`,
  bg = `terminal.background.unwrap_or(background.main)`,
  cursor = `accent.selected`, selection bg = `accent.selected_subtle`,
  selection fg = `text.secondary`, all 16 ANSI from `theme.colors.terminal`.
- Stock dark palette values (`TerminalColors::dark_default`,
  `src/theme/types.rs:722-745`):
  black `#000000`, red `#cd3131`, green `#50fa7b`, yellow `#e5e510`,
  blue `#5c9ceb`, magenta `#bc3fbc`, cyan `#56d4e2`, white `#e5e5e5`,
  brightBlack `#666666`, brightRed `#f14c4c`, brightGreen `#69ff94`,
  brightYellow `#f5f543`, brightBlue `#6eb4ff`, brightMagenta `#d670d6`,
  brightCyan `#8be9fd`, brightWhite `#ffffff`.
  (Light palette :748-768; unfocused dims everything ×0.7 —
  `theme_adapter/impls.rs:166-190` — capture MUST be focused.)
- Note: `src/terminal/theme_adapter.rs:37-53` has a second, DIFFERENT set of
  dark ANSI defaults (`ANSI_GREEN 0x0dbc79` etc.) used only by
  `AnsiColors::default()` / `dark_default()` fallback when no theme is passed.
  Themed terminals (the app path) always take `theme.colors.terminal` — record
  as a conflict, do not blend the two.

### Footer ("command bar" reality check)
- `AppView::TermPrompt` has NO native footer surface
  (`native_footer_surface → None`, `src/main_sections/app_view_state.rs:1081-1088`),
  so `main_window_footer_slot` returns the GPUI footer, not the AppKit spacer
  (`src/app_impl/ui_window.rs:1421-1450`). This surface is the exception to the
  "empty footer band in GPUI captures" rule — its footer is GPUI-painted
  (chrome audit declares `custom_hint_strip` /
  `terminal_owns_contextual_footer`, `src/render_prompts/term.rs:193-206`).
- Footer content: `render_terminal_prompt_hint_strip(None, None, true)` →
  items `["⌘K Actions", "Esc Cancel", "⌘W Close"]` joined into ONE string with
  `" · "` (`src/render_prompts/term.rs:92-146`, join :145).
- HintStrip metrics (`src/components/hint_strip.rs`): height 36
  (`HINT_STRIP_HEIGHT` ← `MAIN_WINDOW_HINT_STRIP_HEIGHT = 36.0`
  `src/window_resize/mod.rs:37,108`), padding 14/8 (:104,:106), content gap 8
  (hint_strip.rs:20), text 12.5 (:51), text color = `text.primary` @ 0.80
  (`HINT_TEXT_OPACITY` `src/window_resize/mod.rs:114` ←
  `OPACITY_TEXT_MUTED` `src/theme/opacity.rs:57` — matches existing
  `--sk-footer-text`), keycap 6/1 pad, radius 5, bg `text.primary` @ 0.12
  (:47-50), ⌘ SVG icon 14 (:31), icon/label gap 3 (:36), right-aligned via
  flex-1 spacer (:735).
- **Paint-order quirk** — MEASURED-from-source: `parse_hint`
  (hint_strip.rs:526-604) consumes only the LEADING `⌘` + `K` of the joined
  string; everything after (including "Esc Cancel · ⌘W Close") becomes the
  label, and `render_hint_element_hsla` paints label FIRST, keys LAST
  (:624-663). Painted truth: `Actions · Esc Cancel · ⌘W Close [⌘][K]`.
- **Clipping conflict** — MEASURED-from-source arithmetic (needs runtime
  proof): window content 700 = header 38 + clipped flex box ≈ 662, but the
  terminal shell is a fixed 700 with the strip at shell-y 664-700 → the strip
  starts ~2pt past the clip line and never paints. Root cause: the shell
  budgets no header, and the entity uses `FOOTER_HEIGHT` (30) while the strip
  is `HINT_STRIP_HEIGHT` (36). The terminal GRID is unaffected (last row ends
  at shell-y 645 < 662). The mockup keeps the strip in the DOM, clipped, like
  the confirm screen's off-window footer spacer.
- The bespoke `TerminalCommandBar` component
  (`src/terminal/command_bar_ui.rs`) is re-exported but not mounted by any
  current render path — ⌘K on this surface opens the shared ActionsDialog in a
  separate vibrancy window with `terminal_actions_for_dialog()`
  (`src/app_impl/actions_toggle.rs:1626-1660`, backdrop wiring
  `src/render_prompts/term.rs:449-464`). GUESS (by absence of references):
  the command-bar popup is legacy; do not mock it.

---

## 2. Fixture strategy + proposed capture block

### Why this is the reproducible path
- `term` is an SDK/script-session message — there is no devtools one-shot
  command. A fixture script + the existing `{ type: "run", path }` stdin
  command (usage precedent: `scripts/agentic/scenario.ts:8441-8445`) opens it.
- The PTY always spawns the user's interactive `$SHELL` (never `-c`) and
  TYPES the initial command + `\r` into it —
  `src/terminal/alacritty/handle_creation.rs:93-94,165-172`, shell detection
  :187-196. Environment is scrubbed to an allowlist (`TERM=xterm-256color`,
  `COLORTERM`, `CLICOLOR_FORCE`, `PROMPT_EOL_MARK=""`, + HOME/USER/PATH/
  SHELL/TMPDIR/LANG, zsh `ZDOTDIR` shim) —
  `src/terminal/pty/lifecycle.rs:115-152`.
- Determinism limits (honest): the FIRST prompt (user's zsh theme) and the
  echo of the initial command are user-specific. The fixture erases them with
  `clear` and `exec`s a PS1-pinned `/bin/sh`, after which every glyph is
  deterministic. `setInput` does NOT route to `AppView::TermPrompt`
  (`src/prompt_handler/mod.rs:9907-9928` supports QuickTerminalView only), so
  the visible command line must be typed through the key path —
  `simulateGpuiEvent` keyDown is the real-dispatch automation path (repo
  keyboard-routing contract); whether legacy `simulateKey` also reaches
  `TermPrompt::handle_key` is an open verification item.

### Fixture script (proposed: `tests/fixtures/terminal-prompt-fixture.ts`)
```ts
// Deterministic SDK terminal fixture for design-reference capture.
await term({ command: "clear; exec env PS1='$ ' /bin/sh" });
```

### Proposed `scripts/agentic/design-reference-capture.ts --screen terminal`
```ts
// DEFAULT_OUT.terminal = "design/mockups/screens/terminal-prompt/reference/terminal-prompt@2x.png"
// CAPTURE_TARGET.terminal = { type: "kind", kind: "main" }
if (screen === "terminal") {
  await driver.request(
    { type: "run", path: join(PROJECT_ROOT, "tests/fixtures/terminal-prompt-fixture.ts") } as never,
    { timeoutMs: 8_000 },
  ).catch(() => {});
  await driver.waitForSettle();
  await Bun.sleep(1_500); // PTY cold start: shell spawn + clear + exec

  // Type one deterministic command through the REAL key dispatch path
  // (simulateGpuiEvent keyDown per char; setInput does not reach TermPrompt).
  const cmd = String.raw`printf '\033[1;32m%s\033[0m %s\n' 'Script Kit' 'terminal ready'`;
  for (const ch of cmd) await driver.simulateGpuiKeyDown(ch);
  await driver.simulateGpuiKeyDown("enter");
  await driver.waitForSettle();
  await Bun.sleep(600);

  // Fail closed: currentView must be "term" and the PS1-pinned prompt visible.
  const state = (await driver.getState()) as { currentView?: string };
  if (state.currentView !== "term") throw new Error("term prompt not active");
}
```
Expected deterministic screen (mirrored by the mockup):
```
$ printf '\033[1;32m%s\033[0m %s\n' 'Script Kit' 'terminal ready'
Script Kit terminal ready        ← "Script Kit" bold, ANSI green #50fa7b
$ █                              ← block cursor, accent #fbbf24
```
Residual nondeterminism: none in glyph content after `clear`+`exec` (sh
builtin `printf`, PS1/TERM pinned, focused window required to avoid the ×0.7
unfocused dim). If typing via automation proves impossible, fallback fixture:
`term({ command: "clear; exec env PS1='$ ' /bin/sh" })` alone (prompt + cursor
only, no command/output rows) — still deterministic, less illustrative.

---

## 3. Proposed `src/design_contract/mod.rs` section

Token name → Rust resolver expression (stage in parens):

| Token | Resolver | Stage |
|---|---|---|
| `terminal.window.height` | `f32::from(window_resize::layout::MAX_HEIGHT)` | resolved |
| `terminal.entity.height` | `f32::from(layout::MAX_HEIGHT) - layout::FOOTER_HEIGHT` | resolved |
| `terminal.grid.fontSize` | `Config::default().get_terminal_font_size()` | source |
| `terminal.grid.cellWidth` | `term_prompt::conservative_cell_width(PINNED_FONT_MONO_ADVANCE_14PT /* 8.4287 */)` | resolved |
| `terminal.grid.cellHeight` | `font_size * term_prompt::LINE_HEIGHT_MULTIPLIER` | resolved |
| `terminal.grid.paddingTop/Left/Right` | `Config::default().get_padding().{top,left,right}` | source |
| `terminal.grid.paddingBottom` | `= paddingTop` (`effective_padding_bottom`) | resolved |
| `terminal.color.foreground` | `theme.colors.terminal.foreground.unwrap_or(theme.colors.text.primary)` | resolved |
| `terminal.color.cursorBackground` | `theme.colors.accent.selected` | resolved |
| `terminal.color.selectionBackground` | `(theme.colors.accent.selected_subtle << 8) \| 0x0f` | resolved |
| `terminal.ansi.{black..brightWhite}` (16) | `theme.colors.terminal.{...}` | source |
| `terminal.hint.height` | `window_resize::main_layout::HINT_STRIP_HEIGHT` | source |
| `terminal.hint.paddingX/Y` | `main_layout::HINT_STRIP_PADDING_{X,Y}` | source |
| `terminal.hint.gap` | `hint_strip::HINT_STRIP_CONTENT_GAP` | source |
| `terminal.hint.textSize` | `hint_strip::FOOTER_HINT_TEXT_SIZE` | source |
| `terminal.hint.textColor` | `text.primary @ main_layout::HINT_TEXT_OPACITY` (= existing `--sk-footer-text`) | resolved |
| `terminal.hint.keycapPaddingX/Y`, `.keycapRadius` | `hint_strip::KEYCAP_*` | source |
| `terminal.hint.keycapBackground` | `text.primary.with_opacity(KEYCAP_BG_OPACITY)` | resolved |
| `terminal.hint.iconSize`, `.iconLabelGap` | `hint_strip::KEY_ICON_SIZE`, `KEY_ICON_LABEL_GAP` | source |
| `terminal.contract.lineHeightOverridesPhi` | `"true"` (decision lock: 1.3, not φ) | source |
| `terminal.contract.footerVisible` | `"false"` pending runtime proof (conflict below) | resolved |
| `terminal.header.paddingX/Y` | reuse existing header tokens (`HEADER_PADDING_X/Y`) — dedupe with `confirm.header.*` into a shared `panelHeader.*` if the exporter grows a third consumer | resolved |

CSS var mapping: `terminal.grid.cellWidth` → `--sk-terminal-cell-width`, etc. —
the exact `--sk-terminal-*` names already referenced by
`screens/terminal-prompt/screen.css` (aliases delete cleanly on arrival).

## 4. Resolver-extraction plan

- `src/term_prompt/mod.rs` — `conservative_cell_width` is already pure
  (:61-68); make it `pub(crate)` and add a pure
  `pub(crate) fn terminal_cell_metrics(font_size: f32, measured_advance: Option<f32>) -> (f32 /*w*/, f32 /*h*/)`
  encapsulating `cell_width()`/`cell_height()` (:625-651) so
  `TermPrompt::render`, `resize_if_needed`, `pixel_to_cell`, AND the exporter
  share one implementation. Signature keeps `measured_advance` optional; the
  exporter passes the pinned 8.4287 measurement, the renderer passes the live
  text-system value.
- `src/terminal/theme_adapter/impls.rs::ThemeAdapter::from_theme` (:15-59) is
  already a pure `&Theme → colors` resolver — the exporter should call it
  directly (plus `AnsiColors::get`) instead of re-deriving palette math.
- `src/design_contract/mod.rs` — add
  `fn resolve_terminal_prompt_tokens(theme: &Theme, config: &Config) -> BTreeMap<String, TokenRecord>`
  following the confirm section's pattern, and register its conflicts (below).

## 5. Expected conflicts (record, don't collapse)

1. `terminal.footerClip.sourceMathVsPaintedTruth` — hint strip band (shell-y
   664-700) falls past the ~662 clip line (header 38 unbudgeted;
   FOOTER_HEIGHT 30 ≠ HINT_STRIP_HEIGHT 36). Severity: warning. Verify with
   the first capture; if the strip IS visible in reality, the layout model in
   this brief is wrong and must be corrected before locking tokens.
2. `terminal.cellWidth.fallbackVsMeasured` — first frame uses
   `BASE_CELL_WIDTH 8.5 × (size/14)` until `em_advance` is measured (8.43).
   Captures must settle past first render. Severity: info.
3. `terminal.ansi.themePaletteVsAdapterDefaults` — `theme/types.rs:722-745`
   (themed path, e.g. green `#50fa7b`) vs `terminal/theme_adapter.rs:37-53`
   fallback palette (green `#0dbc79`). App path always themes; the fallback
   only fires with no theme. Severity: info.
4. `terminal.hint.paintOrderQuirk` — footer paints
   `Actions · Esc Cancel · ⌘W Close [⌘][K]` because the three hints are
   pre-joined into one string before `parse_hint`. Moot while the strip is
   clipped; becomes user-visible if conflict 1 resolves to "visible".
5. `terminal.windowHeight.tokensVsMainMenu` — existing
   `--sk-window-main-height` (480) does not apply; terminal is a
   full-content view at 700. New token required (already proposed).
6. Protocol layout models may misreport this surface (confirmed pattern on
   confirm: `confirmLayout.protocolModelVsRendererTruth` in tokens.json) —
   trust renderer source + pixels for the terminal too.

## 6. Tokens to generate (`--sk-terminal-*`)

Dimension/typography (9): `window-height`, `entity-height`, `font-size`,
`cell-width`, `cell-height`, `padding-top`, `padding-bottom`, `padding-left`,
`padding-right`.
Colors (20): `cursor-background`, `selection-background`, `text` (alias of
`--sk-color-text-primary` unless a theme sets `terminal.foreground`),
`hint-text` (alias of `--sk-footer-text`), plus 16 × `ansi-*`
(`ansi-black` … `ansi-bright-white`).
Hint strip (10): `hint-height`, `hint-padding-x`, `hint-padding-y`,
`hint-gap`, `hint-text-size`, `hint-icon-size`, `hint-icon-gap`,
`hint-keycap-padding-x`, `hint-keycap-padding-y`, `hint-keycap-radius`,
`hint-keycap-background` (11 counting keycap bg).
Header (2, dedupe candidates): `header-padding-x`, `header-padding-y`.

**Total: 42** (38 new magnitudes + 4 aliases of existing tokens). The mockup
currently declares the subset it paints (25 aliases; the unused ANSI entries
land with the exporter).

## 7. Open unknowns

- Runtime proof of the clipped hint strip (conflict 1) — top priority.
- Exact automation primitive for typing into `TermPrompt` (simulateGpuiEvent
  keyDown assumed; legacy `simulateKey` routing unverified).
- Whether `Driver` exposes a per-char keyDown helper or needs a small addition
  for the capture recipe.
- JetBrains Mono availability in the mockup-viewing browser (falls back to
  ui-monospace; raster divergence noted).
