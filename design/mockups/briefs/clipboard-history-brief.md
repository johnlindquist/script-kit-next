# Clipboard History — design-contract mockup brief

Screen slice for `design/mockups/screens/clipboard-history/`. Fixture: stock
`script-kit-dark`, InfoBarBase, `MainWindowMode::Full`, 6 seeded entries,
row 0 (pinned) selected.

> A reference capture landed mid-draft at
> `screens/clipboard-history/reference/clipboard-history@2x.png` (2026-07-11,
> 1500×960 @2x = 750×480 — confirming the window tokens). The mockup fixture
> mirrors its content (pinned `npm install -g mdflow@next` selected; link and
> color rows with no icon slot — pixel-confirming the no-icon branches; empty
> native footer band — confirming the overlay truth). Two capture-vs-source
> mismatches were found and recorded (placeholder text, preview label
> dimness); see §5. Every claim below is tagged **MEASURED** (read from
source, with file:line) or **GUESS** (needs a runtime receipt before it may be
treated as contract).

Builtin id `builtin/clipboard-history` registered at `src/builtins/mod.rs:745-752`
(feature `BuiltInFeature::ClipboardHistory`, icon `"clipboard"`).
Renderer: `ScriptListApp::render_clipboard_history`, `src/render_builtins/clipboard.rs:196`.
Preview pane: `src/render_builtins/clipboard_preview.rs:3`.

---

## 1. Surface anatomy

### Window / shell
- **MEASURED** Window 750×480: `MAIN_WINDOW_WIDTH = 750.0` (`src/window_resize/mod.rs:291`),
  `main_window_full_height()` (`src/window_resize/mod.rs:45`) — locked at 480 by the
  contract test `src/design_contract/mod.rs:2058-2059`. Opening the view calls
  `open_builtin_filterable_view_with_filter(..., expanded: true)` →
  `MainWindowMode::Full` + `resize_to_view_sync(ViewType::MainWindow, 0)`
  (`src/app_execute/builtin_execution.rs:3539-3561`, helper contract comment
  `:3352-3363`: "wide full-width window for info-heavy views like clipboard
  history"). Radius/vibrancy/footer host = the already-generated
  `--sk-window-*` tokens.
- **MEASURED** Shared main-view chrome: `render_main_view_chrome_footer_flush` +
  `render_main_view_input_shell` + clickable context zone
  (`src/render_builtins/clipboard.rs:799-863`), locked by the source audit
  `clipboard_history_uses_shared_main_view_chrome` (`clipboard.rs:877-942`).
  Header = padding-y 4 + context 22 + gap 2 + search 26 → 58px
  (all existing `--sk-main-menu-*` tokens). Divider chrome `visible: false`
  (`clipboard.rs:816-820`).
- **MEASURED** Search input = the shared main filter input
  (`self.render_search_input()`, `clipboard.rs:806`) — gpui-component Input at
  InfoBarBase search tokens: height 26, line-height 26, font 20
  (`src/render_builtins/common.rs:48-61`; tokens `main_menu_theme.rs:734-743`).
  Placeholder text: source sets `"Search clipboard history..."`
  (`src/app_execute/builtin_execution.rs:3557`) but the 2026-07-11 reference
  paints the ROOT launcher placeholder — capture-vs-source conflict (§5 #8);
  the mockup mirrors the capture.
- **GUESS** Caret geometry on this input: the mockup reuses the main-menu caret
  tokens (`--sk-caret-width/height` = panel.rs CURSOR_*), but the gpui-component
  `InputState` caret may differ from the launcher's custom caret (memory:
  "main filter caret = gpui-component InputState"). Verify against the capture.

### Main region — split
- **MEASURED** `#clipboard-history-root` is `flex flex_row h_full w_full` with two
  `flex_1` children (list pane, preview pane) → exact 50/50 split, 375px each
  at 750 (`src/render_builtins/clipboard.rs:821-845`).

### Left pane — list
- **MEASURED** List pane has `py(design_spacing.padding_xs)` = **4px**
  (`clipboard.rs:693`; `DesignSpacing::default().padding_xs = 4.0`,
  `src/designs/traits/spacing.rs:52`).
- **MEASURED** Persistent leading section separator (POLISH.md §2, commit
  9bd506f5e): `render_section_header(filter.is_empty() ? "Clipboard" : "Results",
  None, list_colors, true)` (`clipboard.rs:753-769`). It renders through
  `resolved_list_item_metrics()` = `ListItemMetricsOverride::default_main_menu()`
  (`src/list_item/mod.rs:2410`, `:351-358`) — the **legacy** metrics, NOT the
  InfoBarBase list tokens: first-slot height = `SECTION_HEADER_HEIGHT 32 −
  MAIN_MENU_SECTION_PADDING_TOP 12 / 2` = **26px**, padding-top **6**,
  padding-x **14**, padding-bottom **4**, `justify_start`
  (`list_item/mod.rs:107,166-172,2463-2488`). Label 12px SEMIBOLD
  (`:203,209`), color text-primary at the muted alpha (`:2428` →
  `--sk-text-muted` rgb(255 255 255 / 0.647)). ⚠ differs from the main menu's
  generated 28px slot — see conflicts.
- **MEASURED** Rows: `uniform_list` of shared `ListItem`s (`clipboard.rs:515-668`).
  `ListItem::new` never sets `.main_menu_theme(...)` here, so metrics resolve
  from `MainMenuThemeVariant::default()` = `InfoBarBase`
  (`list_item/mod.rs:1396,1656-1658`; default at
  `src/designs/core/main_menu_theme.rs:18-22`): **row height 44**
  (`main_menu_theme.rs:745`), outer padding 4/0, inner 14/4, radius 14,
  icon-text gap 8.4, name-desc gap ≈2.22 (`:642-666`), icon container 20 /
  svg 16 / tile radius 7 (`:815-822`), name 14/16 weight 450→500 selected,
  desc 12/16 (`:823-833`). All already generated as `--sk-main-menu-*`.
- **MEASURED** Fills via the shared resolver `resolved_main_menu_row_fill`
  (`list_item/mod.rs:296-348`, called at `:1725`): selected `#FFFFFF20`,
  hover `#FFFFFF12`, icon tile `#FBBF24F2` radius 7 — byte-locked by
  `src/design_contract/mod.rs:2047-2054`.
- **MEASURED** Row content (`clipboard.rs:534-565`):
  - Name = `entry.display_preview()`; pinned entries get a `"📌 "` prefix
    (`clipboard.rs:543-550`). `display_preview` flattens newlines and
    truncates >50 chars with `"…"`; image entries show `"{w}×{h} image"` or
    `"[Image]"` (`crates/sk-clipboard/src/types.rs:92-110`).
  - Description = short relative time (`format_relative_time_short_millis`,
    `src/formatting.rs:59-64`; examples "just now", "3 minutes ago",
    "2 hours ago" `:53-57`), painted only when selected/hovered (progressive
    disclosure, `list_item/mod.rs:1945-1951`), selected color = text-primary
    at muted alpha (`--sk-text-muted`).
  - Icon: Text rows get the `"📄"` emoji, image rows get the cached thumbnail
    (`clipboard.rs:560-565`); **Link/File/Color rows and uncached-image rows
    mount NO icon slot at all** (`list_item/mod.rs:2109-2115` — a zero-size
    placeholder would consume the flex gap). Emoji renders `.text_sm` = **14px**
    inside the 20×20 container (`list_item/mod.rs:1760-1769`;
    `text_sm` = 0.875rem, `vendor/gpui/src/styled.rs:513`).
  - Selected + icon → 20×20 accent tile `#FBBF24F2`, radius 7 (IconTile row
    kind, `list_item/mod.rs:1812-1825`).
- **MEASURED** Sort order: `pinned DESC, timestamp DESC`
  (`src/clipboard_history/database.rs:528,623`); dataset =
  `get_cached_entries(100)` snapshotted on open
  (`src/app_execute/builtin_execution.rs:3543`).
- **MEASURED** Scrollbar: `builtin_uniform_list_scrollbar(handle, len, 8)`
  (`clipboard.rs:669-673`) — transient overlay, omitted in the mockup
  (known-divergence).

### Right pane — preview panel (`src/render_builtins/clipboard_preview.rs`)
All colors resolve from raw theme fields + hand alphas; with stock
`script-kit-dark` (`src/theme/presets.rs:946-975`: main `#0F0F0F`, search_box
`#2A2A2A`, border `#343434`, **every text color `#FFFFFF`**) and dark-default
`opacity.main = 0.50` (`src/theme/types.rs:253-262`):
- **MEASURED** Panel: `bg = background.main @ (opacity.main×255×0.30) as u8` =
  `#0F0F0F` α 38 → `rgb(15 15 15 / 0.1490196078)` (`clipboard_preview.rs:20,27`);
  `border_l_1` = `ui.border @ ALPHA_BORDER_SUBTLE 0x30` →
  `rgb(52 52 52 / 0.1882352941)` (`:28-31`; `src/ui_foundation/mod.rs:102`);
  padding = `padding_lg` **16** (`:32`; `spacing.rs:56`); flex-col gap =
  `gap_sm` **4** (`:35`; `spacing.rs:58`).
- **MEASURED** Content block (text/link/file/color): padding `padding_md` **12**,
  radius `radius_md` **8** (`clipboard_preview.rs:120-121`;
  `src/designs/traits/visual.rs:68`), bg = `search_box @
  (opacity.main×255×0.40) as u32` = `#2A2A2A` α 51 → `rgb(42 42 42 / 0.2)`
  (`:21,122`), text = mono family, `.text_sm` **14px**, text-primary
  (`:126-130`). Line height is GPUI's implicit phi: round(14 × 1.618034) =
  **23px** (`vendor/gpui/src/style.rs` `line_height_in_pixels`; same rule the
  confirm screen pixel-validated). Content = full entry payload via
  `get_entry_content` with a cached-preview fallback note (`:94-113`).
- **MEASURED** Information block: `border_t_1` = `ui.border @ ALPHA_DIVIDER 0x60`
  → `rgb(52 52 52 / 0.3764705882)` (`clipboard_preview.rs:66-69`;
  `ui_foundation/mod.rs:100`), padding-top `padding_md` **12** (`:70`), gap
  `gap_sm` **4** (`:73`). Heading "Information": `.text_xs` **12px** SEMIBOLD,
  `rgb(text_secondary)` (`:75-79`). Rows `label → value`: both `.text_xs`,
  `justify_between`, gap `gap_md` **8** (`:50-60`); label `rgb(text_muted)`,
  value `rgb(text_primary)` — **all pure white under the stock preset** (see
  conflict 4). Info line height = phi: round(12 × 1.618034) = **19px**.
  Text rows add Characters + Lines (`:100-104`).
- **MEASURED** Image branch (not in the default fixture): container padding
  `padding_lg` 16, bg `search_box @ 0.15×opacity.main` →
  `rgb(42 42 42 / 0.0745098039)`, image max 300×300 contain, radius_sm 4
  (`clipboard_preview.rs:134-199`).
- **MEASURED** Fixture preview strings for row 0 (content
  `"deploy checklist:\n- bump version\n- tag + push"`): Type "Text", Size
  "45 bytes", Pinned "Yes", Characters "45", Lines "3"; Copied =
  `relative · absolute` (`clipboard_preview.rs:81-87`; absolute format
  `"%b %-d, %Y at %-I:%M %p"`, `src/formatting.rs:66-71`) — time-dependent,
  non-blocking.

### Footer
- **MEASURED** Hints: `universal_prompt_hints_with_primary_label("Paste")` →
  `["↵ Paste", "⌘K Actions", "⌘↵ Agent"]` (`clipboard.rs:786`;
  `src/components/prompt_layout_shell.rs:718-736`;
  `AGENT_CHAT_CMD_ENTER_HINT = "⌘↵ Agent"`,
  `src/ai/agent_chat/ui/labels.rs:2`), routed through
  `main_window_footer_slot(render_simple_hint_strip(...))` (`clipboard.rs:789-790`).
- **GUESS** Native AppKit footer overlay button labels/order
  (`Paste ↵ · Actions ⌘K · Agent ⌘↵` in the mockup): the real footer is a
  separate overlay window (36px host, 32px rail — cross-cutting truth,
  receipt-verified for main-menu/confirm) and the GPUI capture band is EMPTY.
  Run a protocol `activeFooter` probe on this surface before treating the
  labels as contract.

---

## 2. Fixture strategy

### Seeding SQL (sandboxed `$HOME/.scriptkit/db/clipboard-history.sqlite`)
Schema matches the proven recipe at
`scripts/agentic/root-source-filter-clipboard.ts:111-162` (`history` table;
startup migrations add the remaining columns, `database.rs:82-171`). Content
hashes MUST be real SHA-256 hex of the content (`compute_content_hash`,
`database.rs:41-45`) or the startup dedup breaks (see determinism below).

```sql
CREATE TABLE IF NOT EXISTS history (
  id TEXT PRIMARY KEY, content TEXT NOT NULL, content_hash TEXT,
  content_type TEXT NOT NULL DEFAULT 'text', timestamp INTEGER NOT NULL,
  pinned INTEGER DEFAULT 0, ocr_text TEXT, text_preview TEXT,
  image_width INTEGER, image_height INTEGER, byte_size INTEGER
);
DELETE FROM history;
-- :now = Date.now() at seed time; :sha(x) = sha256 hex of x (compute in bun)
INSERT INTO history VALUES
 ('clip-pinned',  'deploy checklist:' || char(10) || '- bump version' || char(10) || '- tag + push',
   :sha1, 'text',  :now -  2*60*1000, 1, NULL,
   'deploy checklist:' || char(10) || '- bump version' || char(10) || '- tag + push', NULL, NULL, 45),
 ('clip-short',   'Fix the flaky clipboard scroll test',
   :sha2, 'text',  :now -  9*60*1000, 0, NULL, 'Fix the flaky clipboard scroll test', NULL, NULL, 35),
 ('clip-long',    'const rows = entries' || char(10) || '  .filter((entry) => entry.pinned)' || char(10) || '  .map((entry) => entry.id);',
   :sha3, 'text',  :now - 31*60*1000, 0, NULL, 'const rows = entries' || char(10) || '  .filter((entry) => entry.pinned)' || char(10) || '  .map((entry) => entry.id);', NULL, NULL, 84),
 ('clip-url',     'https://scriptkit.com/docs/clipboard-history',
   :sha4, 'link',  :now -  2*3600*1000, 0, NULL, 'https://scriptkit.com/docs/clipboard-history', NULL, NULL, 44),
 ('clip-sql',     'SELECT id, text_preview FROM history ORDER BY pinned DESC, timestamp DESC;',
   :sha5, 'text',  :now -  4*3600*1000, 0, NULL, 'SELECT id, text_preview FROM history ORDER BY pinned DESC, timestamp DESC;', NULL, NULL, 75),
 ('clip-image',   'blob:fixture-missing',
   :sha6, 'image', :now - 26*3600*1000, 0, NULL, NULL, 1280, 800, 245760);
```

Notes: the `link` row exercises the no-icon branch; `clip-image` deliberately
points at a missing blob so the row shows `"1280×800 image"` with no thumbnail
and the preview (if selected) shows the "Image preview loading" placeholder —
fully deterministic without shipping a binary asset. `display_preview`
truncation makes `clip-long`/`clip-sql` render as 50 chars + `…`.

The reference PNG that landed mid-draft used a sibling fixture (pinned
`npm install -g mdflow@next`, a truncated design-tokens sentence, meeting
notes, `https://scriptkit.com/downloads`, `const answer = 42;`, and a
`#FBBF24` color row — no image row). The mockup mirrors THAT content for
pixel registration; whichever fixture is canonicalized, keep one `link` row,
one `color` row (both no-icon), one >50-char truncation, one pinned+selected
row, and — if the image branch should stay covered — add the `clip-image`
entry above and re-capture.

### Determinism vs the clipboard monitor
`init_clipboard_history()` runs unconditionally at startup
(`src/main_entry/app_run_setup.rs:169`) and the monitor's **first poll always
captures the live pasteboard** (`ClipboardChangeDetector::has_changed` returns
`Some(true)` on first check, `src/clipboard_history/change_detection.rs:70-78`;
capture loop `monitor.rs:191-222`). In a fixture home this inserts one
nondeterministic stranger row at the top. Mitigation (no code changes):
**pbcopy a seeded entry's exact content before launch** — the capture then
dedups via `(content_type, content_hash)` and merely refreshes that entry's
timestamp (`database.rs:352-416`). Use `clip-short` (not the pinned row) so the
selected row's "2 minutes ago" stays stable and ordering is unchanged
(pinned-first). The copied text must not trip the secret-pattern rejection
(`monitor.rs:243-251`, `rejection.rs`). This is exactly why the hashes must be
real SHA-256.

### Opening the surface
`{"type":"triggerBuiltin","builtinId":"builtin/clipboard-history","protocolVersion":2}`
is a proven ingress form (`src/protocol/ingress.rs:149`; alias
`{"name":"clipboardHistory"}` `:159`). Direct-open semantics: escape closes the
window (stability-CLS eval receipts). Selection defaults to index 0
(`builtin_execution.rs:3560-3561`), which is the pinned row — no key presses
needed.

### Proposed block for `scripts/agentic/design-reference-capture.ts`
⚠ Reality check: the current script launches with **`sandboxHome: false`**
(`design-reference-capture.ts:42-47`), not `sandboxHome: true` as the tasking
assumed. Also `Driver.launch` **`rmSync`s its session dir before spawning**
(`scripts/devtools/driver.ts:541-548`), so a `sandboxHome: true` home cannot be
pre-seeded. The deterministic recipe is therefore `sandboxHome: false` plus an
explicit `env.HOME` override (base env spreads `process.env` then
`options.env`, `driver.ts:550-558`):

```ts
// --screen clipboard additions
DEFAULT_OUT.clipboard =
  "design/mockups/screens/clipboard-history/reference/clipboard-history@2x.png";
CAPTURE_TARGET.clipboard = { type: "kind", kind: "main" };

// Before Driver.launch, when screen === "clipboard":
import { mkdirSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
const fixtureHome = join(PROJECT_ROOT, ".test-output/design-capture-clipboard/home");
const dbDir = join(fixtureHome, ".scriptkit", "db");
mkdirSync(dbDir, { recursive: true });
// keep root passive search quiet, same as root-source-filter-clipboard.ts:114-117
writeFileSync(join(fixtureHome, ".scriptkit", "config.ts"),
  "export default { unifiedSearch: { clipboardHistory: { enabled: false } } };\n");
const sha = (s: string) => createHash("sha256").update(s).digest("hex");
const sql = buildSeedSql(Date.now(), sha); // the SQL block above
spawnSync("sqlite3", [join(dbDir, "clipboard-history.sqlite")], { input: sql });
// startup-capture determinism: monitor's first poll dedups into clip-short
spawnSync("pbcopy", [], { input: "Fix the flaky clipboard scroll test" });

const driver = await Driver.launch({
  sessionName: "design-reference-capture-clipboard",
  sandboxHome: false,
  env: { HOME: fixtureHome, SK_PATH: join(fixtureHome, ".scriptkit") },
  defaultTimeoutMs: 10_000,
});
// after show + settle:
await driver.request({
  type: "triggerBuiltin", builtinId: "builtin/clipboard-history",
} as never, { timeoutMs: 5_000 });
await driver.waitForSettle();
await Bun.sleep(600); // image-cache prewarm + relative-time strings settle
// then the existing captureScreenshot({ hiDpi: true, target, savePath }) path,
// PLUS an activeFooter probe recorded next to the PNG (footer band is empty
// in the GPUI capture — receipts are the footer truth).
```

---

## 3. Proposed `src/design_contract/mod.rs` section

Mirror the actions/confirm pattern (resolved chrome helpers + `b.source_len` /
`b.resolved_color`), inside `checked_in_design_bundle()`:

```rust
// ── Clipboard history (builtin browser) ────────────────────────────────
let default_spacing = crate::designs::get_tokens(crate::designs::DesignVariant::Default).spacing();
let default_visual  = crate::designs::get_tokens(crate::designs::DesignVariant::Default).visual();
let preview = crate::render_builtins::resolved_clipboard_preview_chrome(
    &theme, &default_spacing, &default_visual); // new shared resolver, §4
let section = crate::list_item::resolved_builtin_section_header_slot(true); // new, §4

b.source_len("clipboard.listPane.paddingY", "--sk-clipboard-list-pane-padding-y",
    default_spacing.padding_xs, "DesignSpacing.padding_xs (clipboard.rs list_pane py)");
b.add("resolved.clipboard.firstSectionSlotHeight",
    TokenStage::Resolved, Some("--sk-clipboard-first-section-slot-height"),
    TokenValue::Length { value: section.height as f64 }, None, false,
    &["crate::list_item::SECTION_HEADER_HEIGHT", "MAIN_MENU_SECTION_PADDING_TOP"]);
b.add("resolved.clipboard.firstSectionPaddingTop",
    TokenStage::Resolved, Some("--sk-clipboard-first-section-padding-top"),
    TokenValue::Length { value: section.padding_top as f64 }, None, false,
    &["MAIN_MENU_SECTION_PADDING_TOP"]);
b.source_len("clipboard.row.emojiFontSize", "--sk-clipboard-row-emoji-font-size",
    14.0, "gpui Styled::text_sm (list_item emoji icon)"); // rems(0.875) × 16
b.resolved_color("resolved.clipboard.preview.background",
    "--sk-clipboard-preview-background", preview.panel_bg_rgba,
    &["theme.colors.background.main", "theme.opacity.main × 0.30"]);
b.resolved_color("resolved.clipboard.preview.border",
    "--sk-clipboard-preview-border", preview.panel_border_rgba,
    &["theme.colors.ui.border", "ui_foundation::ALPHA_BORDER_SUBTLE"]);
b.source_len("clipboard.preview.padding", "--sk-clipboard-preview-padding",
    default_spacing.padding_lg, "DesignSpacing.padding_lg");
b.source_len("clipboard.preview.gap", "--sk-clipboard-preview-gap",
    default_spacing.gap_sm, "DesignSpacing.gap_sm");
b.source_len("clipboard.preview.contentPadding", "--sk-clipboard-preview-content-padding",
    default_spacing.padding_md, "DesignSpacing.padding_md");
b.source_len("clipboard.preview.contentRadius", "--sk-clipboard-preview-content-radius",
    default_visual.radius_md, "DesignVisual.radius_md");
b.resolved_color("resolved.clipboard.preview.contentBackground",
    "--sk-clipboard-preview-content-background", preview.content_bg_rgba,
    &["theme.colors.background.search_box", "theme.opacity.main × 0.40"]);
b.source_len("clipboard.preview.contentFontSize", "--sk-clipboard-preview-content-font-size",
    14.0, "gpui Styled::text_sm");
b.add("resolved.clipboard.preview.contentLineHeight",
    TokenStage::Resolved, Some("--sk-clipboard-preview-content-line-height"),
    TokenValue::Length { value: preview.content_line_height as f64 }, None, false,
    &["gpui TextStyle default phi() line height, rounded (14 → 23)"]);
b.resolved_color("resolved.clipboard.preview.infoDivider",
    "--sk-clipboard-preview-info-divider", preview.info_divider_rgba,
    &["theme.colors.ui.border", "ui_foundation::ALPHA_DIVIDER"]);
b.source_len("clipboard.preview.infoPaddingTop", "--sk-clipboard-preview-info-padding-top",
    default_spacing.padding_md, "DesignSpacing.padding_md");
b.source_len("clipboard.preview.infoGap", "--sk-clipboard-preview-info-gap",
    default_spacing.gap_sm, "DesignSpacing.gap_sm");
b.source_len("clipboard.preview.infoRowGap", "--sk-clipboard-preview-info-row-gap",
    default_spacing.gap_md, "DesignSpacing.gap_md");
b.source_len("clipboard.preview.infoFontSize", "--sk-clipboard-preview-info-font-size",
    12.0, "gpui Styled::text_xs");
b.add("resolved.clipboard.preview.infoLineHeight",
    TokenStage::Resolved, Some("--sk-clipboard-preview-info-line-height"),
    TokenValue::Length { value: preview.info_line_height as f64 }, None, false,
    &["gpui TextStyle default phi() line height, rounded (12 → 19)"]);
b.resolved_color("resolved.clipboard.preview.infoLabelColor",
    "--sk-clipboard-preview-info-label-color", preview.info_label_rgba,
    &["theme.colors.text.muted (full alpha — ladder bypass, see conflict)"]);
b.resolved_color("resolved.clipboard.preview.infoValueColor",
    "--sk-clipboard-preview-info-value-color", preview.info_value_rgba,
    &["theme.colors.text.primary"]);
b.resolved_color("resolved.clipboard.preview.infoHeadingColor",
    "--sk-clipboard-preview-info-heading-color", preview.info_heading_rgba,
    &["theme.colors.text.secondary (full alpha)"]);
b.add("clipboard.preview.infoHeadingFontWeight",
    TokenStage::Source, Some("--sk-clipboard-preview-info-heading-font-weight"),
    TokenValue::FontWeight { value: gpui::FontWeight::SEMIBOLD.0 as f64 },
    Some("FontWeight::SEMIBOLD in render_clipboard_preview_panel"), true, &[]);
```

Row/section tokens the mockup already consumes stay on the existing
`--sk-main-menu-*` exports (same `ListItem` contract, byte-identical fills).

---

## 4. Resolver-extraction plan (renderer ↔ exporter cannot drift)

1. **`src/render_builtins/clipboard_preview.rs`** — extract the inline alpha
   math (`clipboard_preview.rs:19-22,27-31,66-69,122,192`) into a pure
   resolver, mirroring `crate::actions::resolved_actions_dialog_*`
   (consumed by the exporter at `design_contract/mod.rs:1105-1108`):
   ```rust
   pub struct ClipboardPreviewChrome {
       pub panel_bg_rgba: u32, pub panel_border_rgba: u32,
       pub content_bg_rgba: u32, pub image_bg_rgba: u32,
       pub info_divider_rgba: u32,
       pub info_label_rgba: u32, pub info_value_rgba: u32, pub info_heading_rgba: u32,
       pub content_line_height: f32, pub info_line_height: f32,
   }
   pub fn resolved_clipboard_preview_chrome(
       theme: &crate::theme::Theme,
       spacing: &crate::designs::DesignSpacing,
       visual: &crate::designs::DesignVisual,
   ) -> ClipboardPreviewChrome
   ```
   `render_clipboard_preview_panel` then paints exclusively from the struct.
2. **`src/list_item/mod.rs`** — extract the section-slot math duplicated in
   `render_section_header` (`:2463-2473`) into
   ```rust
   pub struct SectionHeaderSlot { pub height: f32, pub padding_top: f32,
       pub padding_x: f32, pub padding_bottom: f32 }
   pub fn resolved_builtin_section_header_slot(is_first: bool) -> SectionHeaderSlot
   ```
   so the exporter records the legacy 26/32 slots that builtin browsers
   actually paint (distinct from the InfoBarBase 28px launcher slots).
3. Row fills need nothing new — `resolved_main_menu_row_fill` is already the
   shared resolver (`list_item/mod.rs:286-296` doc, exporter call at
   `design_contract/mod.rs:241`).
4. Footer labels: record `universal_prompt_hints_with_primary_label("Paste")`
   output as a `Text` token (`clipboard.hints.primaryLabel = "Paste"`) so the
   fixture's footer copy is exporter-tracked.

---

## 5. Expected conflicts (record, never collapse)

1. `clipboardRow.uniformListFallback40VsPainted44` — the out-of-range fallback
   row is `h(px(LIST_ITEM_HEIGHT)) = 40` (`clipboard.rs:659`) while painted
   rows are 44 (InfoBarBase). Never paints in practice; info severity.
2. `clipboardSectionHeader.legacyMetricsVsInfoBarSlot` — the persistent leading
   separator paints the **legacy** 26px first slot / 6px padding-top
   (`resolved_list_item_metrics`, `list_item/mod.rs:2410,166-172`) while the
   launcher's generated tokens say 28px/4px
   (`--sk-main-menu-first-section-slot-height`). Same visual language, two
   metric sources.
3. `clipboardRowTheme.defaultVariantVsShellTheme` — rows resolve metrics from
   `MainMenuThemeVariant::default()` (`list_item/mod.rs:1396`) while the shell
   uses `self.current_main_menu_theme` (`clipboard.rs:799`). Benign today
   (`every_variant_preserves_base_non_header_geometry`,
   `main_menu_theme.rs:914-923`), but a future geometry-divergent variant
   would split shell vs rows on this screen.
4. `clipboardPreview.textLadderBypass` — the preview paints
   `rgb(text_muted)` / `rgb(text_secondary)` at FULL alpha
   (`clipboard_preview.rs:58-59,78`) and stock `script-kit-dark` sets every
   text color to `#FFFFFF` (`presets.rs:954-961`), so "muted" labels render
   pure white; the `AppChromeColors` opacity ladder is bypassed on this panel.
5. `clipboardPreview.designTokensVsMainMenuDef` — the preview pane consumes
   `DesignSpacing`/`DesignVisual` (`get_tokens(self.current_design)`,
   `clipboard.rs:206-209`) while the list pane consumes `MainMenuThemeDef`:
   two token systems on one screen; a `current_design` change restyles only
   the right half.
6. `clipboardFooter.hintStripVsNativeOverlay` — the renderer feeds a GPUI hint
   strip into `main_window_footer_slot` (`clipboard.rs:789-790`) but visible
   truth is the native AppKit overlay; the GPUI band captures empty. Footer
   verification must be a protocol `activeFooter` receipt.
7. `clipboardStartupCapture.livePasteboardVsFixtureDb` — the monitor's first
   poll always captures the live pasteboard (`change_detection.rs:70-78`),
   mutating fixture DBs at launch; mitigated only by the pbcopy-dedup recipe
   (§2), not by any code guarantee. Observed live in the 2026-07-11 reference:
   the pinned row's list description says "1 hour ago" while the preview says
   Copied "just now" — the startup capture refreshed the entry's timestamp
   after `cached_clipboard_entries` was snapshotted
   (`builtin_execution.rs:3543`), so the two panes disagree about the same
   entry.
8. `clipboardPlaceholder.configuredVsPainted` — `open_clipboard_history_surface*`
   sets `pending_placeholder = "Search clipboard history..."`
   (`builtin_execution.rs:3557` via the shared helper `:3372`), but the
   2026-07-11 reference paints the root launcher placeholder ("Search • @
   context • / commands • ; capture • : filters"). Either the pending
   placeholder never syncs into the gpui-component `InputState` on this path,
   or the capture's open path bypassed the helper. Needs a root-cause receipt;
   until then the painted (root) string is the mockup fixture.
9. `clipboardPreviewLabels.stockWhiteVsCaptureDim` — source resolution says
   info labels are pure white (`rgb(text_muted)` full alpha,
   `clipboard_preview.rs:58`; stock preset muted `#FFFFFF`,
   `presets.rs:954-961`; `normalize_dark_interactive_tokens` only rewrites
   `text.primary`, `presets.rs:858`), yet the capture shows clearly dimmed
   labels. Candidate causes: live theme file overriding the preset, or the
   focus-aware unfocused blend (`theme/types.rs:1066-1074`) active during the
   non-activating capture. Keep the stock token; treat capture hue as
   non-blocking until re-captured focused under the stock preset.

---

## 6. Tokens to generate (23 — every `--sk-clipboard-*` var the mockup uses)

All currently alias numerically-near existing tokens in
`screens/clipboard-history/screen.css`; the exporter section in §3 replaces
the aliases with real generated values. Intended resolved values (stock
profile):

| Token | Value | Source |
| --- | --- | --- |
| `--sk-clipboard-list-pane-padding-y` | 4px | `DesignSpacing.padding_xs` (`clipboard.rs:693`) |
| `--sk-clipboard-first-section-slot-height` | 26px | 32 − 12/2 (`list_item/mod.rs:166-167,385-386`) |
| `--sk-clipboard-first-section-padding-top` | 6px | 12/2 (`list_item/mod.rs:172`) |
| `--sk-clipboard-row-emoji-font-size` | 14px | `.text_sm` (`list_item/mod.rs:1766`) |
| `--sk-clipboard-preview-background` | rgb(15 15 15 / 0.1490196078) | main @ opacity.main×0.30 (`clipboard_preview.rs:20,27`) |
| `--sk-clipboard-preview-border` | rgb(52 52 52 / 0.1882352941) | border @ 0x30 (`:28-31`) |
| `--sk-clipboard-preview-padding` | 16px | `padding_lg` (`:32`) |
| `--sk-clipboard-preview-gap` | 4px | `gap_sm` (`:35`) |
| `--sk-clipboard-preview-content-padding` | 12px | `padding_md` (`:120`) |
| `--sk-clipboard-preview-content-radius` | 8px | `radius_md` (`:121`) |
| `--sk-clipboard-preview-content-background` | rgb(42 42 42 / 0.2) | search_box @ opacity.main×0.40 (`:21,122`) |
| `--sk-clipboard-preview-content-font-size` | 14px | `.text_sm` (`:128`) |
| `--sk-clipboard-preview-content-line-height` | 23px | phi(14) rounded |
| `--sk-clipboard-preview-info-divider` | rgb(52 52 52 / 0.3764705882) | border @ 0x60 (`:66-69`) |
| `--sk-clipboard-preview-info-padding-top` | 12px | `padding_md` (`:70`) |
| `--sk-clipboard-preview-info-gap` | 4px | `gap_sm` (`:73`) |
| `--sk-clipboard-preview-info-row-gap` | 8px | `gap_md` (`:57`) |
| `--sk-clipboard-preview-info-font-size` | 12px | `.text_xs` (`:58-59`) |
| `--sk-clipboard-preview-info-line-height` | 19px | phi(12) rounded |
| `--sk-clipboard-preview-info-label-color` | rgb(255 255 255) | `rgb(text_muted)` full alpha (`:58`) |
| `--sk-clipboard-preview-info-value-color` | rgb(255 255 255) | `rgb(text_primary)` (`:59`) |
| `--sk-clipboard-preview-info-heading-color` | rgb(255 255 255) | `rgb(text_secondary)` (`:78`) |
| `--sk-clipboard-preview-info-heading-font-weight` | 600 | `FontWeight::SEMIBOLD` (`:77`) |

---

## Not verified from source (runtime receipts required)

- Native footer button labels/order + selection state on this surface
  (`activeFooter` probe).
- The gpui-component input caret geometry/color vs the launcher caret tokens.
- Whether `resize_to_view_sync(ViewType::MainWindow, 0)` lands exactly 480 for
  this view on every display (contract test locks the token; a
  `getLayoutInfo`-vs-paint check is still worth one receipt — protocol layout
  models have lied before).

(Resolved during drafting: relative-time strings are long-form —
`format_relative_seconds` returns "just now" under 60s, else chrono-humanize
`Accuracy::Precise, Tense::Past` on the minute floor, e.g. "2 minutes ago";
`src/formatting.rs:153-161`. **MEASURED**.)
