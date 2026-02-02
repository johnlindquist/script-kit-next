# Cmd+K Actions Button (Clipboard History) — Research

## Files investigated
- `src/components/prompt_footer.rs` — PromptFooterConfig defaults, show_secondary flag, and secondary button wiring.
- `src/render_builtins.rs` — clipboard history render + footer wiring; clipboard actions toggle logic.
- `src/actions/dialog.rs` — ActionsDialog::with_clipboard_entry constructor and callback plumbing.
- `src/actions/builders.rs` — get_clipboard_history_context_actions action list for clipboard entries.

## Current behavior
- Clipboard history footer only shows the primary "Paste" button.
- There is no secondary "Actions" button in the clipboard footer, so Cmd+K has no visible affordance there.

## Root cause
- Clipboard history footer uses `PromptFooterConfig::new().primary_label("Paste")...show_secondary(false)`.
- The clipboard footer is not wired with an `on_secondary_click` callback, so even if the button were shown, there is no toggle path connected to open the actions dialog.

## Proposed solution
- Show a secondary "Actions" button with shortcut hint "⌘K" when a clipboard entry is selected.
- Wire the footer secondary click to toggle actions: call `toggle_clipboard_actions(...)` to open `ActionsDialog::with_clipboard_entry(...)`.
- Use `get_clipboard_history_context_actions(...)` (already defined) as the action source.

## References
- PromptFooter config + show_secondary flag: `src/components/prompt_footer.rs` (PromptFooterConfig + builder, ~lines 90-170).
- Clipboard history footer config (currently show_secondary=false): `src/render_builtins.rs` (clipboard history footer, ~lines 670-710).
- Clipboard actions dialog builder: `src/actions/dialog.rs` (`ActionsDialog::with_clipboard_entry`, ~lines 311-350).
- Clipboard action list: `src/actions/builders.rs` (`get_clipboard_history_context_actions`, ~lines 814+).

## Verification
1) What changed
   - Added `toggle_clipboard_actions` method.
   - Added Cmd+K key handler.
   - Updated footer with `show_secondary(has_entry)` and `on_secondary_click` callback.
2) Test results
   - `cargo check` passed.
   - `cargo clippy --all-targets -- -D warnings` passed.
   - `cargo test` passed.
3) Before/after
   - Before: footer had `show_secondary(false)`.
   - After: footer shows Actions button when a clipboard entry is selected.
4) Deviations
   - No deviations from proposed solution.
