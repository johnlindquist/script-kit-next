## Role Findings

The cleanest refactor is to make Notes Cmd+P a Notes-only switcher again.

Remove the explicit Day Page append-and-handoff path from [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:397). That means:

- `open_browse_panel` builds only `get_note_switcher_actions(...)`.
- `execute_note_switcher_action` no longer recognizes `daypage_YYYY-MM-DD`.
- Delete `src/notes/day_page_rows.rs` if no other owner remains.
- Remove `crate::notes::day_page_rows` from [src/notes/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/mod.rs).
- Remove startup registration in [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs:255).
- Remove `ScriptListApp::open_day_page_in_main_window_hook` from [src/main_sections/day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:706).
- Remove `DayPageSwitcherInfo` and `get_day_page_switcher_actions` from [src/actions/builders/notes.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders/notes.rs) unless some other caller appears during implementation.

Do not make day pages “real notes” as part of this fix. That is a larger document-model change: day pages are files under `brain/days`, while Notes rows are database-backed `Note` records. Treating them as notes inside the floating Notes window would need a shared document abstraction and one persistence path, otherwise it risks two save semantics for the same conceptual text.

If users still need an intentional jump from Notes to the main Day Page, put that behind an explicit Cmd+K action like `Open Today in Day Page`, not in Cmd+P search results. I would not add that in the narrow refactor unless product explicitly asks for it.

## Evidence And Assumptions

The current leak is deliberate, not incidental. [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:397) loads `brain/days` rows, appends Day Page actions, and logs note switcher counts including day pages.

The redirection is explicit. [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:308) handles `daypage_` action ids, closes the Notes browse panel, then defers `open_day_page_in_main`.

The extra module documents the old product model: [src/notes/day_page_rows.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/day_page_rows.rs:1) says these are “Day Page rows for the Notes Cmd+P switcher” and that selection hands off to the main window.

The Day Page already has the correct host-local browser. [src/main_sections/day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:1) loads `brain/days`, filters query text, and on Enter calls `bind_day` in the existing `DayPageView`.

Assumption: the product contract prefers no Day Page rows in Notes Cmd+P, rather than keeping them and changing their destination. That best matches “current Notes window” and “never open or redirect into the main launcher.”

## Failure Modes

Leaving the `daypage_` handler in `execute_note_switcher_action` is a latent footgun. Even if rows disappear today, any future action id collision or reused action builder could reintroduce main-window activation.

Keeping `src/notes/day_page_rows.rs` around after disconnecting it invites a future rewire. Since it exists only for the Notes Cmd+P handoff, delete it if the compiler allows.

Making day pages Notes-openable now risks duplicate ownership: Notes database IDs and `brain/days/YYYY-MM-DD.md` paths would be mixed in one switcher without a typed document identity.

A source-audit test forbidding all strings like `daypage_` would be too broad because the Day Page surface can legitimately use day switching. Prefer behavior/unit tests and compiler deletion first.

## Recommendation

Implementation plan:

1. In `src/notes/window/panels.rs`, remove the `daypage_` branch from `execute_note_switcher_action`.
2. In `open_browse_panel`, change `let mut note_switcher_actions` to `let note_switcher_actions`, remove `load_day_page_switcher_rows`, remove `get_day_page_switcher_actions`, and update the log to only report note switcher actions.
3. Delete `src/notes/day_page_rows.rs`, remove its module export, remove startup hook registration, and remove `open_day_page_in_main_window_hook`.
4. In `src/actions/builders/notes.rs`, delete `DayPageSwitcherInfo`, `get_day_page_switcher_actions`, and its unit test if unused.
5. Move useful day-page loader coverage from `notes/day_page_rows.rs` into `src/main_sections/day_page_switcher.rs` tests if equivalent coverage is missing: newest-first ordering, preview extraction, invalid filename filtering, and query filtering.

Tests to run/add:

- Add a focused Notes window unit/behavior test if constructors permit: opening browse panel with day files present produces no action ids starting `daypage_` and no `"Day Pages"` section.
- Add or preserve Day Page switcher tests in `src/main_sections/day_page_switcher.rs` for `load_day_switcher_entries`, `day_switcher_entry_label`, and `filtered_day_switcher_indices`.
- Run:
  ```bash
  ./scripts/agentic/agent-cargo.sh test --lib notes::window
  ./scripts/agentic/agent-cargo.sh test --lib main_sections::day_page_switcher
  ./scripts/agentic/agent-cargo.sh test --lib actions::builders::notes
  ```
  Adjust module paths to the actual test names if the crate’s test filter needs exact substrings.

Runtime proof:

- Create two `brain/days/YYYY-MM-DD.md` files and at least one normal note.
- Open floating Notes window.
- Press Cmd+P.
- Search text unique to a day page.
- Verify no Day Page result appears.
- Select a normal note result and press Enter.
- Verify the floating Notes window remains active, the selected note opens there, and the main launcher does not switch to `AppView::DayPage`.

If there is an existing DevTools probe path for Notes automation, add a narrow probe under `scripts/agentic/` that records `notes.switcher` rows plus main-window current view before and after selection.

## Self Score

8/10. The ownership boundary is clear and the plan is narrow. The main uncertainty is the exact easiest behavior-test harness for `NotesApp::open_browse_panel`; if setup is heavy, the fallback should be a small runtime probe plus unit tests around the Day Page switcher and action builders.


