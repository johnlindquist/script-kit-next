We are in /Users/johnlindquist/dev/script-kit-gpui.

Task: Plan the minimal code change to make the Day Page editor auto-focus and auto-scroll/reveal to the bottom/end of the editor when the day view opens/reopens, when Day Page content is loaded/rebound, and when focus returns after Day Page popups/actions close. Then identify the smallest tests/probes to prove it.

User-visible bug: Day view should land at the bottom of the editor, focused, like a journal/today surface. Existing focus fixes for Cmd+P/actions are committed separately. Now implement bottom focus/scroll behavior.

Relevant repo process:
- AGENTS.md says inspect current source/tests before editing, use shared components, prefer runtime proof for focus/window behavior, cargo through ./scripts/agentic/agent-cargo.sh.
- Notes and Day Page are separate surfaces. Do not change Notes window behavior unless shared helper is safe and only Day invokes the new behavior.
- Day Page uses autosave as truth.

Observed current source:
- src/main_sections/day_page_view.rs owns DayPageView.
- DayPageView::new creates a shared NotesEditor via NotesEditor::new_markdown_pair(... "input:day-page-editor" ...).
- DayPageView::apply_loaded_content_to_editor currently calls editor.load_value_with_cursor_at_end(content, window, cx).
- DayPageView::focus_editor currently calls editor.focus(window, cx).
- DayPageView::set_input and append_main_hotkey_carry set value with cursor at end. append then focuses.
- src/components/notes_editor/component.rs: NotesEditor::focus only focuses InputState. load_value_with_cursor_at_end and set_value_with_cursor_at_end set selection to value.len().
- vendor/gpui-component/crates/ui/src/input/state.rs: InputState::set_selection calls scroll_to(end, None, cx) and focus. InputState::scroll_to returns early if last_layout or last_bounds are not populated, so first mount/rebind may need a post-layout repeat.
- NotesEditor has markdown_runtime_info_with_scroll exposing InputState::automation_scroll_metrics for DevTools proof.
- Existing probes: scripts/agentic/day-notes-editor-runtime-parity-probe.ts checks day editor scroll metrics after long typing; scripts/agentic/day-cmdp-focus-probe.ts opens Day Page via comma special action and verifies focus after popups.

Candidate implementation already under consideration:
- Add NotesEditor::focus_with_cursor_at_end(window, cx), which reads state.value().len() and calls state.set_selection(cursor, cursor, window, cx).
- Make DayPageView::focus_editor call that helper immediately, then use window.defer with cx.entity().downgrade() to call the same helper again after layout.
- Keep the deferred repeat Day-owned, not a change to Notes window focus behavior.

Need from Fusion:
1. Confirm or correct the implementation plan.
2. Explain how to ensure scroll happens after GPUI layout/render timing.
3. Identify any better shared editor helper name/API.
4. Smallest Rust test(s) or source-level checks if useful.
5. DevTools probe design with assertions against focusedSemanticId and editor_scroll_metrics.
6. Risks and edge cases: empty files, short files, switching past/today/fragment, popup Escape restoring Day Page focus, and not disturbing Notes window behavior.
