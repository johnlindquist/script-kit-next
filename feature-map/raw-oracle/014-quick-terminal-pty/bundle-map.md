# 014 Quick Terminal PTY Bundle Map

Slug: `quick-terminal-pty-atlas`

Feature: Quick Terminal PTY / TermPrompt / warm pool / native footer / apply-back.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/quick-terminal-pty/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `lat.md/acp-chat.md`
- `lat.md/surfaces.md`
- `lat.md/automation.md`
- `lat.md/protocol.md`
- `lat.md/verification.md`
- `src/app_execute/utility_views.rs`
- `src/app_impl/quick_terminal_warm.rs`
- `src/app_impl/ui_window.rs`
- `src/app_impl/tab_ai_mode/mod.rs`
- `src/render_prompts/term.rs`
- `src/term_prompt/mod.rs`
- `src/terminal/mod.rs`
- `src/terminal/alacritty.rs`
- `src/terminal/alacritty/handle_creation.rs`
- `src/terminal/alacritty/handle_runtime.rs`
- `src/terminal/alacritty/handle_navigation.rs`
- `src/terminal/pty.rs`
- `src/terminal/pty/lifecycle.rs`
- `src/terminal/pty/io_ops.rs`
- `src/window_resize/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/app_state.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `src/main_entry/app_run_setup.rs`
- `src/prompt_handler/mod.rs`
- `src/actions/builders/file_path.rs`
- `src/app_impl/execution_paths.rs`
- `tests/quick_terminal_contracts.rs`
- `tests/tab_ai_routing.rs`
- `tests/tab_ai_harness_submission.rs`
- `tests/tab_ai_context.rs`
- `tests/tab_ai_input_coverage.rs`
- `tests/main_window_footer_surface_owner_contract.rs`
- `tests/sdk/test-term.ts`
- `scripts/agentic/footer-ownership-matrix.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/quick-terminal-pty/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/acp-chat.md lat.md/surfaces.md lat.md/automation.md lat.md/protocol.md lat.md/verification.md src/app_execute/utility_views.rs src/app_impl/quick_terminal_warm.rs src/app_impl/ui_window.rs src/app_impl/tab_ai_mode/mod.rs src/render_prompts/term.rs src/term_prompt/mod.rs src/terminal/mod.rs src/terminal/alacritty.rs src/terminal/alacritty/handle_creation.rs src/terminal/alacritty/handle_runtime.rs src/terminal/alacritty/handle_navigation.rs src/terminal/pty.rs src/terminal/pty/lifecycle.rs src/terminal/pty/io_ops.rs src/window_resize/mod.rs src/main_sections/app_view_state.rs src/main_sections/app_state.rs src/main_entry/runtime_stdin_match_simulate_key.rs src/main_entry/app_run_setup.rs src/prompt_handler/mod.rs src/actions/builders/file_path.rs src/app_impl/execution_paths.rs tests/quick_terminal_contracts.rs tests/tab_ai_routing.rs tests/tab_ai_harness_submission.rs tests/tab_ai_context.rs tests/tab_ai_input_coverage.rs tests/main_window_footer_surface_owner_contract.rs tests/sdk/test-term.ts scripts/agentic/footer-ownership-matrix.ts -s "QuickTerminalView" -s "quick_terminal" -s "quick_terminal_warm" -s "warm_quick_terminal_pty" -s "take_quick_terminal_warm_pty" -s "open_quick_terminal" -s "TermPrompt" -s "with_existing_terminal" -s "new_with_theme" -s "PROMPT_EOL_MARK" -s "ZDOTDIR" -s "quick_terminal_can_apply_back" -s "apply_tab_ai_result_from_terminal" -s "close_quick_terminal" -s "Cmd+W" -s "Shift+Tab" -s "footer:native:close" -s "quick_terminal_content_height" -s "quick_terminal_panel_height" -l 5 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/quick-terminal-pty-atlas.txt
```

Final bundle size on disk: 167,795 bytes. Packx reported about 47.2k tokens and 165,638 total chars after reducing from the initial 69.7k-token bundle.
