Panel-specific reasoning contract:
Panel role: architect
Focus on the complete design, tradeoffs, implementation shape, and how the pieces fit together.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
We are in /Users/johnlindquist/dev/script-kit-gpui. Please review this behavior/refactor question and propose a concrete implementation plan with files, APIs, and tests.

Product contract we want:
- The floating Notes window has Cmd+P search/browse for notes.
- When Cmd+P is used from the Notes window and a result is opened, it must always open in the current Notes window.
- It must never open or redirect into the main launcher's Day Page / main "day" window.
- The Day Page is just today's notes surface hosted inside the main launcher window so users can jump there from the launcher and stay in flow. It is not a separate destination for Notes-window search results.
- Past/today day-page browsing may still belong to the main Day Page surface's own Cmd+P, but should not leak into the Notes-window Cmd+P behavior unless there is a strongly justified host-specific design.

Current code observations from source:
- GLOSSARY.md says:
  - Notes Window: floating, persistent overlay editor panel for creating and browsing notes; `NotesApp`; src/notes/window.rs.
  - Day Page: today's diary surface inside the main launcher window, same frame as Script List, hosts shared notes editor bound to `brain/days/YYYY-MM-DD.md`; `DayPageView`, `AppView::DayPage`; src/main_sections/day_page_view.rs and day_page_types.rs.
- Notes Cmd+P is a `note_switcher` CommandBar in `src/notes/window/panels.rs`.
- `NotesApp::open_browse_panel` currently builds note rows with `get_note_switcher_actions(...)`, then appends day page rows:
  - `crate::notes::day_page_rows::load_day_page_switcher_rows(&crate::notes::notes_brain_days_dir(), chrono::Local::now().date_naive(), DAY_PAGE_SWITCHER_ROW_LIMIT)`
  - `note_switcher_actions.extend(crate::actions::get_day_page_switcher_actions(...))`
- `NotesApp::execute_note_switcher_action` currently handles action ids starting with `daypage_YYYY-MM-DD` by closing the browse panel and deferring `crate::notes::day_page_rows::open_day_page_in_main(date, cx)`, logging `notes_note_switcher_day_page_handoff`.
- `src/notes/day_page_rows.rs` says: "Day Page rows for the Notes Cmd+P switcher" and documents that Notes lists day pages read-through and hands a pick off to the main window. It has a hook `register_open_day_page_in_main_hook` and `open_day_page_in_main(date, cx)`.
- `src/main_sections/day_page_view.rs` has `open_day_page_in_main_window_hook(date, cx)` registered for that handoff; it shows the main window Day Page and binds the selected date.
- Separately, `src/main_sections/day_page_switcher.rs` is the Day Page's own past-day switcher (Cmd+P). It lists `brain/days/YYYY-MM-DD.md`, filters typed query, and `bind_day(date)` inside `DayPageView`. That seems like the correct place for day-page switching.
- `src/notes/window/window_ops.rs::open_notes_search` opens/raises Notes and calls `app.open_browse_panel(window, cx)`.
- `src/notes/window/keyboard.rs` routes keys while `self.note_switcher.is_open()`; Enter executes `self.note_switcher.execute_selected_action(cx)` then `self.execute_note_switcher_action(&action_id, window, cx)`.
- Note selection action ids are `note_{uuid}`; `execute_note_switcher_action` closes browse and calls `self.select_note(note_id, window, cx)`.
- Existing source-audit-ish tests mention Notes command routing and note switcher in `tests/actions.rs`, `src/notes/window/tests.rs`, and tests around `day_page_rows`.

Please answer:
1. What is the cleanest refactor to enforce the contract above?
2. Should Notes Cmd+P simply stop including day page rows, or should day pages become real notes openable in Notes, or should there be an explicit separate action/surface? Explain the tradeoff.
3. Which files/functions should change?
4. Which tests should be added/changed, preferring compiler/behavior tests over source-audit tests per AGENTS.md policy?
5. What runtime proof should verify this if we implement it? We use `./scripts/agentic/agent-cargo.sh` for cargo.

Please be concrete and critical. Assume we want a narrow, maintenance-friendly refactor and no surprise main-window activation from Notes-window search.