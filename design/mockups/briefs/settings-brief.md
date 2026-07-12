# Settings screen brief (design-contract mockup slice)

Screen: the **Settings hub** — `AppView::SettingsView`, rendered by
`render_settings` in `src/render_builtins/settings.rs:430-722`. It is a
built-in list surface hosted in the main window and shares the InfoBarBase
main-view chrome (header + context zone + search shell + native footer) with
the main menu, so most of its contract is the already-generated
`--sk-main-menu-*` / `--sk-window-*` / `--sk-footer-*` token families.
Deliverables live in `design/mockups/screens/settings/`.

Tags: **MEASURED** = read from source at the cited line; **GUESS** = inference
that needs a runtime receipt.

---

## 1. Surface anatomy

### Window shell (shared with main menu — all tokens already generated)
- Window 750×480, radius 22, menu material + tint + effect stack. MEASURED —
  generated tokens `--sk-window-main-width/height/radius`
  (`design/mockups/generated/tokens.css:163-167`), same shell as main-menu
  mockup.
- Chrome entry: `render_main_view_chrome_footer_flush(render_main_view_shell() …)`
  at `src/render_builtins/settings.rs:695-722`; shell helpers at
  `src/components/main_view_chrome.rs:198,221`.

### Header (render_builtin_main_input_header)
- Built by `render_builtin_main_input_header` at
  `src/render_builtins/common.rs:97-111`: context zone
  (`render_clickable_main_view_context_zone`) above the shared input shell,
  with shell padding 2/4 and gap 2. MEASURED — InfoBarBase
  `MainMenuShellTokens { header_padding_x: 2.0, header_padding_y: 4.0,
  header_gap: 2.0, divider_height: 0.0 }` at
  `src/designs/core/main_menu_theme.rs:724-733`. Divider is therefore
  `visible: false` (`settings.rs:712-716`).
- Context zone: identical to main menu (22px row, mono 10.5px, pills).
  MEASURED — InfoBarBase `header_info_bar_tokens(Split, 0.50, 22.0, …)` at
  `src/designs/core/main_menu_theme.rs:479-482`; tokens
  `--sk-main-menu-context-*` already generated.
- Search shell: `render_main_view_input_shell` at
  `src/components/main_view_chrome.rs:784-842`. Height 26, radius 9,
  `surface_alpha: 0x00`, `border_alpha: 0x00` (invisible box), font 20 /
  weight 430, text inset left 16 / body right inset 16×0.5. MEASURED —
  `MainMenuSearchTokens` at `src/designs/core/main_menu_theme.rs:734-743`.
- Placeholder text: `"Search settings..."`. MEASURED —
  `open_builtin_filterable_view(AppView::SettingsView …, "Search settings...", false, cx)`
  at `src/app_execute/builtin_execution.rs:4364-4373` (string at 4370).
- Caret 2.5×18: shared `--sk-caret-*` tokens (main-menu contract).
- **Count label (settings-specific trailing element)**:
  `render_builtin_main_input_count_label(format!("{} setting{}", …))` at
  `src/render_builtins/settings.rs:705-711`; implementation at
  `src/render_builtins/common.rs:67-80` — `flex_none`, `whitespace_nowrap`,
  `pr(search.text_inset_x)` (= 16, `main_menu_theme.rs:737` →
  `SEARCH_INPUT_TEXT_INSET_X_PX`), `.text_sm()`, color
  `chrome.text_hint_rgba`. MEASURED:
  - Font size 14px: gpui `.text_sm()` = `rems(0.875)` at
    `vendor/gpui/src/styled.rs:513-516`, 16px rem → 14px.
  - Color: `AppChromeColors.text_hint_rgba` — the exact resolver already
    exported as `--sk-text-hint` (`src/design_contract/mod.rs:393-394`),
    `rgb(255 255 255 / 0.4470588235)` in `tokens.css:125`.
  - Text: `"11 settings"` with default config (see item census below).
  - GUESS (line box): count label line height uses gpui default phi
    (14 → 23px, `vendor/gpui/src/style.rs` line_height_in_pixels); vertically
    centered by the shell's `items_center`, so it does not affect layout.

### Content column (settings-specific)
- `div().flex().flex_col().flex_1().py(px(design_spacing.padding_xs))` at
  `src/render_builtins/settings.rs:657-664`. MEASURED — `padding_xs = 4.0`
  for the Default design variant at `src/designs/traits/spacing.rs:52`
  (locked by `src/designs/core/tests.rs:234`).
- **Persistent leading section header** (CLS rule, POLISH.md §2):
  `render_section_header(if filter.trim().is_empty() { "Settings" } else
  { "Results" }, None, list_colors, true)` at
  `src/render_builtins/settings.rs:665-681`. No icon. `is_first=true` →
  28px slot, `justify_start`, padding top 4 / bottom 4 / x 14. MEASURED —
  `render_section_header` at `src/list_item/mod.rs:2404-2500`
  (first-height/padding branch at 2481-2492); InfoBarBase
  `first_section_header_height: 28.0` at
  `src/designs/core/main_menu_theme.rs:747`; label color = text_primary at
  `alpha_muted`, 12px / weight 600 (generated `--sk-main-menu-section-*`,
  `--sk-text-muted`).
- Row stack: `render_tracked_scroll_column("settings-row-stack", …)` at
  `src/render_builtins/settings.rs:650-654` — plain flex column with the
  vendor overlay scrollbar (`src/components/scrollbar.rs:26-49`). GUESS: the
  scrollbar thumb is not modeled in the mockup (overlay style, appears on
  scroll; the capture may or may not include it).

### Rows (ListItem, iconless)
- Items built at `src/render_builtins/settings.rs:573-642`:
  `ListItem::new(name, colors).icon_kind_opt(IconKind::from_icon_hint(item.icon))
  .description_opt(Some(desc)).selected(is_selected).hovered(is_hovered)
  .with_accent_bar(is_selected)`.
- **Iconless painted truth (key finding)**: every authored icon hint
  (`palette`, `sliders-horizontal`, `mic`, `eraser`, `circle-check`,
  `shield-check`, `accessibility`, `monitor-check`, `key-round`,
  `square-split-horizontal`, `rotate-ccw` — `settings.rs:66-147`) fails
  `IconKind::from_icon_hint` (`src/list_item/mod.rs:37-52`), because
  `icon_name_from_str` (`src/designs/icon_variations/parse.rs:13-65`)
  recognizes none of those lucide names; the hints contain ASCII alnum so the
  emoji fallback also rejects them → `icon = None`. `ListItem` only mounts
  the icon slot when `self.icon.is_some()` (`src/list_item/mod.rs:2109-2116`),
  so **row text starts at the inner padding: 4 (outer) + 14 (inner) = 18px
  from the window edge**, 4px right of the 14px section-header origin.
  MEASURED. (`with_accent_bar` is a no-op — `src/list_item/mod.rs:1411`.)
- Row box: height 44 (`item_height` — `main_menu_theme.rs:745`; outer
  container `h(metrics.item_height).px(4).py(0)` at
  `src/list_item/mod.rs:2264-2273`), inner padding 14/4, radius 14, flex gap
  8.4 (single child ⇒ no visible gap). All generated
  (`--sk-main-menu-row-*`).
- Selected fill: `resolved_main_menu_row_fill` → text_primary at the 0x20
  component byte (`src/list_item/mod.rs:1723-1737`); generated
  `--sk-main-menu-row-selected-background: rgb(255 255 255 / 0.1254901961)`
  (`tokens.css:153`). Hover = `--sk-main-menu-row-hover-background`.
- Name: 14px / line 16, weight 450 → 500 selected, `--sk-text-name`
  (generated). Description: 12px / line 16, shown only when
  `selected || hover` (no filter active) at `src/list_item/mod.rs:1944-1950`;
  selected description color = text_primary at `alpha_muted`
  (`--sk-text-muted`). MEASURED.
- No shortcut/badge/source-hint accessories on any settings row
  (`settings.rs:573-642` sets none). MEASURED.

### Item census (default config, empty filter)
`get_settings_items` at `src/render_builtins/settings.rs:66-147`:
10 unconditional items + `Configure Snap Mode` (always pushed, 130-135)
= **11 rows**; `Reset Window Positions` appends only when
`crate::window_state::has_custom_positions()` (137-144). Sandbox HOME ⇒ 11
rows, count label `"11 settings"`. Real user home usually ⇒ 12. Order and
copy in the fixture are verbatim from 66-147. MEASURED.

### Footer
- GPUI slot: `main_window_footer_slot(render_simple_hint_strip(["↵ Open",
  "Esc Back"], None))` at `src/render_builtins/settings.rs:684-690`; when the
  native footer is active the slot renders a transparent spacer instead
  (`src/app_impl/ui_window.rs:1433-1449` →
  `render_native_main_window_footer_spacer`,
  `src/components/prompt_layout_shell.rs:771-779`, height = footer rail 32).
- Native footer truth: `SettingsView` has native surface `"settings"`
  (`src/main_sections/app_view_state.rs:1074`) and falls through to
  `standard_main_window_footer_buttons` (`src/app_impl/ui_window.rs:741-781`
  — no SettingsView special-case in
  `main_window_footer_buttons_for_current_view`, 882-1340): **Run ↵**
  (primary label defaults to "Run" at `src/app_impl/ui_window.rs:175`) +
  **Actions ⌘K** (SettingsView is a shared-actions host —
  `ActionsDialogHost::BuiltinList` at `src/app_impl/actions_dialog.rs:20-80`).
  The Agent ⌘↵ button is ScriptList-only (`ui_window.rs:768-779`). MEASURED
  from source; **GUESS until a live `activeFooter` probe confirms** (recorded
  non-blocking in known-divergence.json).
- Band geometry: 32pt rail inside 36pt host — generated
  `--sk-footer-rail-height` / `--sk-window-native-footer-host-height`
  (cross-cutting truth; `FooterMetricsTokens` at
  `src/designs/core/main_menu_theme.rs:688-715`).

---

## 2. Fixture strategy

Open path (deterministic, no list navigation):
`{"type":"triggerBuiltin","builtinId":"builtin/settings","protocolVersion":2}`
— `builtin/settings` entry at `src/builtins/mod.rs:1589-1607`
(`BuiltInFeature::Settings`), trigger id mapping at
`src/builtins/trigger_registry.rs:100`, dispatch →
`SurfaceOpenBuiltinAction::Settings` →
`open_builtin_filterable_view(AppView::SettingsView { filter: "", selected_index: 0 }, …)`
at `src/app_execute/builtin_execution.rs:4358-4373`. Direct-open semantics:
`opened_from_main_menu` is not set on this path, so Escape closes the window
rather than going back (fine for a capture; close with Escape afterwards).

Determinism: **`sandboxHome: true`** — settings rows are static except
`Reset Window Positions`, which keys off `window_state::has_custom_positions()`
(`settings.rs:137-144`); a fresh sandbox HOME has no custom positions ⇒
exactly 11 rows / "11 settings". Selection starts at index 0
("Theme Designer"), which also reveals its description. Note: a sandbox HOME
also changes context-zone pill content (cwd/model chips) versus the
real-home main-menu reference — treat the context zone as fixture-variable
when comparing across screens.

Proposed `scripts/agentic/design-reference-capture.ts` addition:

```ts
// DEFAULT_OUT:
settings: "design/mockups/screens/settings/reference/settings@2x.png",
// CAPTURE_TARGET:
settings: { type: "kind", kind: "main" },

// after the existing per-screen blocks:
if (screen === "settings") {
  await driver.request({
    type: "triggerBuiltin",
    builtinId: "builtin/settings",
    protocolVersion: 2,
  } as never, { timeoutMs: 5_000 }).catch(() => {});
  await driver.waitForSettle();
  await Bun.sleep(600);
}
```

plus `sandboxHome: true` in `Driver.launch` for this screen (the script
currently pins `sandboxHome: false` for all screens — make it per-screen).
Also grab the footer receipt in the same run: an `activeFooter` probe should
report `run='Run ↵'`, `actions='Actions ⌘K'` for surface `"settings"`.

---

## 3. Proposed `src/design_contract/mod.rs` section

```rust
// ── Settings hub (built-in list surface) ────────────────────────────
// Shares the InfoBarBase main-view chrome; only the hub-specific values
// are new tokens. Rows reuse mainMenu.* row/section tokens verbatim.
let default_spacing =
    crate::designs::get_tokens(crate::designs::DesignVariant::Default).spacing();

b.source_len(
    "settings.list.paddingY",
    "--sk-settings-list-padding-y",
    default_spacing.padding_xs,
    "DesignSpacing.padding_xs (render_settings content .py)",
);
b.add(
    "settings.countLabel.fontSize",
    TokenStage::Resolved,
    Some("--sk-settings-count-label-font-size"),
    TokenValue::Length { value: 14.0 },
    Some("gpui Styled::text_sm() rems(0.875) × 16px rem in render_builtin_main_input_count_label"),
    false,
    &[],
);
b.source_len(
    "settings.countLabel.insetRight",
    "--sk-settings-count-label-inset-right",
    def.search.text_inset_x,
    "MainMenuSearchTokens.text_inset_x (count label pr)",
);
b.add(
    "settings.countLabel.color",
    TokenStage::Resolved,
    Some("--sk-settings-count-label-color"),
    color_value(chrome.text_hint_rgba),
    Some("AppChromeColors.text_hint_rgba"),
    false,
    &["chrome.textHint"], // derived: same resolver as --sk-text-hint
);
b.add(
    "settings.section.defaultLabel",
    TokenStage::Source,
    None,
    TokenValue::Text { value: "Settings".into() },
    Some("render_settings persistent leading separator (empty filter)"),
    false,
    &[],
);
b.add(
    "settings.section.filteredLabel",
    TokenStage::Source,
    None,
    TokenValue::Text { value: "Results".into() },
    Some("render_settings persistent leading separator (active filter)"),
    false,
    &[],
);
```

Plus conflicts (section 5) appended to `b.conflicts`.

## 4. Resolver-extraction plan

- File: `src/render_builtins/settings.rs` (surface-local today; the values
  are inlined in `render_settings` at 657-664 and in
  `src/render_builtins/common.rs:67-80`).
- Proposed pure resolver:

```rust
pub(crate) struct SettingsHubMetrics {
    pub list_padding_y: f32,        // DesignSpacing.padding_xs
    pub count_label_font_size: f32, // 0.875 rem → px
    pub count_label_inset_right: f32, // def.search.text_inset_x
}

pub(crate) fn resolved_settings_hub_metrics(
    spacing: &crate::designs::DesignSpacing,
    def: crate::designs::MainMenuThemeDef,
    rem_px: f32,
) -> SettingsHubMetrics
```

  consumed by both `render_settings`/`render_builtin_main_input_count_label`
  and the exporter section above, mirroring the
  `resolved_confirm_prompt_metrics` pattern
  (`src/design_contract/mod.rs:1512-1516`). Since the count label is shared
  by ALL builtin browsers (`common.rs:67-80`), a follow-up may hoist the
  count-label fields to a `builtinInput.*` namespace; starting under
  `settings.*` keeps this slice narrow — flag at review.

## 5. Expected conflicts

1. `settingsRows.authoredIconHintsNeverResolve` — `get_settings_items`
   authors 11 lucide icon hints (`settings.rs:66-147`), but
   `IconKind::from_icon_hint` (`list_item/mod.rs:37-52`) only accepts
   `icon_name_from_str` names (`parse.rs:13-65`), which match none of them;
   rows paint iconless (`list_item/mod.rs:2109-2116`). Authored intent vs
   painted truth. (The settings EMPTY state does resolve an icon —
   `IconName::Settings`, `settings.rs:644-648` — deepening the mismatch.)
2. `settingsFooter.nativeRunVsGpuiOpenHint` — native footer advertises
   `Run ↵` (`ui_window.rs:746-752` + default label at 175) while the GPUI
   fallback hint strip on the same surface advertises `↵ Open`
   (`settings.rs:684-690`). Two live code paths, two verbs for one Enter.
3. `settingsHub.duplicateCommandCopy` — hub items restate builtin-registry
   twins with different copy (e.g. hub "Clear Suggested Items / Reset
   Suggested and Recently Used launcher history" at `settings.rs:86-91` vs
   builtin "Clear Suggested / Clear all items from Suggested / Recently
   Used" at `src/builtins/mod.rs:1566-1580`; same for Dictation Setup and
   Check Permissions descriptions rebuilt inline at `settings.rs:207-305`).
4. `settingsCount.configDependentRowCount` — `has_custom_positions()`
   (`settings.rs:137-144`) makes row count and the count label
   config-dependent (11 vs 12); the checked-in fixture pins the sandbox
   default. Severity: fixture, not paint.

## 6. Tokens to generate

New (6): `settings.list.paddingY` (4), `settings.countLabel.fontSize` (14),
`settings.countLabel.insetRight` (16), `settings.countLabel.color`
(= `chrome.text_hint_rgba`), `settings.section.defaultLabel` ("Settings"),
`settings.section.filteredLabel` ("Results").

Until they are exported, `screen.css` declares literal-free **placeholder
alias hooks** (pure `var()` indirection, lint-clean):
`--sk-settings-list-padding-y → var(--sk-main-menu-header-padding-y)`,
`--sk-settings-count-label-font-size → var(--sk-main-menu-name-font-size)`,
`--sk-settings-count-label-color → var(--sk-text-hint)`,
`--sk-settings-count-label-inset-right → var(--sk-main-menu-search-text-inset-x)`.
Each aliases a token whose resolved value equals the settings value today;
swap to the generated names when the exporter section lands.

Everything else reuses generated tokens: `--sk-window-*`,
`--sk-main-menu-{header,context,search,row,section,name,description}-*`,
`--sk-caret-*`, `--sk-text-{name,muted,hint,placeholder}`,
`--sk-footer-*`, `--sk-vibrancy-*`.

## 7. Open unknowns

- Native footer labels for surface `"settings"` need a live `activeFooter`
  receipt (source says Run ↵ / Actions ⌘K; no probe run this slice — probes
  are out of scope for this task).
- Whether the vendor overlay scrollbar paints in a fresh capture (11 rows
  overflow the ~354px viewport by ~2 rows; ScrollbarShow behavior for the
  tracked column is `ScrollableElement::vertical_scrollbar`, not the
  Always-shown uniform-list variant).
- Context-zone pill content under sandbox HOME (cwd/model chips) — fixture
  copies the main-menu reference pills; confirm at capture time.
- Count-label baseline: `.text_sm()` inside the 26px shell centers on the
  flex axis; exact baseline vs the 20px query text needs the capture.
