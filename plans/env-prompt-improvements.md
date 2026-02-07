# Env Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-env-prompt`  
Scope: `src/prompts/env.rs`, `src/render_prompts/**/*.rs`

## Executive Summary

The env prompt is visually polished, but there are correctness and UX gaps across editing, masking, validation, and persistence.

Top issues:

1. Existing secrets are silently auto-submitted in script-driven env flows, so users cannot recover from stale credentials in-context.
2. Secret masking rendering does not preserve cursor/selection behavior and uses byte length for bullet count.
3. Validation is minimal (non-empty only) and persistence failures are logged but not surfaced to users.
4. Storage/persistence messaging is inconsistent with actual behavior and backend implementation.
5. Protocol fields for env are too limited to support robust validation/persistence UX.

## Current Behavior Map

`src/prompts/env.rs` currently handles:

1. Env prompt layout, key handling, and footer actions (`src/prompts/env.rs:309`).
2. Optional auto-submit from stored secret (`src/prompts/env.rs:171`).
3. Secret persistence via encrypted store hooks (`src/prompts/env.rs:195`).
4. Existing-secret metadata display and delete action (`src/prompts/env.rs:565`).

`src/render_prompts/other.rs` contributes:

1. Prompt shell wrapper and global shortcut interception for env prompt (`src/render_prompts/other.rs:57`).

Upstream dependencies shaping behavior:

1. Env message shape (`src/protocol/message.rs:267`).
2. Message mapping into `PromptMessage::ShowEnv` (`src/execute_script.rs:1284`).
3. Script-driven auto-submit decision (`src/prompt_handler.rs:1287`).
4. SDK secret heuristic (`scripts/kit-sdk.ts:4625`).
5. Actual persistence backend (`src/secrets.rs:1`).

## Findings (Ranked)

### P0: Silent auto-submit blocks stale-credential recovery in script env flow

Evidence:

1. `PromptMessage::ShowEnv` path checks key store and returns early when found (`src/prompt_handler.rs:1287`).
2. `check_keyring_and_auto_submit()` immediately calls submit callback with stored value (`src/prompts/env.rs:179`).

Impact:

1. Users cannot edit/replace stored values from `env()` script flow when credentials expire.
2. Scripts can repeatedly receive bad saved values with no interactive remediation path.

Recommendation:

1. Add `auto_submit_existing: bool` (default `true`) to env protocol/config.
2. Add explicit force-refresh path (for example, SDK `env(key, { forcePrompt: true })`).
3. Log an explicit state transition when auto-submit fires to improve debuggability.

### P0: SDK secret heuristic over-classifies keys and forces masking/persistence unexpectedly

Evidence:

1. SDK marks secret true when key name contains `"key"` or similar substrings (`scripts/kit-sdk.ts:4625`).
2. Protocol message only carries `secret` bool without persistence policy distinction (`src/protocol/message.rs:267`).

Impact:

1. Non-secret variables with incidental names can be treated as secrets.
2. User experience becomes unpredictable (masked input + encrypted persistence when not intended).

Recommendation:

1. Replace heuristic-first behavior with explicit env options (`secret`, `persist`).
2. Keep heuristic as fallback only when options are absent.
3. Add a compatibility warning log when heuristic infers secret mode.

### P1: Secret masking UI does not reflect real cursor/selection state

Evidence:

1. Secret rendering path always appends cursor at the end of dots (`src/prompts/env.rs:514`).
2. Selection/caret data from `TextInputState` is ignored in that branch (`src/prompts/env.rs:514`).
3. Bullet count uses `self.input.text().len()` (bytes), not char count (`src/prompts/env.rs:516`).

Impact:

1. Keyboard editing behavior (left/right/select/delete) can diverge from what user sees.
2. Non-ASCII secrets can render wrong mask length.

Recommendation:

1. Use one rendering pipeline for secret and non-secret text with real selection/cursor state.
2. Use character count (`chars().count()`) for mask generation.
3. Add optional reveal toggle (press-and-hold or explicit eye button) with safe default masked state.

### P1: Validation and persistence error handling are mostly silent

Evidence:

1. Submit only guards on `!text.is_empty()`; empty submit is silent no-op (`src/prompts/env.rs:194`).
2. `set_secret` failure is logged, but callback still returns success value (`src/prompts/env.rs:197`).
3. Delete failure is logged, but prompt closes as if operation succeeded (`src/prompts/env.rs:228`).

Impact:

1. Users get no actionable feedback for empty or invalid input.
2. Scripts may proceed with values that were not persisted, causing future inconsistencies.

Recommendation:

1. Introduce explicit validation state and inline feedback row.
2. Gate success callback on persistence success when persistence is required.
3. Show toast/HUD for storage and delete failures with retry guidance.

### P1: Persistence copy and storage semantics are inconsistent

Evidence:

1. Env prompt docs/hints mention keyring/keychain (`src/prompts/env.rs:1`, `src/prompts/env.rs:545`).
2. Actual backend is encrypted file store `~/.scriptkit/secrets.age` (`src/secrets.rs:1`).
3. Only `secret=true` values are persisted (`src/prompts/env.rs:196`).

Impact:

1. User-facing messaging can mislead about where/how values are stored.
2. Non-secret env prompts imply persistence that does not occur.

Recommendation:

1. Align copy with actual backend: encrypted local secrets store.
2. Show conditional persistence hint based on mode (`secret`, explicit persist policy).
3. Decide product behavior for non-secret persistence and make it explicit.

### P2: Delete action is destructive and conflates outcomes

Evidence:

1. Inline `Delete` link executes immediately with no confirm step (`src/prompts/env.rs:605`).
2. Delete and cancel both emit `None` callback, losing intent (`src/prompts/env.rs:233`).

Impact:

1. Accidental deletions are easy.
2. Callers cannot distinguish cancel vs delete for better UX messaging.

Recommendation:

1. Add confirm affordance (double-action or confirm modal).
2. Introduce typed outcome (`Submitted | Canceled | Deleted`) in callback/message path.

### P2: Env protocol is under-specified for UX, validation, and persistence policy

Evidence:

1. `Message::Env` includes only `id`, `key`, and optional `secret` (`src/protocol/message.rs:267`).
2. `execute_script` maps env with `prompt: None` and no metadata (`src/execute_script.rs:1288`).

Impact:

1. No per-provider validation rules (regex/length/checksum).
2. No way to control auto-submit behavior, delete affordance, or persistence mode.

Recommendation:

1. Extend env protocol with optional fields:
   `title`, `description`, `placeholder`, `validation`, `auto_submit_existing`, `persist`, `allow_delete`.
2. Thread these fields through `PromptMessage::ShowEnv` and `EnvPrompt::new`.

## Prioritized Roadmap

### Phase 1 (Correctness)

1. Add `auto_submit_existing` and `forcePrompt` support to stop silent stale-value loops.
2. Fix secret rendering to honor real cursor/selection state.
3. Prevent success callback when required persistence write fails.

### Phase 2 (Validation + Trust)

1. Add validation states and inline user feedback.
2. Add delete confirmation and distinct delete/cancel outcomes.
3. Correct user-facing storage copy and mode-specific persistence hints.

### Phase 3 (Protocol + SDK)

1. Expand env protocol/options to carry validation and persistence policy.
2. Replace broad secret inference with explicit options-first behavior.
3. Add telemetry for env prompt state transitions (`auto_submit`, `validation_failed`, `store_failed`, `deleted`).

## Suggested Tests (TDD Names)

1. `test_env_prompt_auto_submit_respects_auto_submit_existing_flag`
2. `test_env_prompt_force_prompt_bypasses_stored_secret_auto_submit`
3. `test_env_prompt_secret_mask_preserves_cursor_position_when_editing_mid_string`
4. `test_env_prompt_secret_mask_uses_char_count_for_unicode_input`
5. `test_env_prompt_shows_validation_error_when_submit_pressed_on_empty_input`
6. `test_env_prompt_does_not_emit_submit_success_when_secret_persist_fails`
7. `test_env_prompt_delete_requires_confirmation_before_secret_removal`
8. `test_env_prompt_emits_distinct_outcome_for_deleted_vs_canceled`
9. `test_sdk_env_secret_inference_does_not_force_secret_mode_for_generic_key_names`
10. `test_env_prompt_persistence_hint_matches_selected_persistence_mode`

## Risks / Known Gaps

1. Tightening secret inference may change behavior for scripts relying on legacy heuristic defaults.
2. Introducing typed delete/cancel outcomes requires protocol and SDK compatibility handling.
3. Validation schema design needs product decisions (strictness, provider-specific presets, fallback behavior).
