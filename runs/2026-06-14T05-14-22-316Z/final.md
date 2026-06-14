The cleanest refactor is to make floating Notes Cmd+P a Notes-only switcher and remove the Notes-to-main-Day-Page handoff path.

The current violation is assembled in `NotesApp::open_browse_panel` and dispatched in `NotesApp::execute_note_switcher_action`: `open_browse_panel` appends day-page rows to the Notes switcher, then `execute_note_switcher_action` recognizes `daypage_YYYY-MM-DD`, closes the Notes browse panel, and defers opening the main launcher Day Page. That directly violates “result opens in the current Notes window.”

**Plan**

1. In `src/notes/window/panels.rs`
   - Remove `load_day_page_switcher_rows(...)`.
   - Remove `get_day_page_switcher_actions(...)`.
   - Build the switcher only from `get_note_switcher_actions(...)`.
   - Remove the entire `daypage_` branch from `execute_note_switcher_action`.
   - Simplify logs so they only describe Notes actions.

2. Delete the Notes-owned day-page handoff machinery if the compiler confirms no remaining callers:
   - `src/notes/day_page_rows.rs`
   - `pub(crate) mod day_page_rows` in `src/notes/mod.rs`
   - `DayPageSwitcherInfo` and `get_day_page_switcher_actions` in `src/actions/builders/notes.rs`
   - related re-exports in `src/actions/builders.rs` and `src/actions/mod.rs`
   - startup registration in `src/app_impl/startup.rs`
   - `ScriptListApp::open_day_page_in_main_window_hook` in `src/main_sections/day_page_view.rs`

Deleting those APIs is a good current-code cleanup because it makes existing direct references fail to compile. It is not a permanent semantic firewall: future code could still invent a new `daypage_` action or call generic main-window APIs, so the behavior test below matters.

3. Preserve the main Day Page switcher:
   - Keep `src/main_sections/day_page_switcher.rs` as the owner for browsing past/today day pages from the main launcher Day Page.
   - If deleted `src/notes/day_page_rows.rs` tests covered useful day-file parsing behavior, move equivalent coverage to `day_page_switcher` tests instead of losing it.

**Tradeoff**

Notes Cmd+P should simply stop including day-page rows.

Do not make day pages “real notes” in this refactor. Day pages are file-backed under `brain/days/YYYY-MM-DD.md`; Notes appear to be note-record-backed with `note_{uuid}` action ids. Mixing them inside the floating Notes switcher would require a real shared document abstraction and save-path design. That is larger, riskier, and unnecessary for this contract.

A future explicit action like “Open Today in Day Page” could be acceptable if product wants an intentional cross-surface jump, but it should not live as a Cmd+P search result from the floating Notes window.

**Tests**

Prefer behavior/compiler coverage, not source-audit tests.

Add or update a test that covers the final Notes Cmd+P action set, not only `get_note_switcher_actions`. A builder-only test is weak because the current bug is appended in `open_browse_panel` after the builder returns.

Best target, after inspecting available helpers in `src/notes/window/tests.rs`:
- create a test fixture with normal notes and day files present;
- call `NotesApp::open_browse_panel`;
- inspect the command bar actions;
- assert no action id starts with `daypage_`;
- assert no section/title represents “Day Pages”;
- assert normal `note_{uuid}` actions still select notes in the same `NotesApp`.

Also:
- delete tests for `get_day_page_switcher_actions`;
- migrate useful day-file loader tests to `src/main_sections/day_page_switcher.rs`;
- keep `tests/actions.rs` source-audit tests untouched unless they fail for a legitimate reason.

Verification commands should use the wrapper:

```bash
./scripts/agentic/agent-cargo.sh check --lib
./scripts/agentic/agent-cargo.sh test --lib notes::window
./scripts/agentic/agent-cargo.sh test --lib main_sections::day_page_switcher
./scripts/agentic/agent-cargo.sh test --test actions
```

Adjust filters to actual test names after implementation.

**Runtime Proof**

Add or run a narrow devtools/runtime probe, using existing probe primitives after inspection rather than inventing API names.

It should prove both sides of the contract:

1. Floating Notes window Cmd+P opens the Notes browse/search UI.
2. With day files present, a query unique to a day page produces no Day Page rows.
3. Selecting a normal note result opens that note in the same floating Notes window.
4. The main launcher does not activate, does not become the target window, and does not switch to `AppView::DayPage` as a result of Notes Cmd+P.

That runtime proof catches the real failure mode: not just row absence, but absence of surprise main-window Day Page activation.
