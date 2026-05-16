# 015 SDK TermPrompt Bundle Map

Slug: `sdk-term-prompt-atlas`

Feature: SDK TermPrompt / `term()` prompt runtime / terminal actions / full-height terminal.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/quick-terminal-pty/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `lat.md/design.md`
- `lat.md/surfaces.md`
- `lat.md/acp-chat.md`
- `lat.md/protocol.md`
- `lat.md/scripting.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/render_prompts/term.rs`
- `src/term_prompt/mod.rs`
- `src/terminal/mod.rs`
- `src/terminal/alacritty.rs`
- `src/terminal/alacritty/handle_creation.rs`
- `src/terminal/alacritty/handle_runtime.rs`
- `src/terminal/pty.rs`
- `src/terminal/pty/lifecycle.rs`
- `src/terminal/pty/io_ops.rs`
- `src/app_impl/actions_dialog.rs`
- `src/app_impl/actions_toggle.rs`
- `src/app_impl/ui_window.rs`
- `src/app_layout/collect_elements.rs`
- `src/app_layout/build_layout_info.rs`
- `src/main_sections/app_view_state.rs`
- `src/window_resize/mod.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `tests/sdk/test-term.ts`
- `tests/smoke/test-term.ts`
- `tests/smoke/test-term-height.ts`
- `tests/smoke/test-term-footer.ts`
- `tests/quick_terminal_contracts.rs`
- `tests/main_window_footer_surface_owner_contract.rs`
- `tests/source_audits/resize_presentation_contract.rs`
- `scripts/agentic/footer-ownership-matrix.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/quick-terminal-pty/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/design.md lat.md/surfaces.md lat.md/acp-chat.md lat.md/protocol.md lat.md/scripting.md lat.md/verification.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/render_prompts/term.rs src/term_prompt/mod.rs src/terminal/mod.rs src/terminal/alacritty.rs src/terminal/alacritty/handle_creation.rs src/terminal/alacritty/handle_runtime.rs src/terminal/pty.rs src/terminal/pty/lifecycle.rs src/terminal/pty/io_ops.rs src/app_impl/actions_dialog.rs src/app_impl/actions_toggle.rs src/app_impl/ui_window.rs src/app_layout/collect_elements.rs src/app_layout/build_layout_info.rs src/main_sections/app_view_state.rs src/window_resize/mod.rs src/main_entry/runtime_stdin_match_simulate_key.rs tests/sdk/test-term.ts tests/smoke/test-term.ts tests/smoke/test-term-height.ts tests/smoke/test-term-footer.ts tests/quick_terminal_contracts.rs tests/main_window_footer_surface_owner_contract.rs tests/source_audits/resize_presentation_contract.rs scripts/agentic/footer-ownership-matrix.ts -s "globalThis.term" -s "function term" -s "type: 'term'" -s "AppView::TermPrompt" -s "TermPrompt::with_height" -s "ViewType::TermPrompt" -s "render_term_prompt" -s "render_terminal_prompt_hint_strip" -s "ActionsDialogHost::TermPrompt" -s "TERM_PROMPT_CLEAR" -s "TERM_PROMPT_ACTIONS" -s "collect_term_prompt_elements" -s "prompt_type" -s "TerminalHandle::with_command_and_theme" -s "TerminalHandle::new_with_theme" -s "term_prompt" -s "Escape" -s "Cmd+K" -s "Cmd+Shift+K" -l 5 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/sdk-term-prompt-atlas.txt
```

Initial reduced bundle size on disk: 174,612 bytes. Packx reported about 46.9k tokens and 172,457 total chars after reducing from the initial 76.2k-token bundle.

Browser recovery attempts with the 46.9k-token bundle left stale Oracle metadata before a recoverable answer was written, so the session was retried with a tighter bundle:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/quick-terminal-pty/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/protocol-automation/SKILL.md lat.md/design.md lat.md/surfaces.md lat.md/acp-chat.md lat.md/scripting.md lat.md/verification.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/render_prompts/term.rs src/term_prompt/mod.rs src/terminal/alacritty/handle_creation.rs src/terminal/pty/lifecycle.rs src/app_impl/actions_toggle.rs src/app_impl/actions_dialog.rs src/app_layout/collect_elements.rs src/main_sections/app_view_state.rs src/window_resize/mod.rs tests/sdk/test-term.ts tests/quick_terminal_contracts.rs tests/source_audits/resize_presentation_contract.rs tests/main_window_footer_surface_owner_contract.rs -s "globalThis.term" -s "AppView::TermPrompt" -s "TermPrompt::with_height" -s "ViewType::TermPrompt" -s "render_term_prompt" -s "render_terminal_prompt_hint_strip" -s "ActionsDialogHost::TermPrompt" -s "TERM_PROMPT_CLEAR" -s "TERM_PROMPT_ACTIONS" -s "collect_term_prompt_elements" -s "TerminalHandle::with_command_and_theme" -s "TerminalHandle::new_with_theme" -s "quick_terminal_native_footer_does_not_capture_sdk_term_prompt_footer" -s "term()" -l 3 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/sdk-term-prompt-atlas-tight.txt
```

Tight bundle size on disk: 37,833 bytes. Packx reported about 10.2k tokens and 36,193 total chars.

Successful fallback command:

```bash
oracle --engine browser --browser-model-strategy ignore --browser-attachments never --timeout 30m --heartbeat 30 --slug sdk-term-atlas-cli --write-output feature-map/raw-oracle/015-sdk-term-prompt/answer.cli.md --prompt "[sdk-term-atlas-cli] ..." --file /Users/johnlindquist/.oracle/bundles/sdk-term-prompt-atlas-tight.txt /Users/johnlindquist/dev/script-kit-gpui/feature-map/raw-oracle/015-sdk-term-prompt/prompt.md
```

The CLI routed through the remote browser host, completed with `gpt-5.4-pro`, and wrote the final answer to `answer.cli.md`, which was copied to canonical `answer.md`.
