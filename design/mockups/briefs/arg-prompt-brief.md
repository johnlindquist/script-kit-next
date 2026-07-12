# Arg Prompt — design-contract mockup brief

Screen slice for the script-driven arg prompt (`AppView::ArgPrompt`: input +
filtered choice list). Mockup at `design/mockups/screens/arg-prompt/`.
Every metric/color claim below is tagged **MEASURED-from-source** (verified at
the cited file:line on 2026-07-11) or **GUESS** (needs a capture/probe to
confirm).

Render entry: `src/main_sections/render_impl.rs:484-490` dispatches
`AppView::ArgPrompt` to `render_arg_prompt`
(`src/render_prompts/arg/render.rs:75-312`), which composes
`render_minimal_list_prompt_shell_with_footer`
(`src/components/prompt_layout_shell.rs:404-447`).

---

## 1. Surface anatomy

### Window

| Part | Value | Source | Tag |
|---|---|---|---|
| Width | 750 (unchanged: `width_for_view` returns `None` for arg, window keeps `MAIN_WINDOW_WIDTH`) | `src/window_resize/mod.rs:687-696` | MEASURED-from-source |
| Height (6 choices) | **319px** = `ARG_HEADER_HEIGHT` 68 + 6×`LIST_ITEM_HEIGHT` 40 + `ARG_LIST_PADDING_Y` 8 + `ARG_DIVIDER_HEIGHT` 1 + `WINDOW_BORDER_Y` 2, clamped to [68, 500] | `src/window_resize/mod.rs:623-629`; constants `:521-536` (`ARG_HEADER_HEIGHT` = 8×2 + 22 + 30 = 68 at `:532`) | MEASURED-from-source |
| Radius | 22 (`LIQUID_GLASS_WINDOW_RADIUS_PX`) — existing `--sk-window-radius` | `src/ui/chrome/tokens.rs:15` | MEASURED-from-source |
| Background | vibrancy: `get_vibrancy_background` returns `None` when vibrancy enabled (stock script-kit-dark), so the Root tint paints — same `--sk-window-vibrancy-tint` as main menu | `src/ui_foundation/mod.rs:200-208`; `src/render_prompts/arg/render.rs:284` | MEASURED-from-source |
| Shader effect | background effect layer paints at the window root behind ALL views incl. ArgPrompt | `src/main_sections/render_impl.rs:833, 1107-1109` | MEASURED-from-source |
| Context zone | **absent** — `ArgPrompt` is not in `uses_shared_main_view_header()` | `src/main_sections/app_view_state.rs:984-1017` | MEASURED-from-source |

### Header / input

| Part | Value | Source | Tag |
|---|---|---|---|
| Header padding | px 16 (`HEADER_PADDING_X`), py 8 (`HEADER_PADDING_Y`), min-h 28 (`HEADER_BUTTON_HEIGHT`) → painted band 8+22+8 = **38px** | scaffold `src/components/prompt_layout_shell.rs:417-421`; `src/window_resize/mod.rs:100-102`; `src/panel.rs:20` | MEASURED-from-source |
| Input host height | **22px** = `CURSOR_HEIGHT_LG` 18 + 2×`CURSOR_MARGIN_Y` 2 | `src/render_prompts/arg/render.rs:220`; `src/panel.rs:81,87` | MEASURED-from-source |
| Input component | gpui-component `Input::new(&self.gpui_input_state)`, `px(0) py(0)`, `appearance(false)`, `bordered(false)` — NOT the repo TextInput and NOT `render_arg_input_text` (that helper is only used by `micro.rs:205`) | `src/render_prompts/arg/render.rs:234-244` | MEASURED-from-source |
| Input font size | **20px** = `font_size_xl` = `ui_size` 16 × 1.25 (Default design variant reads theme fonts) | `src/theme/color_resolver.rs:256-263`; `src/theme/types.rs:865-868` | MEASURED-from-source |
| Input font family | `.AppleSystemUIFont` (Default `DesignTypography.font_family`) — existing `--sk-font-ui` | `src/designs/core/tests.rs:245`; shell sets it at `src/render_prompts/arg/render.rs:291` | MEASURED-from-source |
| Input text / placeholder color | text = theme `text.primary`; placeholder = gpui-component `muted_foreground` = text.primary @ `opacity.text_placeholder` (0.40 ⇒ same byte as `--sk-text-placeholder`) | `src/theme/gpui_integration.rs:251-257` | MEASURED-from-source |
| Caret | color = `cx.theme().caret` = theme `text.primary` (white, stock — no focus-aware cursor override); width **2px**; height **0.85 × line_height** (`Size::Size(px(20))` falls into the `_` arm) | color `src/theme/gpui_integration.rs:302-310`, paint `vendor/gpui-component/crates/ui/src/input/element.rs:1677-1684`; width `vendor/gpui-component/crates/ui/src/input/blink_cursor.rs:32`; height `element.rs:283-287` | MEASURED-from-source |
| Caret pixel height | exact `line_height` the vendor Input resolves for a 22px host with 20px font — **unmeasured**; mockup falls back to 18px | — | GUESS (measure from `arg` reference capture) |
| Divider | 1px (`HEADER_DIVIDER_HEIGHT`), mx 16 (`HEADER_DIVIDER_MARGIN`), color `chrome.divider_rgba` (= `--sk-chrome-divider`) | `src/components/prompt_layout_shell.rs:1304-1311`; `src/panel.rs:23,26` | MEASURED-from-source |

### Choice rows

Rows are the **shared `ListItem` with NO `metrics_override`**
(`src/render_prompts/arg/render.rs:199-206`), so metrics resolve
`ListItemMetricsOverride::from_main_menu_theme(MainMenuThemeVariant::default())`
(`src/list_item/mod.rs:1656-1658`, default set at `:1396`), and the default
variant is `InfoBarBase` (`src/designs/core/main_menu_theme.rs:849,896`).
**Arg rows therefore paint the exact same resolved values as the existing
`--sk-main-menu-row-*` / `--sk-main-menu-name-*` tokens** (44px slot, outer
4/0, inner 14/4, radius 14, selected fill text-primary @0x20, hover @0x12,
name 14px w450 lh16, selected w500, desc 12px lh16). Fill bytes come from the
same `resolved_main_menu_row_fill` the exporter already consumes
(`src/list_item/mod.rs:296-348`, applied at `:1725-1735`; container geometry
`:2265-2274`; inner padding/radius `:2089-2100`). — MEASURED-from-source.

Arg-specific row facts:

| Fact | Source | Tag |
|---|---|---|
| No icon slot mounted (`Choice` has no icon field; icon only mounted when present) → name origin x = outer 4 + inner 14 = **18px** | `src/protocol/types/primitives.rs:116-133`; `src/list_item/mod.rs:2113-2115` | MEASURED-from-source |
| Description paints only when `selected \|\| hovered` (progressive disclosure); selected desc color = text.primary @ `alpha_muted` (= `--sk-text-muted`) | `src/list_item/mod.rs:1945-1962` | MEASURED-from-source |
| `.with_accent_bar(true)` at the call site is a **no-op** (accent bar removed) | `src/render_prompts/arg/render.rs:204`; `src/list_item/mod.rs:1408-1413` | MEASURED-from-source |
| No section headers, no painted list padding — `uniform_list` starts directly under the divider | `src/render_prompts/arg/render.rs:186-217, 249-259` | MEASURED-from-source |
| Empty-filter state = `render_shared_empty_state(InfoEmptySurface::ArgChoices)` with `padding_md`/`padding_sm` (not in fixture) | `src/render_prompts/arg/render.rs:173-185` | MEASURED-from-source |

### Footer

| Fact | Source | Tag |
|---|---|---|
| GPUI paints only a transparent spacer when the native footer is active: height = `current_main_menu_footer_height()` = `FooterMetricsTokens.height_px` = **32px** (= existing `--sk-footer-rail-height`) | `src/render_prompts/arg/render.rs:276-280`; `src/app_impl/ui_window.rs:1433-1450`; `src/components/prompt_layout_shell.rs:771-779`; `src/components/footer_chrome.rs:178-180` | MEASURED-from-source |
| Native overlay buttons for arg = `standard_main_window_footer_buttons`: **Run ↵** (label from `main_window_primary_action_label` `_ =>` arm) + **Actions ⌘K** (ArgPrompt is a shared-actions host); NO Agent chip (ScriptList-only) | `src/app_impl/ui_window.rs:741-781, 143-176`; host `src/app_impl/actions_dialog.rs:52` | MEASURED-from-source |
| GPUI hint-strip fallback (native footer inactive) = `universal_prompt_hints()`: "↵ Run", "⌘K Actions", Agent ⌘↵ hint | `src/components/prompt_layout_shell.rs:712-735` | MEASURED-from-source |
| Native footer surface id = `"arg_prompt"` | `src/main_sections/app_view_state.rs:1051` | MEASURED-from-source |
| Exact overlay labels at capture time | — | GUESS (verify via `activeFooter` probe; captures show an empty band) |

---

## 2. Fixture strategy

The stdin protocol accepts SDK prompt messages as a fallback after
`ExternalCommand` parsing: `StdinCommand::Protocol`
(`src/stdin_commands/mod.rs:947-950`, fallback parse `:1060-1075`) routes via
`handle_stdin_protocol_message` (`src/main_entry/runtime_stdin.rs:1467-1470`)
→ `prompt_message_from_protocol_message`
(`src/prompt_handler/message_route.rs:20-34`, `Message::Arg` wire shape
`src/protocol/message/variants/prompts_media.rs:48-57`) →
`PromptMessage::ShowArg` (`src/prompt_handler/mod.rs:1962-1990`). This is the
same mechanism `scripts/agentic/fields-prompt-parity.ts:65-71` already uses
for `{"type":"fields",…}` — no live script needed.

**Exact command JSON** (deterministic, ASCII-only):

```json
{"type":"arg","id":"design-arg-fixture","placeholder":"Pick a fruit","choices":[
  {"name":"Apple","value":"apple","description":"Crisp and sweet — the default pick"},
  {"name":"Banana","value":"banana"},
  {"name":"Cherry","value":"cherry"},
  {"name":"Dragonfruit","value":"dragonfruit"},
  {"name":"Elderberry","value":"elderberry"},
  {"name":"Fig","value":"fig"}
],"actions":[{"name":"Inspect Fruit","description":"Design fixture action","value":"inspect","hasAction":false}]}
```

(`Choice` fields: `src/protocol/types/primitives.rs:116-133`. The `actions`
entry keeps the footer's ⌘K chip honest; drop it to test the no-actions
variant.)

**Proposed `scripts/agentic/design-reference-capture.ts --screen arg` block**
(pattern: existing `confirm` block at `:62-73`):

```ts
// DEFAULT_OUT:
arg: "design/mockups/screens/arg-prompt/reference/arg-prompt@2x.png",
// CAPTURE_TARGET:
arg: { type: "kind", kind: "main" },

if (screen === "arg") {
  await driver.request({
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
    actions: [{ name: "Inspect Fruit", description: "Design fixture action", value: "inspect", hasAction: false }],
  } as never, { timeoutMs: 5_000 }).catch(() => {});
  await driver.waitForSettle();
  await Bun.sleep(600);
  // Receipt: getState should report promptType "arg", promptId "design-arg-fixture"
  // (state surface: src/prompt_handler/mod.rs:3040-3062). Also record activeFooter
  // for the native button labels (Run ↵ / Actions ⌘K).
}
```

Post-capture: Escape once returns to the script list (DismissPolicy), leave
`windowVisible:false` per repo convention.

---

## 3. Proposed `src/design_contract/mod.rs` section

Add after the actions-dialog section, reusing `def`/`metrics` already in
scope. Arg rows intentionally emit **no new row tokens** — they alias the
main-menu row tokens by construction (same `from_main_menu_def(def)` path).

```rust
// ── Arg prompt (script-driven choices/input, AppView::ArgPrompt) ─────
// Rows are the shared ListItem with NO metrics override — they resolve the
// same InfoBarBase def as the main menu, so --sk-main-menu-row-* /
// --sk-main-menu-name-* are the row contract. Only shell/input/footer-slot
// values are arg-specific.
let arg = crate::render_prompts::resolved_arg_prompt_chrome(&theme); // see §4

b.source_len(
    "argPrompt.header.paddingX",
    "--sk-arg-header-padding-x",
    crate::window_resize::main_layout::HEADER_PADDING_X, // 16
    "crate::window_resize::main_layout::HEADER_PADDING_X",
);
b.add(
    "resolved.argPrompt.header.height",
    TokenStage::Resolved,
    Some("--sk-arg-header-height"),
    TokenValue::Length { value: arg.header_height as f64 }, // 8*2 + 22 = 38
    None,
    false,
    &["argPrompt.input.height", "window.headerPaddingY"],
);
b.add(
    "resolved.argPrompt.input.height",
    TokenStage::Resolved,
    Some("--sk-arg-input-height"),
    TokenValue::Length {
        value: (crate::panel::CURSOR_HEIGHT_LG + 2.0 * crate::panel::CURSOR_MARGIN_Y) as f64, // 22
    },
    None,
    false,
    &["mainMenu.caret.height"],
);
b.add(
    "resolved.argPrompt.input.fontSize",
    TokenStage::Resolved,
    Some("--sk-arg-input-font-size"),
    TokenValue::Length {
        value: crate::theme::TypographyResolver::new(&theme, crate::designs::DesignVariant::Default)
            .font_size_xl() as f64, // 20
    },
    None,
    false,
    &["theme.fonts.uiSize"],
);
b.source_len(
    "argPrompt.input.caretWidth",
    "--sk-arg-input-caret-width",
    2.0, // vendor/gpui-component blink_cursor::CURSOR_WIDTH — re-export, do not inline
    "gpui_component::input::blink_cursor::CURSOR_WIDTH",
);
b.add(
    "resolved.argPrompt.input.caretHeight",
    TokenStage::Resolved,
    Some("--sk-arg-input-caret-height"),
    TokenValue::Length { value: arg.input_caret_height as f64 }, // 0.85 × input line height
    None,
    false,
    &["argPrompt.input.fontSize"],
);
b.source_len(
    "argPrompt.divider.marginX",
    "--sk-arg-divider-margin-x",
    crate::panel::HEADER_DIVIDER_MARGIN, // 16
    "crate::panel::HEADER_DIVIDER_MARGIN",
);
b.add(
    "resolved.argPrompt.footer.spacerHeight",
    TokenStage::Resolved,
    Some("--sk-arg-footer-spacer-height"),
    TokenValue::Length {
        value: crate::components::footer_chrome::current_main_menu_footer_height() as f64, // 32
    },
    None,
    false,
    &["footer.railHeight"],
);
// Fixture height: what height_for_view(ArgPromptWithChoices, 6) actually
// resolves — reuses the production model, never re-derived by hand.
b.add(
    "resolved.argPrompt.shell.fixtureHeight",
    TokenStage::Resolved,
    Some("--sk-arg-window-height"),
    TokenValue::Length {
        value: f32::from(crate::window_resize::height_for_view(
            crate::window_resize::ViewType::ArgPromptWithChoices,
            6,
        )) as f64, // 319 with default layout config
    },
    None,
    false,
    &["argPrompt.header.height", "window.width"],
);
```

Plus the conflicts in §5 recorded via `b.conflict(...)`.

## 4. Resolver-extraction plan

- **File:** `src/render_prompts/arg/helpers.rs` (or a new
  `src/render_prompts/arg/chrome.rs`) — alongside the existing pure helpers
  (`arg_input_visible_chars_for_width` etc.).
- **Proposed signature:**

  ```rust
  pub struct ResolvedArgPromptChrome {
      pub header_height: f32,       // HEADER_PADDING_Y*2 + input height = 38
      pub input_height: f32,        // CURSOR_HEIGHT_LG + 2*CURSOR_MARGIN_Y = 22
      pub input_font_size: f32,     // TypographyResolver::font_size_xl() = 20
      pub input_caret_height: f32,  // 0.85 * resolved input line height
      pub divider_margin_x: f32,    // panel::HEADER_DIVIDER_MARGIN = 16
      pub footer_spacer_height: f32,// current_main_menu_footer_height() = 32
  }

  pub fn resolved_arg_prompt_chrome(theme: &crate::theme::Theme) -> ResolvedArgPromptChrome;
  ```

- **Refactor:** `render_arg_prompt` currently inlines
  `CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)` (`render.rs:220`) and
  `typography_resolver.font_size_xl()` (`render.rs:240`); move both reads into
  the resolver so renderer and exporter share one function (the
  actions-dialog `resolved_actions_dialog_*_chrome` pattern,
  `src/design_contract/mod.rs:1105-1108`).
- Caret height needs the vendor line-height rule surfaced: either re-export
  `gpui_component` `CURSOR_WIDTH`/height factor or pin them with a vendored
  source-audit-free constant + test against the vendor crate.

## 5. Expected conflicts (record via `b.conflict`, never smooth over)

1. **`argPrompt.rowHeight.modelVsPaint`** — resize model reserves
   `crate::list_item::LIST_ITEM_HEIGHT` = 40/row
   (`src/window_resize/mod.rs:13,626`; `src/list_item/mod.rs:98`) but rows
   paint `InfoBarBase.item_height` = 44
   (`src/list_item/mod.rs:1656-1658,2267`). With 6 choices the window model
   gives 319 while content wants 335 → the 6th row clips to ~28px. Severity:
   high (visible truncation on every arg prompt with a full list).
2. **`argPrompt.listPadding.deadModelField`** — `ARG_LIST_PADDING_Y` = 8
   (`src/window_resize/mod.rs:521`) is counted in the height model but the
   arg renderer paints no list padding (`render.rs:249-259`). Dead field.
3. **`argPrompt.footerHeight.modelVsPaint`** — model `FOOTER_HEIGHT` = 30
   (`src/window_resize/mod.rs:527`, inside `ARG_HEADER_HEIGHT`) vs painted
   native spacer 32 (`footer_chrome.rs:178-180`) vs main-menu native host 36
   (`main_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT`,
   `src/window_resize/mod.rs:110`). Three values for one band.
4. **`argPrompt.caret.repoVsVendor`** — `panel::CURSOR_WIDTH` 2.5 /
   `CURSOR_HEIGHT_LG` 18 (`src/panel.rs:69,81`) size the input *host*, but the
   painted caret is vendor gpui-component: 2px × 0.85·line_height
   (`blink_cursor.rs:32`, `element.rs:283-287`). The dead
   `render_arg_input_text` config (`render.rs:3-46`, only `micro.rs` uses it)
   still advertises the repo constants.
5. **`argPrompt.hints.gpuiFallbackVsNative`** — GPUI fallback strip says
   "↵ Run / ⌘K Actions / ⌘↵ Agent" (`universal_prompt_hints`,
   `prompt_layout_shell.rs:712-735`) while the native footer shows only
   Run + Actions for arg (`ui_window.rs:768-779` gates Agent to ScriptList).
   The ⌘↵ Agent hint would lie on this surface if the fallback ever paints.

## 6. Tokens to generate (8 new; rows reuse existing tokens)

| CSS var | Stage | Expected value | Resolver |
|---|---|---|---|
| `--sk-arg-window-height` | resolved | 319px (6-choice fixture) | `height_for_view(ArgPromptWithChoices, 6)` |
| `--sk-arg-header-height` | resolved | 38px | `HEADER_PADDING_Y*2 + input height` |
| `--sk-arg-header-padding-x` | source | 16px | `main_layout::HEADER_PADDING_X` |
| `--sk-arg-input-height` | resolved | 22px | `CURSOR_HEIGHT_LG + 2*CURSOR_MARGIN_Y` |
| `--sk-arg-input-font-size` | resolved | 20px | `TypographyResolver::font_size_xl()` |
| `--sk-arg-input-caret-width` | source | 2px | vendor `blink_cursor::CURSOR_WIDTH` |
| `--sk-arg-input-caret-height` | resolved | 0.85 × input line height (**measure**) | vendor `element.rs` cursor rule |
| `--sk-arg-divider-margin-x` | source | 16px | `panel::HEADER_DIVIDER_MARGIN` |
| `--sk-arg-footer-spacer-height` | resolved | 32px | `current_main_menu_footer_height()` |

(9 rows listed; `--sk-arg-header-padding-x` and `--sk-arg-divider-margin-x`
may collapse into one shared 16px inset token if design prefers.)

Until generated, `screen.css` references each var with a **pure-var fallback**
to an equal-valued existing token (documented inline); only
`--sk-arg-window-height`'s fallback (480) differs from truth (319) — flagged
in `known-divergence.json` and `compare.html`.

## 7. Open unknowns

- Exact vendor Input line height for a 22px host with 20px font (caret pixel
  height + placeholder baseline) — measure from the first `--screen arg`
  reference capture.
- Whether the stock capture profile shifts the white caret/selection hue the
  way the actions-dialog capture shifted amber — check at capture time.
- `activeFooter` receipt for the arg surface (`"arg_prompt"`) to lock the
  native labels Run/Actions.
