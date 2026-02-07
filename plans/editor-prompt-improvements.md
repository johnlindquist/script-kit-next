# Editor Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-editor-prompt`  
Scope: `src/render_prompts/editor.rs`

## Executive Summary

`render_editor_prompt` is structurally sound, but it has several UX mismatches between what the footer says and how the editor actually behaves. The biggest issues are:

1. Submit guidance/shortcut labels are incorrect for this prompt type.
2. Parent-level SDK action shortcut interception can conflict with expected editor keybindings.
3. Footer status lacks cursor/selection/dirty-state visibility needed for efficient text editing.
4. Fixed wrapper height and passive syntax status reduce editor ergonomics.

## Responsibility Map (Current)

`src/render_prompts/editor.rs` currently does all of the following:

1. Synchronizes `EditorPrompt.suppress_keys` with actions popup visibility (`src/render_prompts/editor.rs:13`).
2. Installs a parent-level `on_key_down` handler for global/editor-actions routing (`src/render_prompts/editor.rs:38`).
3. Renders the editor container with explicit fixed height (`src/render_prompts/editor.rs:183`).
4. Builds a unified footer with helper text + language label (`src/render_prompts/editor.rs:206`).
5. Renders actions-dialog overlay and click-backdrop dismissal (`src/render_prompts/editor.rs:250`).

## Findings (Ranked)

### P1: Submit hint and shortcut label are wrong for EditorPrompt

Evidence:

1. Footer helper text says “press Enter” (`src/render_prompts/editor.rs:208`).
2. Shared footer config hardcodes primary shortcut `↵` (`src/render_prompts/arg.rs:40`).
3. Actual editor submission is `Cmd+Enter/Return` or `Cmd+S` (`src/editor.rs:1122`, `src/editor.rs:1127`).

Impact:

1. Users are told to use an input method that does not submit this prompt.
2. Increases failed-submit friction, especially for first-time editor prompt users.

Recommendation:

1. Use editor-specific footer config: primary shortcut `⌘↵`, secondary text `⌘S`.
2. Replace helper with explicit guidance: “Review input, then press ⌘↵ or ⌘S”.

### P1: Parent-level SDK action shortcuts can steal editor-native keybindings

Evidence:

1. Parent key handler checks `action_shortcuts` on every unhandled key (`src/render_prompts/editor.rs:116`).
2. Handler runs at wrapper level before the key fully falls through (`src/render_prompts/editor.rs:186`).
3. Editor `InputState` is configured as code editor with built-in editing/search behaviors (`src/editor.rs:330`, `src/editor.rs:331`).

Impact:

1. User-defined SDK action shortcuts can collide with editor commands (`Cmd+F`, undo/redo combos, etc.).
2. Collisions can break text editing expectations and selection/cursor workflows.

Recommendation:

1. Add editor guardrails: do not execute `action_shortcuts` for reserved editor combos.
2. Prefer explicit action namespace shortcuts (for example require `Cmd+Shift+...`) in editor context.
3. Add collision detection warning in logs when registering shortcut maps for editor prompts.

### P1: Editor footer keybinding discoverability is incomplete

Evidence:

1. Footer presents generic “Continue” with generic shortcut model (`src/render_prompts/editor.rs:210`).
2. Snippet helper covers only tabstop flow when active (`src/render_prompts/editor.rs:162`).

Impact:

1. Users do not discover submit alternatives (`Cmd+S`) or actions popup (`Cmd+K`) efficiently.
2. Increases trial-and-error and perceived editor inconsistency.

Recommendation:

1. Provide editor-specific footer labels:
   - primary: `Continue` + `⌘↵`
   - helper fallback: mention `⌘S` and `⌘K`
2. Keep snippet helper priority, but append concise static suffix for core shortcuts.

### P2: Missing dedicated key context for editor wrapper

Evidence:

1. Editor wrapper uses `.on_key_down(...)` but no `.key_context(...)` marker (`src/render_prompts/editor.rs:186`).

Impact:

1. Harder to scope key routing/debugging by prompt type.
2. Future keyboard refactors cannot reliably separate editor vs arg/form contexts.

Recommendation:

1. Add `.key_context("editor_prompt")` on the wrapper container.
2. Include key context in keyboard routing logs for grepability.

### P2: Fixed wrapper height can hurt editor ergonomics

Evidence:

1. Wrapper always uses `window_resize::layout::MAX_HEIGHT` (`src/render_prompts/editor.rs:36`).
2. Height is applied unconditionally (`src/render_prompts/editor.rs:183`).

Impact:

1. Editor prompt is less adaptive across small/large displays and dynamic UI scale.
2. Can waste vertical space or compress surrounding affordances depending on context.

Recommendation:

1. Use bounded dynamic height (based on active window visible bounds + config scale).
2. Keep `MAX_HEIGHT` as clamp cap, not fixed value.

### P2: Cursor/selection/edit-state telemetry is absent from footer

Evidence:

1. Footer currently receives snippet helper + language only (`src/render_prompts/editor.rs:210`).
2. Editor state integration exists and emits change/focus events (`src/editor.rs:339`).

Impact:

1. No quick visibility into line/column, selection length, or dirty state.
2. Slows precision editing and review workflows.

Recommendation:

1. Add an `editor.status_snapshot()` API and render:
   - `Ln x, Col y`
   - `Sel n` when active
   - `Modified` indicator
2. Update snapshot on `InputEvent::Change` and cursor move events.

### P2: `suppress_keys` synchronization is render-time mutation

Evidence:

1. `entity.update(... suppress_keys = show_actions)` happens during render (`src/render_prompts/editor.rs:15`).

Impact:

1. Works today, but mixes render with mutable state updates.
2. Can make input/focus regressions harder to reason about when popup state changes rapidly.

Recommendation:

1. Move to event-driven update when actions popup opens/closes.
2. Add structured log lines with `correlation_id` for state transitions.

### P3: Syntax highlighting status is passive and opaque

Evidence:

1. Footer only shows raw language label (`src/render_prompts/editor.rs:140`, `src/render_prompts/editor.rs:214`).
2. Highlighter setup/fallback is internal to `InputState` initialization (`src/editor.rs:330`, `src/editor.rs:420`).

Impact:

1. Users cannot tell whether language highlighting is active or fell back to plain text.
2. Troubleshooting language mismatch is slower.

Recommendation:

1. Surface highlight status badge (for example `TS`, `Plain Text`, `Fallback`).
2. Log highlighter selection result with typed fields and `correlation_id`.

### P3: Undo/redo behavior lacks explicit UX affordance

Evidence:

1. Wrapper and footer do not expose undo/redo hints or action entries (`src/render_prompts/editor.rs`).
2. Editor relies on underlying code editor defaults (`src/editor.rs:330`).

Impact:

1. Users may not discover available undo/redo patterns in this prompt context.
2. Increases mismatch vs historical Script Kit editor muscle memory.

Recommendation:

1. Add optional footer hint or actions entries for undo/redo shortcuts.
2. Add regression tests that ensure action shortcut interception never blocks undo/redo.

## Improvement Plan

### Phase 1 (High impact / low risk)

1. Fix footer submit hint + shortcut labels for editor prompt.
2. Add reserved-shortcut exclusions before triggering SDK action shortcuts.
3. Add editor key context and update logs with context + `correlation_id`.

### Phase 2 (Editing ergonomics)

1. Add footer status snapshot (line/column/selection/dirty).
2. Make editor wrapper height adaptive with clamp-based bounds.

### Phase 3 (Polish and observability)

1. Expose syntax highlighter status/fallback in UI and logs.
2. Add explicit undo/redo discoverability and conflict tests.

## Suggested Tests (TDD Names)

1. `test_editor_footer_displays_cmd_enter_and_cmd_s_submit_hints`
2. `test_editor_action_shortcuts_do_not_override_reserved_editing_bindings`
3. `test_editor_prompt_shortcut_collision_logs_warning_with_correlation_id`
4. `test_editor_footer_shows_cursor_and_selection_status_when_selection_changes`
5. `test_editor_wrapper_uses_bounded_dynamic_height_when_window_resizes`
6. `test_editor_footer_shows_highlighter_fallback_when_language_unavailable`

## Risks / Known Gaps

1. Shortcut reservation policy needs clear product definition (which combos are always editor-owned).
2. Cursor/selection status requires API surface from `EditorPrompt`/`InputState` that is not currently exposed.
3. Dynamic sizing must be validated against multi-monitor bounds and existing `window_resize` constraints.
