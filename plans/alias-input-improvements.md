# Alias Input Improvements Audit

Date: 2026-02-07  
Agent: `codex-alias-input`  
Scope: `src/components/alias_input.rs`

## Executive Summary

`AliasInput` is usable for basic typing + Enter/Escape, but it has several gaps in the exact areas that matter for alias quality:

1. Validation is split across UI and save path, so users can hit Save and still fail later with a HUD-only error.
2. There is no autocomplete/suggestion system (no Tab acceptance, no command-based suggestions, no conflict-aware alternatives).
3. Keyboard interaction is modal-level only (Enter/Escape), with no focus traversal patterns for the button row and weak cross-platform modifier handling.
4. Placeholder/help text is static and repetitive instead of context-aware.
5. Long-input behavior and max-length ergonomics are under-specified (no visible character budget, no caret visibility strategy).

## Current Behavior Map

`src/components/alias_input.rs` currently provides:

1. Inline text rendering with cursor/selection (`src/components/alias_input.rs:279`).
2. Local validation for empty/whitespace/max length only (`src/components/alias_input.rs:116`).
3. Modal keyboard handling with Save on Enter and Cancel on Escape (`src/components/alias_input.rs:487`).
4. Conditional clear button (only when editing an existing alias) (`src/components/alias_input.rs:389`).
5. Validation message area below input (`src/components/alias_input.rs:471`).

End-to-end save path constraints live elsewhere:

1. Save enforces a stricter charset (alnum/`-`/`_`) in `app_impl` (`src/app_impl.rs:5250`).
2. Alias conflicts are resolved later during registry rebuild (first-registered wins) (`src/app_impl.rs:6763`).

## Findings (Ranked)

### P1: Validation contract is split between component and save path

Evidence:

1. Component validation allows any non-whitespace chars up to 32 (`src/components/alias_input.rs:122`, `src/components/alias_input.rs:126`).
2. Save path rejects characters outside `[A-Za-z0-9_-]` (`src/app_impl.rs:5250`).
3. Save button enablement is based on component validation only (`src/components/alias_input.rs:388`).

Impact:

1. Users can see Save enabled, press Enter, and still fail with a later HUD error.
2. Validation feedback is split between inline modal text and out-of-modal HUD messaging.

Recommendation:

1. Extract one shared alias validator used by both `AliasInput` and `save_alias_with_text`.
2. Extend `AliasValidationError` with a charset error variant to match persistence rules.
3. Keep Save disabled whenever shared validation fails.

### P1: No collision-aware validation before save

Evidence:

1. `save_alias_with_text` persists alias immediately after charset validation (`src/app_impl.rs:5270`).
2. Conflict policy is applied later during registry rebuild (`src/app_impl.rs:6763`).
3. Conflict handling is “first registered wins,” so saved alias may not become active (`src/app_impl.rs:6773`, `src/app_impl.rs:6862`).

Impact:

1. Alias can appear saved but still be unusable.
2. User gets late conflict feedback, disconnected from the modal action.

Recommendation:

1. Add pre-save conflict check against active alias registry and alias overrides.
2. Surface conflict inline in modal with “Already used by …”.
3. Offer conflict-safe suggestions as one-click/Tab alternatives.

### P1: Autocomplete is missing entirely (including Tab acceptance)

Evidence:

1. `AliasInput` has no suggestion model or candidate list fields.
2. Keydown handler has no `tab` branch (`src/components/alias_input.rs:501`).
3. `TextInputState::handle_key` has no tab completion behavior (`src/components/text_input.rs:325`).

Impact:

1. Users type from scratch and guess valid patterns.
2. Fast workflows (Tab to accept suggestion) are unavailable.

Recommendation:

1. Add suggestion providers:
   - command initialism (e.g. “Clipboard History” → `ch`),
   - slug prefix (e.g. `clipboard-history` → `clipboard`),
   - existing alias edit fallback.
2. Add `tab` to accept top suggestion; `shift+tab` cycles backward; arrows can cycle suggestion list.
3. Show ghost completion text in the input field when suggestion is available.

### P1: Keyboard interaction pattern is minimal and inconsistent with modal UX expectations

Evidence:

1. Only Escape and Enter are special-cased (`src/components/alias_input.rs:501`).
2. Clear action has no keyboard shortcut path (button-only) (`src/components/alias_input.rs:435`).
3. Key forwarding uses `mods.platform` but ignores `mods.control` (`src/components/alias_input.rs:519`), so non-mac ctrl shortcuts are not represented in this component.

Impact:

1. Keyboard-only users cannot trigger all modal actions efficiently.
2. Behavior is less predictable across platforms/input conventions.

Recommendation:

1. Add explicit modal keyboard map:
   - `Escape` cancel,
   - `Enter` save,
   - `Cmd+Backspace` (or platform equivalent) clear alias when editing,
   - `Tab`/`Shift+Tab` focus traversal between input and action buttons.
2. Normalize command modifier handling with platform-aware mapping (`platform || control` where appropriate).
3. Add visible focus ring/state for the currently focused control.

### P2: Placeholder UX is static and misses context

Evidence:

1. Placeholder string is hardcoded (`src/components/alias_input.rs:40`).
2. Header and helper copy repeat similar guidance (`src/components/alias_input.rs:408`, `src/components/alias_input.rs:41`).

Impact:

1. Placeholder does not leverage command context to suggest meaningful aliases.
2. Repetitive copy increases noise without helping completion.

Recommendation:

1. Make placeholder dynamic from command name (initialism + slug-ish option).
2. Keep one concise instructional line and use the helper area for validation/suggestion status.
3. Add truncated placeholder rendering for long command names.

### P2: Input behavior lacks explicit long-text/caret strategy and max-length affordance

Evidence:

1. Rendering splits text into before/after/selection but does not track horizontal viewport (`src/components/alias_input.rs:295`).
2. Max length is enforced only at validation time (`src/components/alias_input.rs:126`), not while typing.

Impact:

1. Long aliases can degrade editing clarity.
2. Users receive “too long” only after typing instead of getting proactive guardrails.

Recommendation:

1. Add a live `N/32` character counter.
2. Prevent insertion once max length is reached (except replacement/deletion paths).
3. Add a caret-visibility strategy (scroll offset or clipping policy) for long values.

### P3: Per-key logging is noisy and may capture user-entered alias characters

Evidence:

1. Every keydown logs key and modifiers (`src/components/alias_input.rs:492`).

Impact:

1. Excess log volume in active typing paths.
2. Potentially over-verbose capture of user-entered alias content.

Recommendation:

1. Move typing logs to debug level or sample them.
2. Keep structured events for save/cancel/clear/validation transitions only.

## Prioritized Roadmap

### Phase 1: Validation and keyboard correctness

1. Unify validator between modal and save pipeline.
2. Add inline conflict validation.
3. Add explicit keyboard map (`Tab`, clear shortcut, focus traversal).

### Phase 2: Autocomplete and placeholder UX

1. Add suggestion engine and ghost completion.
2. Implement Tab accept/cycle behavior.
3. Convert placeholder to command-aware suggestion text.

### Phase 3: Input polish and observability

1. Add max-length guardrails and counter.
2. Add long-input caret visibility strategy.
3. Reduce key-level logging noise; preserve high-value action logs.

## Suggested Tests (TDD Targets)

1. `test_alias_input_disables_save_when_chars_fail_shared_validator`
2. `test_alias_input_shows_inline_error_when_alias_conflicts_existing_registry`
3. `test_alias_input_accepts_top_suggestion_when_tab_pressed`
4. `test_alias_input_cycles_suggestions_when_shift_tab_pressed`
5. `test_alias_input_clears_existing_alias_when_platform_clear_shortcut_pressed`
6. `test_alias_input_moves_focus_between_controls_when_tab_navigation_used`
7. `test_alias_input_blocks_insert_when_max_length_reached_without_selection`
8. `test_alias_input_renders_dynamic_placeholder_based_on_command_name`

## Risks / Known Gaps

1. Suggestion ranking policy (initialism vs slug vs historical alias) needs product preference.
2. Conflict checks must account for case-insensitive matching to mirror registry behavior.
3. Cross-platform key semantics should be validated on at least one non-mac target if supported.
