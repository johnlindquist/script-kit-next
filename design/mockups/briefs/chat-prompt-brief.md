# Chat Prompt — design-contract brief (DRAFT)

Screen: `design/mockups/screens/chat-prompt/` — the **script-driven SDK chat
prompt** (`src/prompts/chat/**`, protocol `{"type":"chat"}`), rendered inside
the main window. NOT Agent Chat (separate screen, separate owner).

Profile assumed throughout: stock `script-kit-dark`
(src/theme/presets.rs:946-975), vibrancy on, full main-window mode
(`MainWindowMode::Full`), scale-independent logical px (GPUI points).

---

## 1. Surface anatomy (all claims tagged)

### Window shell

| Metric | Value | Source | Tag |
|---|---|---|---|
| Width | 750 | src/window_resize/mod.rs:291 `MAIN_WINDOW_WIDTH` | MEASURED-from-source |
| Height (Full mode) | 500 | ChatPrompt → `compact_ai_view_type_for_mode(Full)` = `ViewType::DivPrompt` (src/app_impl/ui_window.rs:1915-1917) → `standard_height` (src/window_resize/mod.rs:593) = `DEFAULT_LAYOUT_STANDARD_HEIGHT` 500.0 (src/config/defaults.rs:19) | MEASURED-from-source |
| Height (Mini mode) | 480 | `ViewType::MiniAiChat` → `height_for_main_window` = `MAIN_WINDOW_MAX_HEIGHT` 480.0 (src/window_resize/mod.rs:28,222-224,594-611) | MEASURED-from-source |
| Shell | `render_simple_prompt_shell(radius 0, vibrancy bg None)` — chat fills the window; vibrancy tint comes from the window root | src/render_prompts/other.rs:473-508 (`other_prompt_shell_radius` = 0.0 at :14-16); `get_vibrancy_background` returns `None` under vibrancy (src/ui_foundation/mod.rs) | MEASURED-from-source |
| Window radius / tint / material | shared window tokens (`--sk-window-radius` 22, `--sk-window-vibrancy-tint`) | design/mockups/generated/tokens.css (bundleHash 9ebb78b8…) | MEASURED-from-source |

Shared layout constants (src/prompts/chat/mod.rs:52-57):
`CHAT_LAYOUT_PADDING_X = 12.0`, `CHAT_LAYOUT_SECTION_PADDING_Y = 8.0`,
`CHAT_LAYOUT_MESSAGES_PADDING_Y = 8.0`, `CHAT_LAYOUT_CARD_PADDING_X = 12.0`,
`CHAT_LAYOUT_CARD_PADDING_Y = 10.0`, `CHAT_LAYOUT_BORDER_ALPHA = 0x40`.
All MEASURED-from-source.

### Header (`render_header`, src/prompts/chat/render_input.rs:82-111)

- Row: flex, items-center, gap 8, px 12 (`CHAT_LAYOUT_PADDING_X`), py 8
  (`CHAT_LAYOUT_SECTION_PADDING_Y`), border-bottom 1px
  `(prompt_colors.quote_border << 8) | 0x40` = ui.border `0x343434` @ 0x40 →
  `rgb(52 52 52 / 0.2509803922)`. MEASURED-from-source (render_input.rs:86-95).
- Back arrow "←": `text_sm` (14px; gpui `rems(0.875)`,
  vendor/gpui/src/styled.rs:513-517), color `prompt_colors.text_secondary` =
  `colors.text.secondary` (src/theme/helpers.rs:135) = `0xffffff` stock
  (presets.rs:956). MEASURED-from-source.
- Title: default `"Chat"` (src/prompts/chat/prompt.rs:129), `text_sm`,
  `FontWeight::MEDIUM` (500), `prompt_colors.text_primary` = white.
  MEASURED-from-source (render_input.rs:103-110).
- Line height: no explicit line-height → GPUI phi default,
  round(14 × 1.618034) = **23px** (vendor/gpui/src/style.rs
  line_height_in_pixels; cross-checked this session). MEASURED-from-source.
- Header band total: 8 + 23 + 8 + 1 = **40px**. Derived; GUESS until
  pixel-verified against the reference capture.

### Input area — composer at TOP (src/prompts/chat/render_core.rs:324-362)

- Wrapper: w-full, px 12, py 8, flex-col, gap 8 (full mode only,
  render_core.rs:346), border-bottom 1px `(ui.border << 8) | 0x40`
  (render_core.rs:351-354). MEASURED-from-source.
- Field (`#chat-input-field`, render_input.rs:70-78): w-full, **min-height
  28**, flex row items-center, **no horizontal padding of its own** (text
  origin = the wrapper's 12pt inset), background:
  - idle: `AppChromeColors.input_surface_rgba` = `background.search_box`
    (`0x2a2a2a`, presets.rs:951) @ `opacity.search_box` 0.50
    (src/theme/types.rs:257) → `rgb(42 42 42 / 0.5019607843)`
    (src/theme/chrome.rs:144-147).
  - focused: `input_active_rgba` = search_box @ `opacity.input_active` 0.50
    (types.rs:266; chrome.rs:191-194) → **identical to idle in stock dark**.
  All MEASURED-from-source.
- Text: 14px full mode / 16px mini (render_input.rs:26), color
  `theme.colors.text.primary` (white). MEASURED-from-source.
- Caret: shared text-input painter, `cursor_width` = `panel::CURSOR_WIDTH`
  2.5, `cursor_height` = `panel::CURSOR_HEIGHT_LG` 18.0 (src/panel.rs:69,81
  via `TextInputRenderConfig::default_for_prompt`,
  src/components/text_input/render.rs:189-199), color
  `accent.selected` `0xfbbf24` (render_input.rs:41). Matches generated
  `--sk-caret-width`/`--sk-caret-height`/`--sk-color-accent`.
  MEASURED-from-source.
- Placeholder: `"Ask follow-up..."` default (render_input.rs:52-58), color
  `rgb(theme_colors.text.muted)` — **full alpha**, and `text.muted` is
  `0xffffff` stock (presets.rs:958) → pure white. MEASURED-from-source; see
  Conflict C1.
- Input band total: 8 + 28 + 8 + 1 = **45px**. Derived; GUESS until
  pixel-verified.

### Messages area (render_core.rs:364-475)

- Virtualized `list()` with `ListAlignment::Bottom` (prompt.rs:137) — turns
  anchor to the bottom edge. px 12, py 8 (render_core.rs:387-388).
  MEASURED-from-source.
- Each turn wrapper: `pb 8` (render_core.rs:377). MEASURED-from-source.
- "Jump to latest" pill (only after user scrolls up): absolute bottom 12,
  px 10 / py 5, fully rounded, bg `(ui.border << 8) | 0xCC`, `text_xs`
  (render_core.rs:433-460). Not in the idle fixture. MEASURED-from-source.
- Empty state renders conversation starters (render_core.rs:463-474) — not
  exercised by this fixture.

### Turn card (`render_turn`, src/prompts/chat/render_turns.rs:10-208)

One container per user+assistant exchange (full-width card, not bubbles):

- Card: w-full, px 12 (`CARD_PADDING_X`), py 10 (`CARD_PADDING_Y`), bg
  `hover_overlay_bg(theme, 0x15)` = `text.primary` @ 0x15 →
  `rgb(255 255 255 / 0.0823529412)` dark / `0x08` light
  (render_turns.rs:21-26; src/theme/helpers.rs:190-193), radius 8, flex row
  items-start, gap 8 (render_turns.rs:196-207). MEASURED-from-source.
- Content column: flex-col, gap 6 (render_turns.rs:32). MEASURED-from-source.
- User prompt line: `text_sm` (14), `FontWeight::SEMIBOLD` (600), color
  `text.secondary` (white stock) (render_turns.rs:36-45). MEASURED-from-source.
- Optional user image thumb: 64×64, rounded-sm (render_turns.rs:49-57). Not
  in fixture. MEASURED-from-source.
- Copy button: 24×24, radius 4, base opacity 0.7 (hover → 1.0 + overlay
  0x28), svg icon 16×16, color `text.secondary`
  (render_turns.rs:174-193). MEASURED-from-source.
- Streaming affordances (not in idle fixture): empty-stream "Thinking..."
  `text_xs` sine pulse opacity 0.35–0.65 @ 1200ms (render_turns.rs:127-137);
  mid-stream 7px accent dot, radius 999, pulse 0.65–1.0 @ 1200ms
  (render_turns.rs:150-165). MEASURED-from-source.
- Error state: `text_sm` in `ui.error` (`0xef4444`), retry chip px 8 / py 4 /
  radius 4 bg `error|0x40` (render_turns.rs:59-117). Not in fixture.

### Markdown rendering (src/prompts/markdown/**)

- Root: flex-col, **gap 6**, w-full (api.rs:50-57). MEASURED-from-source.
- Paragraph: `text_sm` 14px, color `prompt_colors.text_primary` (white); phi
  line height 23px (inline_render.rs:97-102 fast path;
  render_blocks.rs:13-27). MEASURED-from-source.
- Headings: h1 `text_lg`(18)/BOLD, h2 `text_base`(16)/SEMIBOLD, h3
  `text_sm`/SEMIBOLD (render_blocks.rs:40-45). Not in fixture.
- Code block (code_table.rs:25-168):
  - container: mt 4 / mb 4, radius 6, bg `(code_bg << 8) | 0xE0` where
    `code_bg` = `background.search_box` `0x2a2a2a` (helpers.rs:138) →
    `rgb(42 42 42 / 0.8784313725)`; border 1px `(quote_border << 8) | 0x40`
    → `rgb(52 52 52 / 0.2509803922)` (code_table.rs:39-51).
  - header strip: px 10 / py 4, border-bottom `|0x30` →
    `rgb(52 52 52 / 0.1882352941)`; label `"lang · N lines"` `text_xs` (12px)
    in `text_tertiary` (white stock) (code_table.rs:56-79). Copy control is
    opacity-0 until group hover (code_table.rs:95-97) — omitted from fixture.
  - body: px 10 / py 8, flex-col gap 2, horizontal scroll; each line
    `FONT_MONO` = "JetBrains Mono" (src/list_item/mod.rs), `text_sm` 14px,
    min-height 16 (effective line box = phi 23), syntect span colors
    (code_table.rs:128-165). All MEASURED-from-source; syntax colors are
    known-divergence (untokenized).
- Inline code chip: px 4 / py 1, radius 3, bg `(code_bg << 8) | 0x80` →
  `rgb(42 42 42 / 0.5019607843)`, mono, `text_primary`
  (inline_render.rs:11-20). MEASURED-from-source.
- Links: accent text + 1px underline border `accent|0x40`
  (inline_render.rs:30-46). Not in fixture.

### Footer

- Chat routes its footer through
  `render_main_window_footer_slot_for_prompt_surface("chat_prompt", …)`
  (render_core.rs:65-70); surface id registered at
  src/main_sections/app_view_state.rs:1058. When the main window is visible
  and the native surface matches, GPUI renders only a spacer
  (`render_native_main_window_footer_spacer`,
  src/components/prompt_layout_shell.rs:771-779) at
  `NATIVE_MAIN_WINDOW_FOOTER_HEIGHT` = `HINT_STRIP_HEIGHT` = **36**
  (src/window_resize/mod.rs:37,108-110). MEASURED-from-source. Captures show
  an EMPTY 36pt band (cross-cutting truth).
- Native buttons (source-derived, RECEIPT-PENDING): ChatPrompt hits the
  default branch → `standard_main_window_footer_buttons`
  (src/app_impl/ui_window.rs:741-781): primary
  `main_window_primary_action_label()` falls to `"Run"` for ChatPrompt
  (ui_window.rs:143-175, `_ => "Run"`), plus `⌘K Actions`; the `⌘↵ Agent`
  button is ScriptList-only (ui_window.rs:768-779). GUESS until an
  `activeFooter` probe confirms — see Conflict C6.
- GPUI fallback footer (window hidden / surface mismatch): `HintStrip` 36px,
  px 14 / py 8 (src/window_resize/mod.rs:100-114), leading status text
  `text_xs` at `text.primary` @ `HINT_TEXT_OPACITY` 0.80
  (src/components/prompt_layout_shell.rs:692-704), status =
  `"<model> · Shift+Enter newline"` (+ "Streaming"/"Script mode" prefixes)
  (render_core.rs:5-27); default model = `"claude-haiku-4-5-20250514"`
  (prompt.rs:118-119; src/prompts/chat/types.rs:31-43). Hint labels 12.5px,
  keycaps px 6 / py 1 / radius 5 / bg white@0.12
  (src/components/hint_strip.rs:47-51,649-658). MEASURED-from-source; not in
  the reference (native footer wins while visible).

### Mini mode (not the mockup fixture)

`mini_mode` drops the header, uses `HEADER_PADDING_X/Y` 16/8 for the input
wrapper, 16px input text, no field bg, divider at `DIVIDER_OPACITY` 0.30, and
the plain mini hint strip (render_core.rs:29-37,329-354;
render_input.rs:26,62-68). Documented for the contract; full mode is the
canonical capture.

---

## 2. Fixture strategy + proposed capture block

Deterministic open path: the stdin protocol accepts full protocol messages
(`StdinCommand::Protocol` → `handle_stdin_protocol_message` →
`prompt_message_from_protocol_message` maps `Message::Chat` →
`PromptMessage::ShowChat`; src/stdin_commands/mod.rs:947-950,
src/main_entry/app_run_setup.rs:3637-3640,
src/prompt_handler/message_route.rs:137-159,
src/prompt_handler/mod.rs:8456-8645). No script file needed — same pattern as
the confirm fixture.

Exact command JSON:

```json
{
  "type": "chat",
  "id": "design-chat-fixture",
  "placeholder": "Ask follow-up...",
  "saveHistory": false,
  "useBuiltinAi": false,
  "messages": [
    { "role": "user", "content": "How do I read the clipboard in a script?" },
    { "role": "assistant", "content": "Use the SDK clipboard helper, then show it in a prompt:\n\n```ts\nconst text = await clipboard.readText();\nawait div(md(text));\n```\n\nCall `clipboard.writeText(...)` to write back." }
  ]
}
```

Notes:
- `useBuiltinAi: false` avoids the setup card / "Connecting to AI..." states
  (prompt_handler/mod.rs:8554-8611) and never auto-responds.
- `saveHistory: false` keeps the run read-only (no DB writes).
- `role`/`content` are the AI-SDK-compatible fields
  (src/protocol/types/chat.rs:40-80); turn pairing via
  `build_conversation_turns`.
- Streaming-state capture (optional second reference): follow with
  `{"type":"chatStreamStart","id":"design-chat-fixture","messageId":"m1"}` +
  one `chatStreamChunk` — renders the accent dot pulse
  (render_turns.rs:150-165). Animated; keep out of the primary compare.

Proposed block for `scripts/agentic/design-reference-capture.ts`
(`--screen chat`):

```ts
// DEFAULT_OUT.chat = "design/mockups/screens/chat-prompt/reference/chat-prompt@2x.png"
// CAPTURE_TARGET.chat = { type: "kind", kind: "main" }
if (screen === "chat") {
  // Fresh launches start in MainWindowMode::Mini (app_impl/startup.rs);
  // close_and_reset_window flips to Full (app_impl/lifecycle_reset.rs:190).
  // Round-trip Escape → show so the chat opens at DivPrompt height (750×500)
  // with full chrome. VERIFY via getState/getLayoutInfo that the window is
  // 500pt tall before capturing (mode is not directly exposed).
  await driver.request({ type: "simulateKey", key: "escape" } as never, { timeoutMs: 2_000 }).catch(() => {});
  await driver.waitForSettle();
  await driver.request({ type: "show" }, { timeoutMs: 2_000 }).catch(() => {});
  await driver.waitForSettle();
  await driver.request({
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
  } as never, { timeoutMs: 5_000 }).catch(() => {});
  await driver.waitForSettle();
  await Bun.sleep(600);
}
```

Also probe `activeFooter` in the same session and paste the receipt into
known-divergence (footer button labels are currently source-derived).

---

## 3. Proposed `src/design_contract/mod.rs` section

Following the existing `source_len`/`resolved_color` builder conventions
(`b` = `BundleBuilder`), stock dark, `base` theme:

```rust
// ── Chat prompt (SDK chat, src/prompts/chat/**) ─────────────────────────
b.source_len("chat.window.height", "--sk-chat-window-height",
    crate::config::defaults::DEFAULT_LAYOUT_STANDARD_HEIGHT,
    "config::defaults::DEFAULT_LAYOUT_STANDARD_HEIGHT");
b.source_len("chat.layout.paddingX", "--sk-chat-layout-padding-x",
    crate::prompts::CHAT_LAYOUT_PADDING_X, "prompts::chat::CHAT_LAYOUT_PADDING_X");
b.source_len("chat.section.paddingY", "--sk-chat-section-padding-y",
    crate::prompts::CHAT_LAYOUT_SECTION_PADDING_Y, "prompts::chat::CHAT_LAYOUT_SECTION_PADDING_Y");
// (requires pub(crate) re-export of the chat layout constants, or moving them
//  into a chat contract module — see §4.)

b.source_len("chat.header.gap", "--sk-chat-header-gap", 8.0,
    "prompts::chat::render_input::render_header gap");
b.add("chat.title.fontSize", Source, Some("--sk-chat-title-font-size"),
    Length { 14.0 }, Some("gpui text_sm (rems 0.875)"), false, &[]);
b.add("chat.title.lineHeight", Resolved, Some("--sk-chat-title-line-height"),
    Length { 23.0 }, Some("gpui phi default line height for 14pt"), false,
    &["chat.title.fontSize"]);
b.add("chat.title.fontWeight", Source, Some("--sk-chat-title-font-weight"),
    FontWeight { 500.0 }, Some("FontWeight::MEDIUM"), false, &[]);

b.resolved_color("chat.divider", "--sk-chat-divider",
    (theme.colors.ui.border << 8) | crate::prompts::CHAT_LAYOUT_BORDER_ALPHA,
    &["theme.ui.border", "prompts::chat::CHAT_LAYOUT_BORDER_ALPHA"]);
b.resolved_color("chat.text.secondary", "--sk-chat-text-secondary",
    (theme.colors.text.secondary << 8) | 0xFF, &["theme.text.secondary"]);

b.source_len("chat.input.minHeight", "--sk-chat-input-min-height", 28.0,
    "prompts::chat::render_input field min_h");
b.source_len("chat.input.fontSize", "--sk-chat-input-font-size", 14.0,
    "prompts::chat::render_input input_font_size (full mode)");
b.resolved_color("chat.input.surface", "--sk-chat-input-surface",
    chrome.input_surface_rgba, &["theme.background.search_box", "opacity.search_box"]);
b.resolved_color("chat.input.surfaceActive", "--sk-chat-input-surface-active",
    chrome.input_active_rgba, &["theme.background.search_box", "opacity.input_active"]);
b.resolved_color("chat.input.placeholder", "--sk-chat-placeholder-text",
    (theme.colors.text.muted << 8) | 0xFF, &["theme.text.muted"]); // CONFLICT C1

b.source_len("chat.turn.gap", "--sk-chat-turn-gap", 8.0,
    "prompts::chat::render_core turns pb");
b.source_len("chat.card.paddingX", "--sk-chat-card-padding-x",
    crate::prompts::CHAT_LAYOUT_CARD_PADDING_X, "prompts::chat::CHAT_LAYOUT_CARD_PADDING_X");
b.source_len("chat.card.paddingY", "--sk-chat-card-padding-y",
    crate::prompts::CHAT_LAYOUT_CARD_PADDING_Y, "prompts::chat::CHAT_LAYOUT_CARD_PADDING_Y");
b.source_len("chat.card.radius", "--sk-chat-card-radius", 8.0,
    "prompts::chat::render_turns card rounded");
b.resolved_color("chat.card.background", "--sk-chat-card-background",
    chat_turn_card_fill(&theme), // resolver, §4
    &["theme.text.primary", "prompts::chat::TURN_CARD_OVERLAY_ALPHA_DARK"]);
b.source_len("chat.card.rowGap", "--sk-chat-card-row-gap", 8.0,
    "prompts::chat::render_turns card gap");
b.source_len("chat.card.contentGap", "--sk-chat-card-content-gap", 6.0,
    "prompts::chat::render_turns content gap");
b.add("chat.user.fontWeight", Source, Some("--sk-chat-user-font-weight"),
    FontWeight { 600.0 }, Some("FontWeight::SEMIBOLD"), false, &[]);

b.source_len("chat.copy.buttonSize", "--sk-chat-copy-button-size", 24.0,
    "prompts::chat::render_turns copy button w/h");
b.source_len("chat.copy.buttonRadius", "--sk-chat-copy-button-radius", 4.0,
    "prompts::chat::render_turns copy rounded");
b.source_len("chat.copy.iconSize", "--sk-chat-copy-icon-size", 16.0,
    "prompts::chat::render_turns copy svg size");
b.add("chat.copy.opacity", Source, Some("--sk-chat-copy-opacity"),
    Number { 0.7 }, Some("prompts::chat::render_turns copy opacity"), true, &[]);

b.source_len("chat.md.blockGap", "--sk-chat-md-block-gap", 6.0,
    "prompts::markdown::api root gap");
b.source_len("chat.md.fontSize", "--sk-chat-md-font-size", 14.0,
    "prompts::markdown text_sm");
b.add("chat.md.lineHeight", Resolved, Some("--sk-chat-md-line-height"),
    Length { 23.0 }, Some("gpui phi default line height for 14pt"), false,
    &["chat.md.fontSize"]);

b.source_len("chat.code.marginY", "--sk-chat-code-margin-y", 4.0,
    "prompts::markdown::code_table container mt/mb");
b.source_len("chat.code.radius", "--sk-chat-code-radius", 6.0,
    "prompts::markdown::code_table container rounded");
b.resolved_color("chat.code.background", "--sk-chat-code-background",
    (prompt_colors.code_bg << 8) | 0xE0,
    &["theme.background.search_box", "markdown code block alpha 0xE0"]);
b.resolved_color("chat.code.headerBorder", "--sk-chat-code-header-border",
    (prompt_colors.quote_border << 8) | 0x30,
    &["theme.ui.border", "markdown code header alpha 0x30"]);
b.source_len("chat.code.headerPaddingX", "--sk-chat-code-header-padding-x", 10.0,
    "prompts::markdown::code_table header px");
b.source_len("chat.code.headerPaddingY", "--sk-chat-code-header-padding-y", 4.0,
    "prompts::markdown::code_table header py");
b.source_len("chat.code.labelFontSize", "--sk-chat-code-label-font-size", 12.0,
    "gpui text_xs (rems 0.75)");
b.source_len("chat.code.bodyPaddingX", "--sk-chat-code-body-padding-x", 10.0,
    "prompts::markdown::code_table body px");
b.source_len("chat.code.bodyPaddingY", "--sk-chat-code-body-padding-y", 8.0,
    "prompts::markdown::code_table body py");
b.source_len("chat.code.lineGap", "--sk-chat-code-line-gap", 2.0,
    "prompts::markdown::code_table body gap");
b.source_len("chat.code.lineMinHeight", "--sk-chat-code-line-min-height", 16.0,
    "prompts::markdown::code_table line min_h");

b.source_len("chat.inlineCode.paddingX", "--sk-chat-inline-code-padding-x", 4.0,
    "prompts::markdown::inline_render code px");
b.source_len("chat.inlineCode.paddingY", "--sk-chat-inline-code-padding-y", 1.0,
    "prompts::markdown::inline_render code py");
b.source_len("chat.inlineCode.radius", "--sk-chat-inline-code-radius", 3.0,
    "prompts::markdown::inline_render code rounded");
b.resolved_color("chat.inlineCode.background", "--sk-chat-inline-code-background",
    (prompt_colors.code_bg << 8) | 0x80,
    &["theme.background.search_box", "markdown inline code alpha 0x80"]);
```

Reused existing tokens (no new records): `--sk-caret-width`,
`--sk-caret-height`, `--sk-color-accent`, `--sk-font-mono`,
`--sk-window-native-footer-host-height`, `--sk-footer-*`,
`--sk-window-radius`, `--sk-window-main-width`, `--sk-color-text-primary`,
`--sk-window-divider-height`.

---

## 4. Resolver-extraction plan

The chat renderers bake alphas inline; extract pure resolvers so the exporter
and renderers share one authority:

1. **Turn card fill** — file `src/prompts/chat/render_turns.rs` (fn
   `render_turn`, :21-26). Extract to `src/prompts/chat/types.rs` (pure, no
   gpui linkage):
   `pub(crate) fn chat_turn_card_fill_rgba(text_primary: u32, is_dark: bool) -> u32`
   returning `(text_primary << 8) | if is_dark { 0x15 } else { 0x08 }`, with
   `TURN_CARD_OVERLAY_ALPHA_DARK: u32 = 0x15` /
   `TURN_CARD_OVERLAY_ALPHA_LIGHT: u32 = 0x08` consts. Renderer calls it via
   `theme::hover_overlay_bg`-style packing; exporter calls it directly.
2. **Chat divider** — `render_core.rs:351-354` + `render_input.rs:94-95`
   duplicate `(border << 8) | CHAT_LAYOUT_BORDER_ALPHA`. Extract
   `pub(crate) fn chat_divider_rgba(border_hex: u32) -> u32` next to the
   `CHAT_LAYOUT_*` consts in `src/prompts/chat/mod.rs:52-57`, and make those
   consts `pub(crate)` re-exported through `crate::prompts` for the exporter.
3. **Markdown surface fills** — `src/prompts/markdown/code_table.rs:36,46,48,
   85,180-181` and `inline_render.rs:15,36` hardcode `0xE0/0x80/0x40/0x30/
   0xC0` over `PromptColors`. Extract to `src/prompts/markdown/helpers.rs`:
   `pub(super) fn code_block_fill_rgba(colors: &PromptColors) -> u32`,
   `code_block_border_rgba`, `code_header_border_rgba`,
   `inline_code_fill_rgba`, `table_header_fill_rgba`, `table_row_alt_rgba` —
   all pure `(hex << 8) | ALPHA` fns with named alpha consts. Exporter
   consumes the same fns via a `pub(crate)` façade
   (`markdown::contract_fills(colors)` returning a small struct).
4. **Input surfaces** — already resolved via
   `theme::AppChromeColors::from_theme` (chrome.rs:144-147,191-194); exporter
   reads `chrome.input_surface_rgba` / `input_active_rgba` directly. No
   extraction needed.
5. **Line heights** — expose the phi resolver used this contract cycle
   (`round(font_size * 1.618034)`) as a shared
   `design_contract::phi_line_height(px: f32) -> f32` so `chat.title.lineHeight`
   and `chat.md.lineHeight` are derived, not retyped.

---

## 5. Expected conflicts (record, do not silently collapse)

- **C1 — placeholder grading:** chat placeholder = `rgb(text.muted)` at full
  alpha (render_input.rs:9) → pure white in stock dark; main-menu placeholder
  = `text.primary` @ `opacity.text_placeholder` 0.40 (`--sk-text-placeholder`
  rgb(255 255 255 / 0.40)). Two live composer paths disagree on placeholder
  emphasis. Severity: visual-inconsistency.
- **C2 — stock text grading is flat:** `text.primary/secondary/tertiary/muted`
  are all `0xffffff` in script-kit-dark (presets.rs:955-958), so chat's
  primary/secondary/tertiary distinctions resolve identically. Contract keeps
  distinct tokens with equal resolved values (they diverge in other themes,
  e.g. Dracula presets.rs:1018-1021). Severity: informational.
- **C3 — mini_mode initialization:** `ChatPrompt::new` defaults
  `mini_mode: false` (prompt.rs:177) and `ShowChat` never seeds it from
  `main_window_mode`; only a mode CHANGE syncs it
  (ui_window.rs:2177-2182,2224-2229). A fresh launch (startup mode Mini,
  app_impl/startup.rs) that opens chat gets a MiniAiChat-sized window (480)
  hosting full-mode chrome. Severity: behavior bug candidate; the capture
  recipe works around it (§2).
- **C4 — focus state invisible:** `input_active` == `search_box` == 0.50 in
  both stock opacity presets (types.rs:257,266), so the focused-field fill is
  indistinguishable from idle. Severity: informational (theme data, not code).
- **C5 — dead protocol fields:** `Message::Chat.hint` and `.footer` are
  stored (prompt.rs:6-7) but never rendered — `render_header` shows only the
  title; the footer status line is built from model/streaming state only
  (render_core.rs:5-27). Protocol model ≠ painted truth. Severity:
  contract-mismatch.
- **C6 — footer verb:** native footer for `chat_prompt` says `Run ↵`
  (standard buttons, ui_window.rs:746 + :143-175 default arm) while Enter
  actually submits a chat message; Agent Chat says `Send`. Receipt-pending;
  if the probe confirms "Run", record as UX-inconsistency.
- **C7 — GPUI capture vs user-visible footer:** window captures show an empty
  36pt band; painted truth for buttons lives in the AppKit overlay
  (activeFooter probe only). Inherited from the footer architecture.

---

## 6. Tokens to generate (44 new `--sk-chat-*` records)

Lengths/numbers (Source unless noted):
`--sk-chat-window-height` (500, resolved from layout),
`--sk-chat-layout-padding-x` (12), `--sk-chat-section-padding-y` (8),
`--sk-chat-header-gap` (8), `--sk-chat-title-font-size` (14),
`--sk-chat-title-line-height` (23, Resolved/phi),
`--sk-chat-title-font-weight` (500), `--sk-chat-input-area-gap` (8),
`--sk-chat-input-min-height` (28), `--sk-chat-input-font-size` (14),
`--sk-chat-turn-gap` (8), `--sk-chat-card-padding-x` (12),
`--sk-chat-card-padding-y` (10), `--sk-chat-card-radius` (8),
`--sk-chat-card-row-gap` (8), `--sk-chat-card-content-gap` (6),
`--sk-chat-user-font-weight` (600), `--sk-chat-md-block-gap` (6),
`--sk-chat-md-font-size` (14), `--sk-chat-md-line-height` (23, Resolved/phi),
`--sk-chat-copy-button-size` (24), `--sk-chat-copy-button-radius` (4),
`--sk-chat-copy-icon-size` (16), `--sk-chat-copy-opacity` (0.7),
`--sk-chat-code-margin-y` (4), `--sk-chat-code-radius` (6),
`--sk-chat-code-header-padding-x` (10), `--sk-chat-code-header-padding-y` (4),
`--sk-chat-code-label-font-size` (12), `--sk-chat-code-body-padding-x` (10),
`--sk-chat-code-body-padding-y` (8), `--sk-chat-code-line-gap` (2),
`--sk-chat-code-line-min-height` (16), `--sk-chat-inline-code-padding-x` (4),
`--sk-chat-inline-code-padding-y` (1), `--sk-chat-inline-code-radius` (3).

Colors (Resolved):
`--sk-chat-divider` rgb(52 52 52 / 0.2510),
`--sk-chat-text-secondary` rgb(255 255 255),
`--sk-chat-input-surface` rgb(42 42 42 / 0.5020),
`--sk-chat-input-surface-active` rgb(42 42 42 / 0.5020),
`--sk-chat-placeholder-text` rgb(255 255 255),
`--sk-chat-card-background` rgb(255 255 255 / 0.0824),
`--sk-chat-code-background` rgb(42 42 42 / 0.8784),
`--sk-chat-code-header-border` rgb(52 52 52 / 0.1882),
`--sk-chat-inline-code-background` rgb(42 42 42 / 0.5020).

(That is 36 + 9 = 45 names; `--sk-chat-input-surface-active` may be dropped as
a duplicate of `--sk-chat-input-surface` in stock — keep both so non-stock
opacity presets diverge correctly → 44–45 records.)

Until these land, `screen.css` stages the values under
`--sk-emulator-staged-chat-*` and aliases them (see the `:root` contract
entry in known-divergence.json). Delete that block when the exporter emits
the real records.

---

## 7. Open unknowns

- Footer button labels/selection for `chat_prompt` need a live `activeFooter`
  receipt (C6/C7).
- The Mini→Full round-trip in the capture recipe (Escape → show) needs a
  `getLayoutInfo`/window-height receipt proving 500pt before the shot (C3).
- Band totals (header 40, input 45) are derived from source; pixel-verify
  against the first reference capture.
- Whether the empty composer paints the caret before or after the
  placeholder run (shared text-input painter internals) — mockup assumes
  caret at text origin, placeholder immediately after; verify at 2x.
