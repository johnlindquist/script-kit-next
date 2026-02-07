# Arg Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-arg-prompt`  
Scope: `src/render_prompts/arg.rs`

## Executive Summary

`render_arg_prompt` is functional, but there are several UX gaps in text entry and result selection that make the prompt feel less predictable than the newer prompt types.

Top issues:

1. Enter/submit behavior is ambiguous and has weak validation feedback.
2. Filtering and auto-completion are basic (`contains`) and miss expected fuzzy/smart matching.
3. Input rendering does not keep the caret visible for long text.
4. Placeholder rendering relies on a fragile overlay pattern.
5. Input behavior is keyboard-only for editing (no mouse caret placement/selection path).

## Current Behavior Map

`src/render_prompts/arg.rs` currently handles:

1. Custom text rendering for cursor and selection (`src/render_prompts/arg.rs:58`).
2. Key routing for global shortcuts, actions dialog, list navigation, submit, and text edits (`src/render_prompts/arg.rs:166`).
3. Header input + placeholder UI (`src/render_prompts/arg.rs:437`).
4. Filtered list rendering and empty state (`src/render_prompts/arg.rs:369`).
5. Footer helper/info text + primary/secondary actions (`src/render_prompts/arg.rs:516`).

It depends on:

1. Arg filtering in `src/app_impl.rs:7134`.
2. Text editing primitives in `src/components/text_input.rs:313`.

## Findings (Ranked)

### P1: Submit/validation feedback is ambiguous and sometimes silent

Evidence:

1. Enter submits selected choice first, else raw text, else no-op (`src/render_prompts/arg.rs:267`).
2. Empty/no-op path has no user-visible feedback (`src/render_prompts/arg.rs:278`).
3. Empty filtered list message says no matches (`src/render_prompts/arg.rs:377`), but Enter can still submit raw text when input is non-empty (`src/render_prompts/arg.rs:273`).
4. Footer helper text does not reflect empty/no-match/typed-value states (`src/render_prompts/arg.rs:521`).

Impact:

1. Users cannot reliably predict what Enter will do.
2. Invalid submit attempts feel broken because there is no inline error/feedback.

Recommendation:

1. Introduce a shared `resolve_arg_submit_state()` used by both Enter and footer click.
2. Return explicit states: `SubmitChoice`, `SubmitText`, `InvalidEmpty`, `NoMatchNeedsConfirmation`.
3. Surface state in footer/list line (for example: "Type a value", "Press Enter again to use raw text", "No matches").

### P1: Filtering quality is weak for auto-completion/discovery

Evidence:

1. Filtering only matches `choice.name.to_lowercase().contains(query)` (`src/app_impl.rs:7139`).
2. `description` and `value` are ignored (`src/protocol/types.rs:127` has those fields).
3. No ranking or fuzzy scoring; results stay mostly source-order (`src/app_impl.rs:7140`).

Impact:

1. Harder to find relevant options quickly.
2. Auto-completion quality is noticeably behind `SelectPrompt` fuzzy scoring behavior.

Recommendation:

1. Reuse/port select scoring logic to arg prompt filtering so name/description/value participate.
2. Add deterministic ranking (exact-prefix > word-prefix > fuzzy).
3. Highlight matched ranges in list rows.

### P1: No true autocomplete acceptance flow (Tab/inline completion)

Evidence:

1. Arg key handler has no Tab branch (`src/render_prompts/arg.rs:166`).
2. `TextInputState::handle_key` does not handle Tab completions (`src/components/text_input.rs:325`).

Impact:

1. Users must arrow + Enter even for obvious single-match completion.
2. Slower keyboard workflow compared to typical launcher/search UX.

Recommendation:

1. Add Tab behavior:
   - Single match: complete to full choice label/value.
   - Multi match: cycle or accept current selection.
2. Add optional inline ghost completion suffix for selected match.

### P1: Long input can hide caret with no horizontal scroll behavior

Evidence:

1. Input row is clipped (`overflow_x_hidden`) (`src/render_prompts/arg.rs:124`).
2. Rendering inserts full `before/cursor/after` text but no scroll offset tracking (`src/render_prompts/arg.rs:116`).

Impact:

1. On long input, the active caret can move off-screen.
2. Editing near the end of long values becomes error-prone.

Recommendation:

1. Track input visual scroll offset (or right-anchor when cursor at end).
2. Keep caret visible after each edit/navigation event.
3. Add regression tests for long text editing.

### P2: Placeholder handling is fragile (negative-margin overlay)

Evidence:

1. Placeholder is rendered by overlapping cursor space with `ml(-CURSOR_WIDTH)` (`src/render_prompts/arg.rs:485`).
2. No explicit truncation/ellipsis behavior for long placeholders (`src/render_prompts/arg.rs:487`).

Impact:

1. Long placeholder text can crowd/clip unexpectedly.
2. Layout is harder to reason about and maintain.

Recommendation:

1. Replace overlap hack with a dedicated placeholder layer:
   - absolute inset, left padding equal to cursor gutter
   - explicit `overflow_x_hidden` + ellipsis
2. Keep cursor and placeholder visually independent.

### P2: Input editing is effectively keyboard-only (no mouse caret/selection path)

Evidence:

1. Input container renders text but has no input click handlers for caret placement (`src/render_prompts/arg.rs:447`).
2. Selection support exists in `TextInputState`, but arg prompt wiring is key-event driven (`src/render_prompts/arg.rs:292`).

Impact:

1. Users cannot click to move caret in the arg input.
2. Mouse-based correction workflows are blocked.

Recommendation:

1. Add click-to-focus + click-to-caret support for the input row.
2. Add drag-selection handling where feasible.

### P2: Submit behavior is duplicated in two paths

Evidence:

1. Enter key path has inline submission logic (`src/render_prompts/arg.rs:267`).
2. Footer primary button repeats near-identical logic (`src/render_prompts/arg.rs:542`).

Impact:

1. Easy for behavior to drift between keyboard and click submits.
2. Makes validation improvements riskier.

Recommendation:

1. Extract one `submit_arg_prompt_current_selection_or_text()` method.
2. Reuse it in both Enter and footer click handlers.

### P3: Filtering/render path still allocates heavily while typing

Evidence:

1. `get_filtered_arg_choices_owned()` clones each filtered `Choice` for render (`src/app_impl.rs:7152`).
2. Filtering lowercases each choice name repeatedly on every query (`src/app_impl.rs:7161`).

Impact:

1. Large choice sets can reduce typing responsiveness.
2. Adds avoidable work exactly where latency is user-visible.

Recommendation:

1. Store filtered indices (`Vec<usize>`) rather than cloning full choices.
2. Cache lowercased/searchable fields per choice.
3. Keep `uniform_list` but feed it index-backed immutable source data.

## Prioritized Roadmap

### Phase 1 (Correctness + UX clarity)

1. Add shared submit-state resolver and unify Enter/footer behavior.
2. Add explicit validation/helper feedback for empty/no-match/raw-text submission paths.
3. Add Tab-based completion.

### Phase 2 (Search quality + input ergonomics)

1. Upgrade arg filtering to fuzzy scored matching across name/description/value.
2. Implement caret-visible horizontal scrolling for long input.
3. Replace placeholder overlay with robust truncating placeholder layer.

### Phase 3 (Performance + polish)

1. Move arg filtering to index-based/cached model.
2. Add mouse caret placement/selection behavior.
3. Improve telemetry for submit/filter latency and state transitions.

## Suggested Tests (TDD Names)

1. `test_arg_prompt_shows_validation_feedback_when_enter_pressed_with_empty_input`
2. `test_arg_prompt_submits_selected_choice_when_choices_are_available`
3. `test_arg_prompt_submits_raw_text_when_no_choice_matches_non_empty_input`
4. `test_arg_prompt_applies_tab_autocomplete_for_single_match`
5. `test_arg_prompt_keeps_caret_visible_when_input_exceeds_available_width`
6. `test_arg_prompt_truncates_long_placeholder_without_cursor_overlap`
7. `test_arg_prompt_filter_scores_name_description_and_value`
8. `test_arg_prompt_enter_and_footer_primary_use_identical_submit_resolution`

## Risks / Known Gaps

1. Some behaviors (for example "always allow custom values when choices exist") may require explicit product decision.
2. Mouse-to-caret mapping needs text measurement support to be correct with variable-width fonts.
3. Fuzzy ranking changes can affect script author expectations that depend on current list order.
