Implemented the theme-consistency token replacements requested in `plans/theme-consistency.md` across the targeted theme/UI files.

**Changed files**
- `src/components/button.rs:59`  
Replaced hardcoded hover overlays with token-derived overlays (`accent.selected_subtle` / design `background_selected`), and default colors now derive from `Theme::default()`. Added tests for both theme/design overlay mapping.
- `src/components/form_fields.rs:159`  
Cursor color now resolves from theme cursor style / theme accent fallback, and design path now uses `design_colors.accent` instead of hardcoded cyan.
- `src/components/form_fields_tests.rs:220`  
Added `test_form_field_colors_from_design_uses_design_accent_for_cursor`.
- `src/components/prompt_header.rs:104`  
`logo_icon` now uses `theme.colors.text.on_accent`; hover overlay now derives from theme/design tokens. Added focused test coverage.
- `src/components/alias_input.rs:74`  
Overlay background now uses `theme.colors.background.main` (no hardcoded black). Added test coverage.
- `src/components/shortcut_recorder.rs:72`  
Overlay background now uses `theme.colors.background.main`. Added test coverage.
- `src/components/toast.rs:79`  
`details_bg` and hover overlays now derive from theme/design tokens (no hardcoded `0x00000020`). Added tests for both paths.
- `src/prompts/chat.rs:2453`  
Replaced mixed `cx.theme()` usage for setup CTA text/icon with Script Kit token (`self.theme.colors.text.on_accent`).
- `src/prompts/select.rs:556`  
Added variant-aware search box background resolver so non-default design variants use design-token backgrounds. Added test coverage.

**How to test**
1. `cargo check`  
Result: passed.
2. `cargo build`  
Result: passed.
3. UI stdin protocol smoke test:  
`timeout 8 bash -c 'echo '\''{"type":"show"}'\'' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`  
Verify in `~/.scriptkit/logs/latest-session.jsonl` that `stdin_command_parsed` shows `command_type":"show"` and `Processing external command type=show`.

**Risks / known gaps**
- `cargo clippy --all-targets -- -D warnings` fails due unrelated existing issues outside theme scope (not modified here), including:
  - `src/actions/dialog_builtin_action_validation_tests_*.rs` unused imports
  - `src/actions/dialog.rs:308` (`clippy::too_many_arguments`)
  - `src/prompts/div.rs` (`field_reassign_with_default`)
- `cargo test` fails due unrelated existing test compile error in `src/ai/window.rs:8729` (`AiApp::message_body_content` missing).
- Because of those unrelated failures, full test gate could not be completed end-to-end for this branch state.

**Commits**
- No commits were made.