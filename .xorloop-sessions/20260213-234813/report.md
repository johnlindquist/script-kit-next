# xorloop Report — 20260213-234813

**Project:** script-kit-gpui
**Branch:** main
**Started:** Fri Feb 13 23:48:13 MST 2026

---

## Iteration 1 — DRY violations (00:36)

**Feature:** FEATURE: Prompt key handler setup across render_prompts/ where each prompt type (arg, div, editor, form, term) duplicates ~50 lines of identical closure boilerplate for toggle_predicate, on_toggle_actions, on_actions_dialog_execute, allow_sdk_shortcuts, and on_sdk_shortcut when calling handle_prompt_key_preamble()
**Commit:** `38fb16ad`



Here are the swarm tasks:

## SWARM TASK LIST

- `add-default-preamble-helper` scope:`src/render_prompts/key_handler.rs` — Add convenience function wrapping handle_prompt_key_preamble with default closures for simple prompts
- `simplify-arg-div-form-preamble` scope:`src/render_prompts/arg/render.rs, src/render_prompts/div.rs, src/render_prompts/form/render.rs` — Replace verbose 6-closure preamble calls in arg/div/form with new default helper


---

## Iteration 2 — consistency cleanup (01:16)

**Feature:** FEATURE: Keyboard key matching in prompt renderers uses three different patterns (match with 4 variants, match with 2 mixed-case variants, and eq_ignore_ascii_case) instead of the canonical lowercase-pair match convention.
**Commit:** `18d5f373`



Here are the extracted swarm tasks:

- `ui-key-helpers` scope:`src/ui_foundation/mod.rs, src/confirm/window.rs` — Add `is_key_tab` and `is_key_space` helpers, update confirm window
- `prompts-path-template-drop-keys` scope:`src/prompts/path/render.rs, src/prompts/template/render.rs, src/prompts/drop.rs` — Convert Path, Template, Drop key matching to canonical helpers
- `prompts-select-keys` scope:`src/prompts/select/render.rs` — Replace inline key booleans with canonical helpers in select prompt
- `chat-key-actions` scope:`src/prompts/chat/types.rs` — Refactor setup card and chat input key resolution to canonical helpers


---

