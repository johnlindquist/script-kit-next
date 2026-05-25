# Inline Agent Goal

Status: planned from three Oracle/Packx tracks on 2026-05-24.

Oracle source sessions:

- Track 1, macOS accessibility/text mutation: `~/.oracle/sessions/inline-agent-accessibil-2/output.log`
- Track 2, compact/expanded overlay UI: `~/.oracle/sessions/inline-agent-overlay-ui/output.log`
- Track 3, AI execution/threading: `~/.oracle/sessions/inline-agent-ai-chat/output.log`

## Product Target

Build a system-wide AI text-editing utility inside Script Kit GPUI. A global trigger captures the full text of the currently focused text field in any macOS app, opens a compact anchored overlay, sends the captured text plus the user's instruction to the active AI agent, streams a result, and lets the user Replace, Append, Copy, or expand into a multi-turn chat panel.

The feature must not depend on selected text. Whole-field capture is the core invariant.

## Architecture

Ship this as three cooperating layers with strict ownership boundaries.

1. `src/platform/accessibility/` owns macOS truth:
   - Accessibility permission checks.
   - `AXFocusedUIElement` discovery.
   - Whole focused-field text capture.
   - Active app identity.
   - Caret/field/window anchor geometry.
   - Replace/Append/Copy mutation.
   - Clipboard-safe paste fallback.
   - Double-Command trigger support.

2. `src/inline_agent/` owns the GPUI overlay:
   - Compact anchored overlay.
   - Prompt input and "Thinking..." state.
   - Output preview and action strip.
   - Expanded chat panel projection.
   - Layout, theme/contrast, automation snapshots, and DevTools-visible IDs.
   - No raw AX calls and no provider internals.

3. `src/ai/inline_agent/` owns AI execution:
   - Prompt assembly from focused-field capture plus user instruction.
   - ACP-backed streaming through `AcpConnection::start_turn`.
   - Cancellation, retry, turn state, and ephemeral history.
   - Latest-output action resolution.
   - Privacy/audit contracts.

Do not mount `AcpChatView` wholesale in the inline overlay. Reuse or extract transcript/composer components where useful, but keep inline editing state small and text-edit-specific.

## Track 1: Platform Accessibility

### Owner Files To Reuse

- `src/selected_text.rs`
  - Reuse `has_accessibility_permission`, `request_accessibility_permission`, `open_accessibility_settings`, `show_permission_dialog`.
  - Reuse `simulate_paste_with_cg`.
  - Extract full `PasteboardSnapshot`, `write_plain_text_to_pasteboard`, and `general_pasteboard_change_count` into a shared pasteboard module.
  - Do not use `get_selected_text()` for this feature; it is selected-text-only and may fall back to Cmd+C.

- `src/text_injector.rs`
  - Reuse timing/key simulation ideas only.
  - Do not reuse its text-only clipboard restore for Replace/Append.

- `src/platform/visibility_focus.rs`
  - Capture focused text before showing or making any Script Kit window key.
  - Use conceal/hide patterns before paste fallback so Cmd+V goes to the target app.

- `src/platform/ai_commands.rs`
  - Reuse frontmost-app metadata patterns.

- `src/hotkeys/mod.rs`
  - Add a normal configurable inline-edit action and channel here.
  - Implement double-Command separately through a passive macOS event tap.

- `src/main_entry/runtime_stdin*.rs`
  - Add protocol verbs for focused text capture and mutation using existing typed-response and source-audit patterns.

### New Modules

Create:

- `src/platform/accessibility/mod.rs`
- `src/platform/accessibility/permissions.rs`
- `src/platform/accessibility/ax.rs`
- `src/platform/accessibility/app_identity.rs`
- `src/platform/accessibility/focused_text.rs`
- `src/platform/accessibility/geometry.rs`
- `src/platform/accessibility/mutation.rs`
- `src/platform/accessibility/clipboard.rs`
- `src/platform/accessibility/double_modifier_trigger.rs`
- `src/platform/accessibility/metrics.rs`

Core API:

```rust
pub fn capture_focused_text_field(
    options: CaptureFocusedTextOptions,
) -> Result<FocusedTextSnapshot, FocusedTextError>;

pub fn replace_focused_text(
    session_id: FocusedTextSessionId,
    text: &str,
    options: TextMutationOptions,
) -> Result<TextMutationResult, FocusedTextError>;

pub fn append_focused_text(
    session_id: FocusedTextSessionId,
    text: &str,
    options: TextMutationOptions,
) -> Result<TextMutationResult, FocusedTextError>;

pub fn copy_text_output(text: &str) -> Result<TextMutationResult, FocusedTextError>;
```

Core DTOs:

```rust
pub struct FocusedTextSnapshot {
    pub session_id: FocusedTextSessionId,
    pub captured_at_ms: u128,
    pub app: ActiveAppIdentity,
    pub target: FocusedTextTargetDescriptor,
    pub text: String,
    pub selected_range_utf16: Option<TextRangeUtf16>,
    pub caret_range_utf16: Option<TextRangeUtf16>,
    pub metrics: TextMetrics,
    pub geometry: FocusedFieldGeometry,
    pub capabilities: FocusedTextCapabilities,
}
```

Keep raw `AXUIElementRef` values in a process-local session registry with a short TTL. Track 2 and Track 3 receive `session_id`, not raw AX handles.

### Capture Algorithm

1. Check Accessibility permission. If missing, fail with `AccessibilityPermissionRequired`; do not attempt clipboard fallback.
2. Read frontmost app identity from `NSWorkspace` and existing frontmost tracker when available.
3. Resolve `AXFocusedUIElement` from system-wide AX, with a frontmost-app fallback.
4. Read role/subrole/title lengths only.
5. Reject secure fields early.
6. Read whole text:
   - Primary: `AXValue` as `CFString`.
   - Fallback: `AXNumberOfCharacters` plus `AXStringForRange` over the full range.
   - Never return selected text as a fake whole-field success.
7. Read `AXSelectedTextRange`.
8. Resolve anchor geometry:
   - Caret `AXBoundsForRange`.
   - Selection `AXBoundsForRange`.
   - Focused field `AXPosition` + `AXSize`.
   - Focused window bounds.
   - Mouse/display fallback.
9. Compute metrics: bytes, chars, UTF-16 units, lines, cheap token estimate.
10. Store a focused-text session and return the snapshot.

### Mutation Algorithm

Copy:

- Write output to clipboard.
- Do not snapshot/restore; Copy is intentionally a clipboard operation.

Replace:

- Validate session/target unless `allow_stale` is explicitly enabled.
- Prefer direct `AXValue` set if settable.
- Set caret to output end when possible.
- Verify by rereading `AXValue` when possible.
- Fallback:
  - Refocus target app/element.
  - Select full field via `AXSelectedTextRange`.
  - Paste via extracted full pasteboard snapshot helper.
  - Restore clipboard only if pasteboard `changeCount` still matches.
  - Never paste into an unverified target.

Append:

- Prefer direct `AXValue` set with `current + output`.
- Fallback:
  - If possible, set caret to current UTF-16 end and paste only output.
  - If caret set fails, select all and paste the full appended value.
  - Avoid app-specific "Cmd+End then paste" behavior.

### Trigger

Add `HotkeyAction::InlineAiTextEdit` and an inline AI hotkey channel in `src/hotkeys/mod.rs`.

For double Command:

- Add `double_modifier_trigger.rs`.
- Use a passive/listen-only `CGEventTap` for `kCGEventFlagsChanged`.
- Do not swallow events.
- Reset on non-modifier keys or combined shortcuts like Cmd+C/Cmd+Tab.
- Do not perform AX work inside the event-tap callback; dispatch a channel event.
- Re-enable tap if macOS disables it.
- Keep a normal configurable shortcut fallback.

### Platform Tests

Add source audits:

- `tests/source_audits/focused_text_capture_ax_wired.rs`
- `tests/source_audits/focused_text_geometry_ax_wired.rs`
- `tests/source_audits/focused_text_mutation_wired.rs`
- `tests/source_audits/inline_ai_double_command_trigger_wired.rs`
- `tests/source_audits/stdin_focused_text_protocol_wired.rs`

Add unit tests for:

- UTF-16 metrics with emoji.
- Anchor fallback order.
- Stale mutation rejection.
- Append using current readable value.
- Double-Command state machine, including Cmd+C not triggering.

Add smoke tests:

- TextEdit capture.
- TextEdit replace.
- TextEdit append.
- Copy output.
- Manual/native double-Command proof.

## Track 2: Overlay UI

### Owner Files To Reuse

- `src/components/inline_popup_window.rs`
  - Reuse popup bounds, popup options, NSWindow extraction, and bounds update patterns.
  - Add/split support for standalone external-app overlays, because this overlay usually has no Script Kit parent window.

- `src/platform/secondary_window_config.rs`
  - Reuse non-stealing popup/AppKit configuration and `setBecomesKeyOnlyIfNeeded` policy.

- `src/components/inline_prompt_input.rs`
  - Reuse for compact prompt input.

- `src/app_impl/menu_syntax_trigger_popup_window.rs`
  - Reuse lifecycle pattern: singleton slot, sync/update/close, automation registration, bounds updates.

- `src/platform/vibrancy_config.rs`
  - Reuse dark/light vibrancy policy to avoid dim unreadable text.

- `tests/source_audits/mini_ai_window.rs`
  - Use as source-audit style for machine-readable state, no-fork checks, mode transitions, IDs, and telemetry.

Avoid:

- `src/components/overlay_modal.rs` for this feature.
- Main-window resize paths like `update_window_size_deferred` or `resize_to_view_sync`.
- Any UI code importing AX internals.

### New Modules

Create:

- `src/inline_agent/mod.rs`
- `src/inline_agent/types.rs`
- `src/inline_agent/state.rs`
- `src/inline_agent/layout.rs`
- `src/inline_agent/window.rs`
- `src/inline_agent/render_compact.rs`
- `src/inline_agent/render_expanded.rs`
- `src/inline_agent/render_actions.rs`
- `src/inline_agent/telemetry.rs`
- `src/inline_agent/automation.rs`
- `src/inline_agent/platform_bridge.rs`

The UI bridge consumes Track 1 without knowing AX:

```rust
pub(crate) trait InlineAgentPlatformBridge {
    fn capture_focused_text_snapshot(&self) -> anyhow::Result<InlineAgentSnapshot>;
    fn apply_text_mutation(
        &self,
        anchor: &InlineAgentAnchor,
        mutation: InlineAgentTextMutation,
    ) -> anyhow::Result<InlineAgentMutationReceipt>;
}
```

### State Machine

Use a pure state machine:

```rust
pub(crate) enum InlineAgentMode {
    Compact,
    Expanded,
}

pub(crate) enum InlineAgentRunState {
    Idle,
    Thinking { request_id: String, started_at_ms: u64 },
    Streaming { request_id: String, partial_output: String },
    Completed { output: String },
    Error { message: String, retryable: bool },
    Applying { action: InlineAgentOutputAction },
    Applied { action: InlineAgentOutputAction },
}
```

Transitions:

- Hidden -> capture snapshot -> Compact/Idle.
- Submit -> Compact/Thinking.
- First model delta -> Compact/Streaming.
- Finish -> Compact/Completed.
- Replace/Append -> Applying -> Applied or Error.
- Copy -> Completed with copied receipt.
- Chat -> Expanded using the same session.
- Collapse -> Compact preserving latest output.
- Escape collapses expanded first, then closes compact.

### Window Lifecycle

Capture must happen before opening or focusing the overlay.

Open sequence:

1. Global trigger fires.
2. Track 1 captures focused text and anchor.
3. Track 2 opens overlay near the anchor.
4. Overlay focuses its own prompt input.
5. Later Replace/Append asks Track 1 to refocus/mutate target.

Do not open the overlay first and then capture. That would steal the focused UI element.

Add `InlineOverlayAttachment::{Standalone, AttachedToParent}`. Use `Standalone` for external-app anchored overlays.

### Layout

Compact defaults:

- Width: 420 px, clamped 320 to 560.
- Idle height: about 118 px.
- Thinking height: about 144 px.
- Completed/output height: about 252 px.
- Edge gutter: 12 px.
- Anchor gap: 8 px.

Expanded defaults:

- Width: about 680 px, clamped 560 to 760.
- Height: about 560 px, capped to visible display.

Placement:

- Prefer below caret/field.
- Flip above near bottom edge.
- Clamp to visible display bounds.
- Preserve multi-monitor coordinates.

### Compact UI Requirements

Compact overlay content:

- Header with app badge, e.g. `Slack`.
- Metrics: characters and estimated tokens.
- Optional support chip: editable, copy-only, unsupported, stale.
- Input placeholder exactly: `Edit, refine, ask...`
- Thinking status bar with visible `Thinking...`.
- Inline output preview after completion.
- Action strip: Replace, Append, Copy, Chat.

Action behavior:

- Replace disabled with no output or unsupported mutation.
- Append disabled with no output or unsupported append.
- Copy enabled with output even for unsupported mutation.
- Chat disabled only while an action is applying.

### Expanded UI Requirements

Chat expands the same overlay and session. It is not a new main-window surface.

Expanded view:

- Header label like `Cue - 1 turn`.
- Chronological user/model blocks.
- Distinct user and assistant styling.
- Persistent bottom composer.
- Replace/Append/Copy actions against the latest complete output.
- Collapse button returning to compact without losing output.

### Theme and Contrast

Add `InlineAgentColors::from_theme`.

Rules:

- Primary text must meet at least 4.5:1 contrast against overlay surface.
- Secondary text must remain readable; do not rely on low opacity over blurred vibrancy.
- Disabled text must still meet about 3:1.
- Thinking status uses a high-contrast active color.
- For vibrancy, follow existing dark/light material policy in `vibrancy_config.rs`.

### UI Tests and Proof

Add:

- `tests/source_audits/inline_agent_overlay.rs`
- `tests/inline_agent_state_contract.rs`
- `tests/inline_agent_layout_contract.rs`
- `tests/inline_agent_actions_contract.rs`
- `tests/inline_agent_theme_contract.rs`
- `tests/smoke/test-inline-agent-overlay.ts`
- `tests/smoke/test-inline-agent-vibrancy.ts`

Pin stable IDs:

- `inline-agent-compact`
- `inline-agent-header`
- `inline-agent-app-badge`
- `inline-agent-metrics`
- `inline-agent-input`
- `inline-agent-thinking-bar`
- `inline-agent-thinking-label`
- `inline-agent-output-preview`
- `inline-agent-action-replace`
- `inline-agent-action-append`
- `inline-agent-action-copy`
- `inline-agent-action-chat`
- `inline-agent-expanded`
- `inline-agent-turn-list`
- `inline-agent-expanded-composer`
- `inline-agent-collapse`

Runtime proof should start with mock snapshots, then move to native focused text after Track 1 is stable.

## Track 3: AI Execution and Threading

### Owner Files To Reuse

- `src/ai/acp/client.rs`
  - Reuse `AcpRuntime`, `AcpConnection`, `prepare_session`, `start_turn`, `cancel_turn`.
  - Avoid `stream_prompt`; Oracle identified it as the legacy callback bridge.

- `src/ai/acp/events.rs`
  - Reuse `AcpPromptTurnRequest`, `AcpEvent`, and `AcpCommand::StartTurn`.

- `src/ai/acp/thread.rs`
  - Reuse state patterns and event binding ideas.
  - Do not use `AcpThread` directly without abstraction; it clears staged context after first submit, while inline-agent refinements must keep original captured text available on every turn.

- `src/ai/message_parts.rs`
  - Reuse context receipt patterns and safe structured context ideas.

- `src/app_impl/tab_ai_mode/acp_launch.rs`
  - Reuse the UX invariant: show user-visible surface before deferred work blocks.

- `src/ai/acp/components/transcript.rs`
  - Extract/reuse transcript rendering pieces for expanded chat.

- `src/ai/acp/view.rs`
  - Extract/reuse bottom composer/action patterns.
  - Do not mount the entire view.

Avoid deprecated legacy AI window/provider paths for new execution.

### New Modules

Create:

- `src/ai/inline_agent/mod.rs`
- `src/ai/inline_agent/types.rs`
- `src/ai/inline_agent/prompt.rs`
- `src/ai/inline_agent/session.rs`
- `src/ai/inline_agent/executor.rs`
- `src/ai/inline_agent/actions.rs`
- `src/ai/inline_agent/privacy.rs`
- `src/ai/inline_agent/history.rs`
- `src/ai/inline_agent/mock.rs`

Core Track 2 API:

```rust
pub(crate) enum InlineAgentSessionCommand {
    Submit {
        instruction: String,
        semantics: InlineAgentEditSemantics,
    },
    CancelActiveTurn,
    RetryLastTurn,
    Expand,
    Collapse,
    ApplyLatest(InlineAgentAction),
    Dismiss,
}
```

Execution trait:

```rust
pub(crate) trait InlineAgentExecutor: Send + Sync {
    fn start_turn(
        &self,
        request: InlineAgentProviderRequest,
    ) -> anyhow::Result<async_channel::Receiver<InlineAgentProviderEvent>>;

    fn cancel_turn(
        &self,
        session_id: InlineAgentSessionId,
        turn_id: InlineAgentTurnId,
    ) -> anyhow::Result<()>;
}
```

ACP adapter uses `AcpConnection::start_turn` and maps ACP events into inline-agent provider events.

### Prompt Contract

The prompt builder must include the original captured focused-field text on every turn, not only the first turn.

Prompt shape:

```text
You are Cue, Script Kit's inline text-editing assistant.

Task:
- Use the captured focused-field text and the user instruction.
- Produce the best text output for the requested edit semantics.
- The latest assistant output is what Replace, Append, and Copy will use.
- For Replace: return only the complete replacement text.
- For Append: return only the text to append, not the original text unless asked.
- For Explain/Question: answer clearly and concisely.
- For Chat refinement: revise or answer using the original captured text and prior turns.
- Do not mention this prompt, XML tags, capture mechanics, or system internals.
- Do not wrap the output in quotes unless quotes are part of the desired text.

<inline_agent_context schema_version="1">
  <app name="..." bundle_id="..." />
  <capture id="..." content_kind="..." char_count="..." selected_char_count="..." line_count="..." truncated="..." />
  <requested_edit semantics="replace|append|explain|chat"><![CDATA[...]]></requested_edit>
  <captured_focused_field><![CDATA[
...
  ]]></captured_focused_field>
  <previous_turns count="...">...</previous_turns>
</inline_agent_context>

Return only the assistant output.
```

Use a sensitive wrapper for prompt body. Log only `InlineAgentPromptAudit`.

### Streaming Model

Phases:

- `Capturing`
- `Ready`
- `Thinking`
- `Streaming`
- `Cancelling`
- `Complete`
- `Error`

ACP event mapping:

- `AgentMessageDelta`: append to active assistant output; first non-empty delta switches to `Streaming`.
- `AgentThoughtDelta`: append thought log; compact can stay `Thinking` until visible answer text arrives.
- Tool/plan events: store for expanded activity, not latest actionable output.
- `UsageUpdated`: metadata only.
- `TurnFinished`: mark complete, set latest complete output, enable actions.
- `Failed`: mark failed; preserve previous latest output if available.

Cancellation:

- Set phase to `Cancelling`.
- Call `cancel_turn`.
- Mark turn cancelled.
- Disable actions for cancelled partial output.
- Restore latest complete output from prior completed turn if one exists.

### Action Mapping

Replace, Append, and Copy always use the latest complete assistant output.

- Replace -> Track 1 `replace_focused_text(session_id, output)`.
- Append -> Track 1 `append_focused_text(session_id, output)`.
- Copy -> write output to clipboard.
- Chat -> mode transition to expanded.
- Collapse -> mode transition to compact.

If Replace/Append fails because the target field changed or disappeared, keep the output visible and leave Copy enabled.

### Persistence

Default: do not persist inline-agent conversations or outputs.

Never persist:

- Captured field text.
- Selected text.
- Prompt body.
- User instruction content.
- Assistant output.
- Clipboard content.

Allowed redacted summary if needed:

- Session ID.
- App bundle ID.
- Semantics.
- Turn count.
- Capture character count.
- Completion status.

### AI Tests and Proof

Add:

- `tests/inline_agent_prompt_contract.rs`
- `tests/inline_agent_stream_state_contract.rs`
- `tests/inline_agent_actions_contract.rs`
- `tests/inline_agent_expanded_mode_contract.rs`
- `tests/inline_agent_privacy_contract.rs`
- `tests/inline_agent_acp_adapter_contract.rs`

Existing tests to keep green:

- `tests/acp_thread_replay_generation_contract.rs`
- `tests/acp_composer_state_machine_contract.rs`
- `tests/acp_cancel_midstream_contract.rs`
- `tests/acp_surface_state_contract.rs`
- `tests/mini_ai_actions_contract.rs`

Mocked runtime proof:

1. Simulate focused-field capture with `Hello world`.
2. Submit instruction `Translate to French`.
3. Wait for `inlineAgent.phase == "thinking"`.
4. Mock stream emits `Bonjour le monde`.
5. Wait for latest output preview.
6. Click Chat.
7. Assert expanded mode and label `Cue - 1 turn`.
8. Submit `make it more formal`.
9. Assert prompt audit `previousTurnCount == 1`.
10. Click Replace.
11. Assert Track 1 apply receipt used latest output.

## Implementation Sequence

### Phase 1: Focused Capture Contract

Implement the first shippable platform slice:

- Extract representation-preserving pasteboard helpers.
- Add AX wrapper.
- Add focused-field capture for TextEdit/AppKit text fields.
- Add metrics and anchor fallback.
- Add source-audit tests proving whole-field AX capture, not selected-text fallback.
- Add TextEdit read-only smoke.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test source_audits focused_text
./scripts/agentic/agent-cargo.sh check --lib
```

### Phase 2: Overlay State/Layout With Mock Snapshot

- Add pure inline-agent UI state.
- Add layout tests for below/above/clamp/secondary-display placement.
- Add action availability tests.
- Add compact overlay with mock snapshot and stable automation IDs.
- Add theme contrast tests.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test inline_agent_state_contract
./scripts/agentic/agent-cargo.sh test --test inline_agent_layout_contract
./scripts/agentic/agent-cargo.sh test --test inline_agent_actions_contract
./scripts/agentic/agent-cargo.sh test --test inline_agent_theme_contract
```

### Phase 3: AI Session Skeleton

- Add `src/ai/inline_agent`.
- Add prompt builder, session reducer, mock executor, privacy audit.
- Include original captured text in every turn.
- Add mocked streaming tests.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test inline_agent_prompt_contract
./scripts/agentic/agent-cargo.sh test --test inline_agent_stream_state_contract
./scripts/agentic/agent-cargo.sh test --test inline_agent_privacy_contract
```

### Phase 4: ACP Adapter

- Implement `AcpInlineAgentExecutor`.
- Use `AcpConnection::start_turn` and `cancel_turn`.
- Map ACP events to inline-agent events.
- Keep UI render files provider-agnostic.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test inline_agent_acp_adapter_contract
./scripts/agentic/agent-cargo.sh test --test acp_cancel_midstream_contract
```

### Phase 5: Mutation

- Implement direct AX Replace/Append.
- Implement clipboard-safe fallback.
- Add stale target guard.
- Wire Replace/Append/Copy protocol and UI actions.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test source_audits focused_text_mutation
./scripts/agentic/agent-cargo.sh check --lib
```

Runtime:

- TextEdit replace smoke.
- TextEdit append smoke.
- Copy output smoke.

### Phase 6: Expanded Chat

- Extract shared transcript/composer pieces from ACP where needed.
- Render expanded inline-agent panel from the same session.
- Support follow-up turns, Collapse, Stop, Retry, and latest-output actions.
- Add `Cue - N turns` label contract.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test inline_agent_expanded_mode_contract
./scripts/agentic/agent-cargo.sh test --test mini_ai_actions_contract
```

### Phase 7: Trigger

- Add normal configurable inline-edit hotkey.
- Add double-Command event tap.
- Ensure trigger captures before overlay open.
- Add manual/native proof.

Verification:

```bash
./scripts/agentic/agent-cargo.sh test --test source_audits inline_ai_double_command
```

Native proof must verify actual Command double-press delivery, not only a simulated command.

### Phase 8: End-To-End Runtime Proof

Use DevTools/state-first proofs first, screenshots only for visual requirements, and native input for OS delivery.

Required receipts:

- Compact idle overlay anchored near mock/text field.
- Compact thinking overlay with `Thinking...`.
- Compact completed output with actions.
- Unsupported/read-only state with Copy still enabled.
- Expanded chat with user/model blocks and persistent composer.
- Collapse back to compact preserving output.
- Dark and light contrast screenshots.
- TextEdit capture/replace/append.
- Browser textarea capture/replace/append when Track 1 supports it.
- Double-Command native trigger.

## Privacy and Logging Contract

Allowed logs:

- Correlation/request/session IDs.
- App pid/bundle ID.
- Role/subrole.
- Lengths and metrics.
- Anchor source.
- Strategy/method names.
- Error codes.
- Latency.

Forbidden logs:

- Captured field text.
- Selected text.
- User instruction content.
- Prompt body.
- Assistant output.
- Clipboard content.
- Raw field title/description text unless explicitly privacy-gated.

Add source audits that fail on raw logging patterns in `platform/accessibility`, `inline_agent`, and `ai/inline_agent`.

## Definition Of Done

- Whole focused-field capture works through AX and never fakes success with selected text.
- Compact overlay anchors to caret/field/window fallback and shows app badge plus metrics.
- Compact prompt placeholder is `Edit, refine, ask...`.
- `Thinking...` appears during processing.
- AI prompt includes captured field text and user instruction.
- Streaming updates compact/expanded state.
- Replace replaces the whole field.
- Append appends to the end.
- Copy writes the output to clipboard.
- Chat expands the same session into a larger multi-turn panel.
- Expanded panel shows `Cue - N turns`, user/model blocks, persistent composer, Replace/Append/Copy, and Collapse.
- Conversations remain ephemeral by default.
- Dark/light/vibrancy contrast is verified.
- Protocol/DevTools can inspect state/elements/bounds.
- Targeted source audits, unit tests, runtime proofs, screenshots, and native trigger proof are recorded.
