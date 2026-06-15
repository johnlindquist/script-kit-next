I'll inspect the actual source to ground the pragmatist analysis. Let me batch-read the key files.
Now let me look at the note switcher action builder, the day page view, and the related probes/docs.
Let me inspect the note switcher builder, the day page view bind logic, the existing probe, and any docs describing the no-crossover contract.
Let me verify the current state of `day_page_rows.rs`, the `NoteSwitcherNoteInfo` struct, the Day Page Cmd+P entrypoint, and how Notes opens a note.
Let me confirm the `NoteSwitcherNoteInfo` fields, check whether a no-crossover behavior test exists in source, and see if DayPageView has any CommandBar field.
## Role Findings

The task is almost entirely a **convergence** problem, not new feature work. Verified facts from source:

1. **The prior no-crossover work is already 90% done.** `src/notes/day_page_rows.rs`, `DayPageSwitcherInfo`, `get_day_page_switcher_actions`, and `open_day_page_in_main` **do not exist** (glob/grep return nothing). The *only* surviving remnant is the stale-rejection branch at `src/notes/window/panels.rs:311-319`. That branch is the single biggest thing to flip.
2. **The real divergence is the Day Page's bespoke overlay.** `src/main_sections/day_page_switcher.rs` is a hand-rolled `DaySwitcherState` + occluding `div()` overlay (`render_day_page_day_switcher_panel`, lines 243-337) with a custom `handle_day_switcher_key`. The Notes switcher is a real `CommandBar` (`src/notes/window.rs:375` `note_switcher: CommandBar`) fed by `get_note_switcher_actions` (`src/actions/builders/notes.rs:856`). These are two different containers — that is the "same UI/UX/container/component" violation.
3. **`DayPageView` has no `CommandBar` field** (`day_page_types.rs:60` only has `day_switcher: Option<DaySwitcherState>`). Adding one is the mechanical core of the task.
4. **`NoteSwitcherNoteInfo`** (`builders/notes.rs:757`) is a plain struct; `id: String` is already free-form. Day notes map in trivially.
5. **No no-crossover behavior test exists in source** — only the runtime probe `scripts/agentic/notes-day-page-switcher-probe.ts`, which asserts the *opposite* of the new contract and must be rewritten.
6. **Routing already exists on both sides**: Day Page `bind_day(date)` (`day_page_switcher.rs:129`) rebinds locally; Notes `select_note_internal` (`notes.rs:285`) loads content into `editor_state`. Both are the local-editor hooks we need.

## Evidence And Assumptions

**Verified (cited):**
- `panels.rs:311` is the only `daypage_` consumer in `src/`.
- `get_note_switcher_actions` emits `note_{id}` rows; `no_notes` is the empty-state row (`builders/notes.rs:890`).
- Day Page Cmd+P entrypoint: `day_page_view.rs:640-642` (`cmd+p` → `toggle_day_switcher`); also `day_page:open_past_day` action (`day_page_actions.rs:71,271`).
- Notes Cmd+P: `open_browse_panel` (`panels.rs:360`) builds `NoteSwitcherNoteInfo` rows from `self.notes` only.
- `day_page/document.rs` has `bind_date`/`bind_today`/`bound_date` — the Day editor's rebind API is stable and unit-tested.

**Assumptions (flag before coding):**
- "Open any search-selected referenced note inside its own editor" is read as scoped to **day notes** (the only concrete example given). Regular-note selection from the Day Page switcher is a product ambiguity — I recommend treating regular notes in the Day switcher as **Notes-window-only** (routed via the explicit action), not hosted in the day-bound editor, because the Day editor is day-file-bound and hosting arbitrary notes there breaks the binding model. Confirm with the user.
- Day notes edited from the Notes window should persist back to `brain/days/YYYY-MM-DD.md`. This needs a new Notes "day-note binding mode" (mirrors `select_note_internal` but sourced from/to the day file). Unverified: whether Notes already has any file-backed (non-DB) note persistence path to reuse.

## Failure Modes

1. **Bespoke-overlay deletion risk.** Removing `render_day_page_day_switcher_panel` + `handle_day_switcher_key` touches Day Page focus routing (`day_page_view.rs:640`, the root focus handle swap at `open_day_switcher` line 118). A CommandBar popup has different focus semantics than the in-window overlay; the detached-popup close path (`wire_command_bar_activation`, `panels.rs:49`) must be replicated on DayPageView or shared. This is the highest-risk spot.
2. **Two hosts, one builder, two id schemes.** If day rows reuse `note_{id}` with `id="daypage_..."`, the executor prefix-match (`strip_prefix("note_")`) breaks. Keep `daypage_{date}` as a distinct action-id prefix and handle both prefixes in each host's executor.
3. **Stale probe silently passing.** `notes-day-page-switcher-probe.ts` will *still pass* after a naive edit if rewritten to assert absence — it must be inverted to assert **presence** of day rows + host-local routing, or deleted.
4. **Comment/doc drift.** `panels.rs:308-310` documents the dead no-crossover contract in-source; leaving it misleads the next editor. GLOSSARY.md parity entry is also needed.
5. **"Open in Notes Window" scope creep.** This is a Day Page Cmd+K action, not a switcher concern; keep it in `day_page_actions.rs` and don't entangle it with switcher routing.

## Recommendation

**Smallest shape that satisfies "same container/component/result language":**

**1. Share the container/component (the core move):**
- Add `note_switcher: CommandBar` to `DayPageView` (`src/main_sections/day_page_types.rs`), constructed in `day_page_view.rs` init like `src/notes/window/init.rs:165`.
- Repoint `toggle_day_switcher` (`day_page_switcher.rs:101`) to open that CommandBar via the same `open_centered` + activation-wiring pattern as `NotesApp::open_browse_panel` (`panels.rs:360-408`). Extract the wiring into a shared helper if it's not already generic (check `src/actions/command_bar.rs`).
- **Delete** `DaySwitcherState`, `render_day_page_day_switcher_panel`, `handle_day_switcher_key`, `filtered_day_switcher_indices`, `DAY_SWITCHER_LIST_ID` (lines 15-89, 173-337). **Keep** `load_day_switcher_entries` (line 24) and `day_switcher_entry_label` (line 57) — reuse them to build shared rows.

**2. Model day notes in the shared result set:**
- Add a sibling builder `get_day_note_switcher_actions(days: &[DayNoteSwitcherInfo]) -> Vec<Action>` in `src/actions/builders/notes.rs` emitting `daypage_{date}` ids, reusing the same `Action`/`ActionCategory::ScriptContext`/icon/section language as `get_note_switcher_actions`. (Mirror the deleted builder the prior fusion run described.)
- Map `DaySwitcherEntry` → `DayNoteSwitcherInfo { date, label, preview }` via the existing `load_day_switcher_entries` + `day_switcher_entry_label`.
- Both hosts concat: `get_note_switcher_actions(&notes)` + `get_day_note_switcher_actions(&days)` (Notes historically did exactly this).

**3. Selection routing per host:**
- **Day Page** executor (new, mirrors `execute_note_switcher_action`): `daypage_{date}` → existing `bind_day(date)`; `note_{uuid}` → route to Notes window (or omit regular notes from the Day switcher entirely if the user confirms the narrow reading).
- **Notes** executor (`panels.rs:300`): **flip** the `daypage_` branch (lines 311-319) from reject to `self.open_day_note_in_editor(date, window, cx)` — a new method next to `select_note_internal` (`notes.rs:285`) that reads `brain/days/{date}.md` into `editor_state` and sets a day-note persistence target.
- **Explicit cross-window action**: add `day_page:open_in_notes_window` to `day_page_host_actions_section` (`day_page_actions.rs:34`) and handle it in `execute_day_page_action` (line 254) — opens the currently bound day note in the Notes window. This is the *only* Day→Notes path.

**4. Stale guards / tests / docs to remove or rewrite:**
- **Rewrite** `panels.rs:308-319` comment + branch (flip semantics).
- **Rewrite** `scripts/agentic/notes-day-page-switcher-probe.ts` → assert day rows **present** in both switchers and that selection stays in the host's own editor/window (no main-window activation from Notes; no Notes-window opening from Day Page except via the explicit action).
- **Update** `scripts/agentic/day-page-today-feature-probe.ts` only if the `Open Past Day`/`day_page:open_past_day` label changes (recommend keeping both to avoid churn).
- **Add** GLOSSARY.md entry documenting the shared switcher container with only default-binding difference.
- No source-audit test to remove (none exists in `src/`).

**5. Smallest verification plan:**
- Unit: `get_day_note_switcher_actions` emits `daypage_{date}` ids for sample days (new test in `builders/notes.rs` tests).
- Unit: `DayPageView` executor routes `daypage_2026-06-01` → `session.bound_date() == Some(date)` (reuse `bind_date` test patterns from `day_page/document.rs`).
- Unit: `NotesApp::execute_note_switcher_action("daypage_...")` opens in Notes editor and does **not** touch the main window (compiler-enforced: no main-window hook exists anymore).
- Runtime probe (rewritten): day rows appear in both switchers; Notes pick stays in Notes window; Day pick rebinds Day editor; explicit action is the only Day→Notes path.
- `./scripts/agentic/agent-cargo.sh test --lib notes::window::panels main_sections::day_page_switcher actions::builders::notes`

## Self Score

**8/10.** The convergence target is clear and the stale surface is tiny (one branch + one bespoke overlay + one inverted probe). The plan reuses existing stable APIs (`bind_day`, `select_note_internal`, `CommandBar`, `get_note_switcher_actions`) and deletes more bespoke code than it adds. Two open product/scope questions prevent a 9-10: (a) whether regular notes must be hostable in the Day Page editor (I assumed no), and (b) whether Notes needs full read/write persistence to day files or read-only display (I assumed read/write, which is the larger new capability). Confirm both with the user before coding; either "no" shrinks scope further.
