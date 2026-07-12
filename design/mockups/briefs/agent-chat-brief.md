# Agent Chat — design-contract mockup brief

Screen slice for `design/mockups/screens/agent-chat/`. Covers the Pi-backed
Agent Chat surface (`src/ai/agent_chat/**`), NOT the script chat prompt
(`src/prompts/chat/**`). Fixture: `openAgentChatKitchenSinkFixture`, Standard
UI variant, embedded in the main window, script-kit-dark + InfoBarBase.

Tags: **MEASURED** = read from source at the cited line. **GUESS** = inferred,
needs capture/probe confirmation.

---

## 1. Surface anatomy (with citations)

### 1.1 Host shell

- The Standard Agent Chat renders through the SAME main-view chrome as the
  launcher: `render_main_view_shell()` → `render_main_view_chrome(root, …,
  MainViewChrome { header, divider, main, footer, overlays })` —
  `src/ai/agent_chat/ui/view.rs:14488`, `view.rs:14804-14815`. **MEASURED**
- The kitchen-sink fixture opens EMBEDDED in the main window
  (`open_agent_chat_kitchen_sink_fixture`,
  `src/app_impl/agent_handoff/agent_chat_launch.rs:101-151`;
  `enter_embedded_agent_chat_surface`,
  `src/app_impl/agent_chat_surface_transitions.rs:40-72`). Window geometry is
  therefore the existing main-window tokens: `--sk-window-main-width` 750,
  `--sk-window-main-height` 480, `--sk-window-radius` 22. **MEASURED** (tokens)
  / **GUESS** that no view-specific resize fires for the embedded chat — the
  capture receipt must confirm 750×480.
- Divider chrome renders with `visible: false` (`view.rs:14575-14579`).
  **MEASURED**
- Detached variant (not this fixture): PopUp window, default bounds 480×440 at
  (100,100), `WindowBackgroundAppearance::Blurred` when vibrancy is on,
  automation kind `agentChatDetached`
  (`src/ai/agent_chat/ui/chat_window.rs:65-125,156-174`). **MEASURED**

### 1.2 Header context zone (Cwd chip + Agent·Model chip)

- Rendered by the shared `render_main_view_context_zone_required`
  (`src/components/main_view_chrome.rs:368-632`) with the main-menu
  InfoBarBase tokens already exported (`--sk-main-menu-context-*`). **MEASURED**
- Agent Chat specifics (`view.rs:14542-14574`):
  - cwd label = `AgentChatFooterSnapshot.cwd_display` (home-relativized;
    fixture cwd = `std::env::temp_dir()/script-kit-agent-chat-kitchen-sink-fixture`,
    `agent_chat_launch.rs:114`, formatting `view.rs:1322-1332`). **MEASURED**
  - Tab chip is `MainViewTabChipAction::Inactive` → cwd label WITHOUT the ⇥
    keycap (`view.rs:14554-14557`; keycap suppression
    `main_view_chrome.rs:398-412`). **MEASURED**
  - agent·model chip = `agent_model_header_label()` = `"{profile} · {model}"`
    (`view.rs:348-358`). For the fixture both resolve to
    `"Agent Chat Kitchen Sink"` (profile_display_name
    `agent_chat_launch.rs:120`; `selected_model_display` falls back to
    `display_name` when no model is selected, `thread.rs:2990-2994`; both set
    from `fixture.title`, `agent_chat_launch.rs:117`) → the chip reads
    `"Agent Chat Kitchen Sink · Agent Chat Kitchen Sink"`. **MEASURED**
  - `shift_tab_key_active` defaults true (`main_view_chrome.rs:71`) → ⇧⇥
    keycaps stay on the agent·model chip. **MEASURED**
- InfoBarBase pill values: `pill_border_alpha 0x00`, `pill_bg_alpha 0x00`,
  `pill_padding_x 6`, `pill_radius 14`, opacity 0.34, key_opacity 0.50,
  height 22, gap 7, layout Split (no "·" separator element between lanes)
  (`src/designs/core/main_menu_theme.rs:479-482,614-639`). **MEASURED** —
  note `show_pills` is true (padding > 0) so a fully-transparent border/bg IS
  painted; visually a no-op.

### 1.3 Composer (header input slot)

- `render_composer_input_shell` wraps the shared
  `render_main_view_input_shell_with_height` (`view.rs:8624-8748`;
  `main_view_chrome.rs:801-842`). The embedded default path derives its
  font size, font weight, line height, and growth increment from
  `current_main_menu_theme().def().search`: **20 / 430 / 26**. Shell height
  is **26px** for one line and adds the canonical 26px search height for each
  extra visible line (`composer_height_for_visual_lines`, `view.rs:1118-1133`;
  `composer_visible_line_count` clamps 1..6, `view.rs:502-508`). **MEASURED**
- The older 17/22/default-weight metrics remain deliberately scoped to the
  detached/experimental layouts and focused-text-mini early return. Setup
  returns render the setup card before composer styling is resolved. They are
  not the embedded `AppView::AgentChatView` canonical multiline contract.
  **MEASURED**
- Text insets come from the shared shell: left = `search.text_inset_x` (16 =
  `SEARCH_INPUT_TEXT_INSET_X_PX`, `src/ui/chrome/tokens.rs:33`), right =
  `text_inset_x × 0.5` (`main_view_chrome.rs:808,826-835`). InfoBarBase
  search `surface_alpha`/`border_alpha` are `0x00`
  (`main_menu_theme.rs:734-743`) — the shell paints no visible box.
  **MEASURED**
- Empty state: pulsing caret bar `CURSOR_WIDTH 2.5 × CURSOR_HEIGHT_LG 18`
  (`src/panel.rs:69,81`) in `text.primary`, then the placeholder pulled left
  by `-CURSOR_WIDTH` so its origin sits at the caret origin
  (`view.rs:8522-8541`). Placeholder text: `"Ask anything…"` when the
  transcript is empty, `"Follow up…"` otherwise (`view.rs:8695-8699`);
  placeholder color = `AppChromeColors.placeholder_text_rgba`
  (`view.rs:14368-14369`) = existing `--sk-text-placeholder`. **MEASURED**
- Kitchen-sink composer state: `load_kitchen_sink_fixture` clears the input
  (`thread.rs:3515`) despite `initial_input` being set at thread init
  (`agent_chat_launch.rs:115`), and the transcript is non-empty → empty
  input, `"Follow up…"`. **MEASURED**
- Send button (`render_send_button_for_state`, `view.rs:10907-10982`): 24×24,
  radius 6, `text_sm`. States: idle+empty → `↑`, bg `text.primary@0x06`,
  opacity 0.30; idle+text → `↑`, bg `accent@0x30`, opacity 0.90;
  streaming+text → `⇧` queue, bg `accent@0x24`, opacity 0.92;
  streaming+empty → `●` dot, transparent, opacity 0.40. Fixture shows the
  disabled state. **MEASURED**
- Context chips: pending-context chips are NO LONGER rendered as a chip strip
  in the Standard flow — the render call is commented out ("Context chips
  removed — all attachments are now inline @type:name tokens",
  `view.rs:14867-14868`); attachments surface as accent-highlighted inline
  `@type:name` mentions inside the composer text
  (`attached_inline_mention_highlight_ranges`, `view.rs:14371-14384`).
  `render_pending_context_chips` (`view.rs:9830-9944`) still exists for other
  paths. The fixture has zero pending parts, so neither appears. **MEASURED**
- Transient lanes above/below the transcript reserve fixed heights even when
  empty (`render_reserved_transient_lane` calls, `view.rs:14602-14619,
  14683-14718`); lane height constants
  (`AGENT_CHAT_TRANSIENT_*_LANE_HEIGHT_PX`) were not chased to their
  definitions — **GUESS** that they reserve 0-visible-paint in the idle
  fixture; confirm against the capture before treating header→transcript gap
  as blocking.

### 1.4 Transcript rows

All row styling flows through `AgentChatStyleDef` — base values in
`base_agent_chat_style()` (`src/dev_style_tool/agent_chat_catalog.rs:89-163`),
applied via `effective_agent_chat_style()` (base + dev-tool runtime overrides,
`src/dev_style_tool/runtime_overrides.rs:639-648`). Checked-in mockups mirror
the BASE def (same rule as the exporter's `base_def()` policy). All values
below **MEASURED** at `agent_chat_catalog.rs:89-163` unless noted.

- Row wrapper (`transcript.rs:1586-1608`): `px(row_padding_x 16)`,
  `pb(row_padding_bottom 4)`; response-start rows add
  `mt(response_start_margin_top 4)`; new-turn user rows add
  `mt(turn_margin_top 8) + pt(turn_padding_top 8) + border_t_1` in
  `ui.border@turn_divider_alpha 0x18` → `rgb(52 52 52 / 0.0941…)`.
- List: `ListState::new(len+1, ListAlignment::Bottom, px(200)).measure_all()`
  with `follow_tail(true)` (`transcript.rs:171-172`) — viewport pins to the
  conversation tail; +1 row is the permanent synthetic activity tail row
  (`render_activity_row`: 7px accent dot pulsing, "Thinking…" in
  `text.primary@0xB0`, gap 8, `transcript.rs:803-838`) — hidden (0px) in the
  idle fixture.
- User message (`render_user_message`, `transcript.rs:841-975`): full-width
  bubble `px 12 / py 8`, radius 8, bg `text.primary@0x06` →
  `rgb(255 255 255 / 0.0235…)`. `max_width 520` applies ONLY to the RoleSplit
  presentation (`transcript.rs:965-974`); Standard is full-width.
  Hover-reveal fork/edit button 22×22 radius 6 at top-right (skipped in the
  static mockup).
- Assistant message (`transcript.rs:977-1020`): `px 12 / py 4`, radius 0,
  bg_alpha 0 (no surface painted — the `.when(bg_alpha > 0)` guard).
- Markdown bodies: gpui-component `TextView` with `TextViewStyle`
  (`build_transcript_text_style`, `transcript.rs:584-640`): body 14px,
  paragraph gap `rems(0.28)`, headings 17/16/15px (h1/h2/h3, else body);
  code block: bg `background.search_box 0x2A2A2A @0xA0`, 1px border
  `ui.border 0x343434 @0x40`, radius 5, padding 7×4, text 13px, copy button
  enabled; blockquote: bg `ui.border@0x10`, border `@0x40`, radius 5,
  padding 12×6. Body line height: GPUI default `round(14 × 1.618034)` =
  **23px** (`vendor/gpui/src/style.rs`, verified this session). `rems(0.28)`
  = 4.48px at the default 16px rem — **GUESS** on the rem base.
- Thought/Tool collapsible block (`render_collapsible_block`,
  `transcript.rs:1227-1366`): `pl/pr 12, py 2`, `border_l_2`;
  thought border `text.primary@0x7F`, tool border `accent@0x7F`
  (error tools: `ui.error@0x7F`); header opacity 0.75, status/status-hint
  opacity 0.50; expanded body `pt 4`, `max_h 200`, `overflow_y_hidden`.
  Collapse default: Thought/Tool start collapsed EXCEPT tools with a diff or
  `is_error` (`default_expanded`, `transcript.rs:411-415`) — so the two
  kitchen-sink meta tool cards render expanded with no user interaction.
- Tool card (`render_tool_card`, `transcript.rs:1087-1224`): header =
  chevron ▸/▾ (accent, 0.75) + status glyph (○ ● ✓ ✕;
  pending `text.primary@0x80`, running accent, complete `ui.success 0x00FF00`,
  failed `ui.error 0xEF4444`; `tool_card.rs:70-77`, colors
  `transcript.rs:1098-1103`) + kind-glyph+name (accent 0.75; glyphs 📄 ✎ 📝 ❯
  🔍 🌐 ⚙, `tool_card.rs:38-48`) + mono subject at code-block size 13px,
  opacity 0.50 + red "failed" label for failed tools + "N lines" badge when
  collapsed. Non-diff tool bodies render markdown in the ACCENT color
  (`transcript.rs:1212-1218`).
- Diff body (`render_diff_body`, `transcript.rs:1029-1082`): mono 13px in a
  code-block-styled box (`search_box@0xA0`, radius 5, padding 7×4); added
  lines `ui.success` on `success@0x14`, removed `ui.error` on `error@0x14`,
  context `text.primary` at opacity 0.55; 200-row cap.
- Error row (`render_error_message`, `transcript.rs:1368-1418`): `px 12/py 8`,
  radius 8, bg `0xEF4444@50` (DECIMAL 50 → 0.196…), `border_l_2`
  `0xEF4444@0x80`; "⚠ Error" label opacity 0.75 semibold; footer hint
  opacity 0.40. (Not visible in the mocked tail.)
- System row (`render_system_message`, `transcript.rs:1420-1445`):
  `px 12/py 4`, whole-row opacity 0.60, `border_l_2` `ui.border@0x30`.
- Heavy-markdown preview path exists for scroll-heavy bodies
  (`transcript.rs:657-713`) — none of the kitchen-sink tail rows trip the
  thresholds (`is_scroll_heavy`, `transcript.rs:82-93`). **MEASURED**
  (thresholds) / **GUESS** (that no tail row trips them — msg 5 earlier in
  the transcript might, but it is above the visible tail).

### 1.5 Footer

- Embedded-in-main: when the native main-window footer surface is
  `"agent_chat"`, GPUI renders only
  `render_native_main_window_footer_spacer()` (`view.rs:14776-14786`) — the
  36pt band (`--sk-window-native-footer-host-height`) is EMPTY in a GPUI
  capture; the real content is the native AppKit overlay (32pt rail inside
  the 36pt host — same architecture as the main menu). **MEASURED** (code
  path) / native truth via `activeFooter` probe.
- Button truth (`footer_buttons_for_thread`, `view.rs:1417-1500`): idle +
  empty input + pastable assistant response → `Paste Response ↵`; otherwise
  `Send ↵` (disabled while blank); streaming → `Stop Esc`; always followed by
  `Actions ⌘K`. Kitchen sink (idle, cleared input, assistant messages
  present) → **Paste Response ↵ + Actions ⌘K**, given
  `has_pastable_assistant_response` accepts fixture assistant rows —
  **GUESS** on that predicate (not read); if false, the button is `Send ↵`
  disabled. Confirm with an activeFooter probe.
- Left info (`profile_left_info`, `view.rs:369-390`): status dot +
  `model_status_label` + profile name + cwd chip (folder icon token). Exact
  native layout/gaps are AppKit-side — **GUESS**, marked non-blocking native
  in known-divergence.
- Detached window: same config pushed through
  `footer_popup::sync_window_footer_popup` with
  `agent_chat_detached_native_footer_config` (`view.rs:1365-1390`,
  `view.rs:14764-14775`). **MEASURED**

---

## 2. Fixture strategy + capture

### Why `openAgentChatKitchenSinkFixture`

- Deterministic, provider-free, exercises every role + markdown primitive +
  the REAL tool-card pipeline (structured meta routed through
  `upsert_tool_call_start`/`apply_tool_call_update`,
  `thread.rs:3532-3561`). Protocol JSON:
  `{"type": "openAgentChatKitchenSinkFixture", "requestId": "req-kitchen"}`
  (`src/stdin_commands/mod.rs:480-483,2138-2144`; handler
  `src/main_entry/runtime_stdin.rs:567-591`). It opens in the MAIN window and
  emits an `externalCommandResult` receipt.
- `setAgentChatTestFixture` (`stdin_commands/mod.rs:588-598`) mutates an
  ALREADY-OPEN thread (phases awaitingFirstAssistantText / assistantText /
  idle / error) — useful for state-variant captures, not the baseline.
- `openAgentChatDetachedFixture` (`stdin_commands/mod.rs:468-471`) opens only
  the detached PLACEHOLDER shell (no thread, `chat_window.rs:177-238` +
  fixture bounds 640×520 at (585,177), `runtime_stdin.rs:537-558`) — wrong
  fixture for transcript pixels.

### Proposed `design-reference-capture.ts` extension

Add to `DEFAULT_OUT` / `CAPTURE_TARGET` and the per-screen block in
`scripts/agentic/design-reference-capture.ts` (pattern at lines 25-34, 53-73):

```ts
// DEFAULT_OUT
"agent-chat": "design/mockups/screens/agent-chat/reference/agent-chat@2x.png",
// CAPTURE_TARGET — kitchen sink is EMBEDDED in the main window
"agent-chat": { type: "kind", kind: "main" },

// per-screen block
if (screen === "agent-chat") {
  await driver.request(
    { type: "openAgentChatKitchenSinkFixture", requestId: "design-ref-agent-chat" } as never,
    { timeoutMs: 5_000 },
  ).catch(() => {});
  await driver.waitForSettle();
  await Bun.sleep(800); // markdown TextView layout + measure_all settle
}
```

Capture target kind is `main` (`AutomationWindowKind::Main` → `"main"`,
`src/protocol/types/automation_window.rs:32-55`); the embedded surface rekeys
the main automation surface but the window kind stays `main`
(`agent_chat_surface_transitions.rs:50`). A detached-variant capture would
target `{ type: "kind", kind: "agentChatDetached" }`
(`automation_window.rs:37,59`).

Footer receipts: pair the capture with a protocol `activeFooter`-style probe
(as the main-menu screen did) to record surface `"agent_chat"`, the button
list, and the left-info labels, since the PNG's footer band is empty.

---

## 3. Proposed `src/design_contract/mod.rs` section

Follow the existing `BundleBuilder` idioms (`source_len` / `resolved_color` /
`conflict`). New section after the actions-dialog block:

```rust
// ── Agent Chat (embedded chat surface) ──────────────────────────────
// Base style only — checked-in artifacts never read the dev-style-tool
// runtime overrides that `effective_agent_chat_style()` applies.
let chat = crate::dev_style_tool::agent_chat_catalog::base_agent_chat_style();

// transcript
b.source_len("agentChat.transcript.rowPaddingX", "--sk-agent-chat-row-padding-x",
    chat.transcript.row_padding_x, "AgentChatTranscriptStyle.row_padding_x");
b.source_len("agentChat.transcript.rowPaddingBottom", "--sk-agent-chat-row-padding-bottom",
    chat.transcript.row_padding_bottom, "AgentChatTranscriptStyle.row_padding_bottom");
b.source_len("agentChat.transcript.responseStartMarginTop", "--sk-agent-chat-response-start-margin-top",
    chat.transcript.response_start_margin_top, "AgentChatTranscriptStyle.response_start_margin_top");
b.source_len("agentChat.transcript.turnMarginTop", "--sk-agent-chat-turn-margin-top",
    chat.transcript.turn_margin_top, "AgentChatTranscriptStyle.turn_margin_top");
b.source_len("agentChat.transcript.turnPaddingTop", "--sk-agent-chat-turn-padding-top",
    chat.transcript.turn_padding_top, "AgentChatTranscriptStyle.turn_padding_top");
b.resolved_color("resolved.agentChat.transcript.turnDivider", "--sk-agent-chat-turn-divider",
    (colors.ui.border << 8) | chat.transcript.turn_divider_alpha.round() as u32,
    &["theme.colors.ui.border", "AgentChatTranscriptStyle.turn_divider_alpha"]);

// markdown
b.source_len("agentChat.markdown.bodyFontSize", "--sk-agent-chat-md-body-font-size",
    chat.markdown.body_font_size, "AgentChatMarkdownStyle.body_font_size");
// resolved GPUI default line height: (body_font_size * 1.618034).round()
b.add("resolved.agentChat.markdown.bodyLineHeight", TokenStage::Resolved,
    Some("--sk-agent-chat-md-body-line-height"),
    TokenValue::Length { value: (chat.markdown.body_font_size as f64 * 1.618034).round() },
    None, false, &["agentChat.markdown.bodyFontSize"]);
b.source_len("agentChat.markdown.h1FontSize", "--sk-agent-chat-md-h1-font-size",
    chat.markdown.heading_1_font_size, "AgentChatMarkdownStyle.heading_1_font_size");
b.source_len("agentChat.markdown.h2FontSize", "--sk-agent-chat-md-h2-font-size",
    chat.markdown.heading_2_font_size, "AgentChatMarkdownStyle.heading_2_font_size");
b.source_len("agentChat.markdown.h3FontSize", "--sk-agent-chat-md-h3-font-size",
    chat.markdown.heading_3_font_size, "AgentChatMarkdownStyle.heading_3_font_size");
b.source_len("agentChat.markdown.codeFontSize", "--sk-agent-chat-md-code-font-size",
    chat.markdown.code_block_font_size, "AgentChatMarkdownStyle.code_block_font_size");
b.source_len("agentChat.markdown.codePaddingX", "--sk-agent-chat-md-code-padding-x",
    chat.markdown.code_block_padding_x, "AgentChatMarkdownStyle.code_block_padding_x");
b.source_len("agentChat.markdown.codePaddingY", "--sk-agent-chat-md-code-padding-y",
    chat.markdown.code_block_padding_y, "AgentChatMarkdownStyle.code_block_padding_y");
b.source_len("agentChat.markdown.codeRadius", "--sk-agent-chat-md-code-radius",
    chat.markdown.code_block_radius, "AgentChatMarkdownStyle.code_block_radius");
b.resolved_color("resolved.agentChat.markdown.codeBg", "--sk-agent-chat-md-code-bg",
    (colors.background.search_box << 8) | chat.markdown.code_block_bg_alpha.round() as u32,
    &["theme.colors.background.searchBox", "AgentChatMarkdownStyle.code_block_bg_alpha"]);
b.resolved_color("resolved.agentChat.markdown.codeBorder", "--sk-agent-chat-md-code-border",
    (colors.ui.border << 8) | chat.markdown.code_block_border_alpha.round() as u32,
    &["theme.colors.ui.border", "AgentChatMarkdownStyle.code_block_border_alpha"]);
// paragraph_gap is authored in rems — export as Number + note, not Length
b.add("agentChat.markdown.paragraphGapRems", TokenStage::Source, None,
    TokenValue::Number { value: chat.markdown.paragraph_gap as f64 },
    Some("AgentChatMarkdownStyle.paragraph_gap (rems)"), true, &[]);

// user / assistant messages
b.source_len("agentChat.user.paddingX", "--sk-agent-chat-user-padding-x",
    chat.user_message.padding_x, "AgentChatMessageStyle(user).padding_x");
b.source_len("agentChat.user.paddingY", "--sk-agent-chat-user-padding-y",
    chat.user_message.padding_y, "AgentChatMessageStyle(user).padding_y");
b.source_len("agentChat.user.radius", "--sk-agent-chat-user-radius",
    chat.user_message.radius, "AgentChatMessageStyle(user).radius");
b.resolved_color("resolved.agentChat.user.bg", "--sk-agent-chat-user-bg",
    (colors.text.primary << 8) | chat.user_message.bg_alpha.round() as u32,
    &["theme.colors.text.primary", "AgentChatMessageStyle(user).bg_alpha"]);
b.source_len("agentChat.assistant.paddingX", "--sk-agent-chat-assistant-padding-x",
    chat.assistant_message.padding_x, "AgentChatMessageStyle(assistant).padding_x");
b.source_len("agentChat.assistant.paddingY", "--sk-agent-chat-assistant-padding-y",
    chat.assistant_message.padding_y, "AgentChatMessageStyle(assistant).padding_y");
// declared-but-ineffective in Standard presentation: user/assistant max_width
// (RoleSplit only), assistant radius (bg_alpha 0 → surface never painted)
// → export id-only, writable:false, no css var (same idiom as the
// actions-dialog "Declared-but-ineffective" block, mod.rs:1253-1292).

// collapsible / tool cards
b.source_len("agentChat.block.paddingX", "--sk-agent-chat-block-padding-x",
    chat.collapsible.padding_x, "AgentChatCollapsibleStyle.padding_x");
b.source_len("agentChat.block.paddingY", "--sk-agent-chat-block-padding-y",
    chat.collapsible.padding_y, "AgentChatCollapsibleStyle.padding_y");
b.source_len("agentChat.block.bodyPaddingTop", "--sk-agent-chat-block-body-padding-top",
    chat.collapsible.body_padding_top, "AgentChatCollapsibleStyle.body_padding_top");
b.source_len("agentChat.block.maxBodyHeight", "--sk-agent-chat-block-max-body-height",
    chat.collapsible.max_body_height, "AgentChatCollapsibleStyle.max_body_height");
b.resolved_color("resolved.agentChat.tool.border", "--sk-agent-chat-tool-border",
    (colors.accent.selected << 8) | chat.collapsible.tool_border_alpha.round() as u32,
    &["theme.colors.accent.selected", "AgentChatCollapsibleStyle.tool_border_alpha"]);
b.resolved_color("resolved.agentChat.tool.borderError", "--sk-agent-chat-tool-border-error",
    (colors.ui.error << 8) | chat.collapsible.tool_border_alpha.round() as u32,
    &["theme.colors.ui.error", "AgentChatCollapsibleStyle.tool_border_alpha"]);
b.resolved_color("resolved.agentChat.thought.border", "--sk-agent-chat-thought-border",
    (colors.text.primary << 8) | chat.collapsible.thought_border_alpha.round() as u32,
    &["theme.colors.text.primary", "AgentChatCollapsibleStyle.thought_border_alpha"]);
// header/status opacities → TokenValue::Number sources

// error / system rows → same pattern (note error.bg_alpha is DECIMAL 50)

// embedded default composer: aliases, not new Agent Chat literals
// --sk-agent-chat-composer-font-size   → --sk-main-menu-search-font-size
// --sk-agent-chat-composer-font-weight → --sk-main-menu-search-font-weight
// --sk-agent-chat-composer-line-height → --sk-main-menu-search-height
// --sk-agent-chat-composer-height      → --sk-main-menu-search-height

// send button
b.source_len("agentChat.send.size", "--sk-agent-chat-send-size", 24.0,
    "render_send_button_for_state size");
b.source_len("agentChat.send.radius", "--sk-agent-chat-send-radius", 6.0,
    "render_send_button_for_state radius");
b.resolved_color("resolved.agentChat.send.disabledBg", "--sk-agent-chat-send-disabled-bg",
    (colors.text.primary << 8) | 0x06,
    &["theme.colors.text.primary", "render_send_button_for_state (false,false) arm"]);
```

(The remaining send literals require the §4 extraction to become real
`rust_path`s instead of transcribed numbers; the composer aliases already
have canonical main-menu source records.)

## 4. Resolver-extraction plan

The actions-dialog section works because `resolved_actions_dialog_row_chrome`
etc. are pure functions. Agent Chat needs the same:

1. **File**: `src/ai/agent_chat/ui/style_contract.rs` (new, pub(crate),
   re-exported from `src/ai/agent_chat/ui/mod.rs`).
2. Keep legacy composer constants explicitly scoped to focused-text-mini and
   detached/experimental layouts. The embedded default composer must continue
   reading `current_main_menu_theme().def().search`; only send-button
   size/radius/state alphas need Agent Chat-owned export records.
3. **Proposed pure resolver signatures**:
   ```rust
   pub(crate) struct ResolvedAgentChatTranscriptChrome {
       pub turn_divider_rgba: u32,
       pub user_bg_rgba: u32,
       pub tool_border_rgba: u32,
       pub tool_border_error_rgba: u32,
       pub thought_border_rgba: u32,
       pub code_bg_rgba: u32,
       pub code_border_rgba: u32,
       pub blockquote_bg_rgba: u32,
       pub blockquote_border_rgba: u32,
       pub system_border_rgba: u32,
       pub error_bg_rgba: u32,
       pub error_border_rgba: u32,
       pub body_line_height: f32, // (body_font_size * 1.618034).round()
   }
   pub(crate) fn resolved_agent_chat_transcript_chrome(
       style: &AgentChatStyleDef,
       theme: &crate::theme::Theme,
   ) -> ResolvedAgentChatTranscriptChrome;

   ```
4. Rewire `transcript.rs` color-packing call sites
   (`(theme.colors.X << 8) | style.Y.round() as u32` expressions at
   transcript.rs:589-599, 691, 948-949, 998-999, 1044-1046, 1104-1107,
   1281-1285, 1380-1382, 1434-1435) through the resolver so the exporter and
   the renderer literally share bytes, then lock with a
   `checked_in_bundle_matches_renderer_resolution`-style lib test.

## 5. Expected conflicts (record, don't collapse)

1. **`agentChat.error.bgAlphaUnits`** — `AgentChatErrorStyle.bg_alpha` is
   DECIMAL `50.0` while sibling alphas are hex-authored (`0x…`)
   (`agent_chat_catalog.rs:151`); high foot-gun risk when editing.
2. **`agentChat.search shell alphas`** — the composer reuses the main-view
   input shell which paints border/surface at InfoBarBase alphas `0x00`
   (`main_menu_theme.rs:739-740`): declared-but-ineffective, same idiom as
   the actions-dialog ineffective block.
3. **`agentChat.user.maxWidth` / `assistant.maxWidth` / `assistant.radius`**
   — declared in the style def but only applied in RoleSplit / when
   `bg_alpha > 0` (`transcript.rs:965-974,996-1001`).

The former composer-position, header-overflow, duplicate-profile/model,
font-size, and placeholder-line-height conflicts are closed for the embedded
default path. Painted and measured text share the main-menu search typography;
the composer model occupies the top MainViewInput slot; and the two header
lanes ellipsize without overlapping.

## 6. Agent Chat-owned tokens plus canonical composer aliases

Transcript (6): row-padding-x, row-padding-bottom, response-start-margin-top,
turn-margin-top, turn-padding-top, turn-divider.
Markdown (12): md-body-font-size, md-body-line-height (resolved),
md-paragraph-gap (rems source record), md-h1/h2/h3-font-size,
md-code-font-size/padding-x/padding-y/radius/bg/border
(+ blockquote padding-x/y/radius/bg/border if the exporter wants full
coverage: +5).
User (4): user-padding-x/y, user-radius, user-bg.
Assistant (2): assistant-padding-x/y.
Collapsible/tool (10): block-padding-x/y, block-body-padding-top,
block-max-body-height, block-border-width, block-header-gap,
block-header-opacity, block-status-opacity, tool-border, tool-border-error,
thought-border (11 with thought), tool-status-complete/failed/pending.
Diff (3): diff-added-bg, diff-removed-bg, diff-context-opacity.
System (4): system-padding-x/y, system-opacity, system-border.
Error (7, not yet consumed by the mockup): error-padding-x/y, error-radius,
error-bg, error-border, error-label-opacity, error-hint-opacity.
Composer (4 aliases, no new values): composer-font-size → main-menu search
font-size, composer-font-weight → main-menu search font-weight,
composer-line-height and composer-height → main-menu search height.
Send: send-size, send-radius, send-disabled-bg, send-disabled-opacity,
send-enabled-bg/opacity.
No-CSS-var records (writable:false): user/assistant max_width, assistant
radius, error bg-alpha units note, activity-row dot metrics, composer
placeholder strings ("Ask anything…", "Follow up…").

Agent Chat-owned values are currently PROPOSED in
`design/mockups/screens/agent-chat/screen.css` under the
`--sk-emulator-proposed-agent-chat-*` namespace. Composer typography/height
is different: the stable `--sk-agent-chat-composer-*` names directly alias
checked-in `--sk-main-menu-search-*` tokens, so no duplicate literals need
exporting.

## 7. Open unknowns

- Main-window height while the embedded chat is open (assumed 750×480 window
  tokens) — confirm from the capture receipt.
- `has_pastable_assistant_response` truth for fixture rows → Paste Response
  vs disabled Send in the native footer.
- `AGENT_CHAT_TRANSIENT_*_LANE_HEIGHT_PX` values (reserved lanes between the
  composer and transcript) — chase definitions before treating the
  header→transcript gap as blocking.
- Whether the background shader effect (Starfield) visibly animates behind
  the transcript in the capture (mockup omits the canvas).
- gpui-component TextView internals: exact list indent, heading margins,
  code-block copy-button geometry, rem base for `paragraph_gap`.
