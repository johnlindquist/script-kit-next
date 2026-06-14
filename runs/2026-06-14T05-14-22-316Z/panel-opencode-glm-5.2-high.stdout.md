I'll explore the actual code to ground my edge-case analysis in evidence before proposing a plan.
Now let me look at the Day Page switcher (the main window's own), the actions module, and the keyboard router.
Let me look at the actions builder for day pages, the day_page_view handoff hook, and existing tests for note switcher routing.
Now let me read the day page hook in the main window, the actions builder for day pages, and check how the main window day page switcher differs.
Let me check the existing test for the day page switcher actions and any ratchet/source-audit tests that mention daypage, plus confirm the `no_notes` placeholder path.
Let me check the existing source-audit test referencing `open_notes_search` and the `no_notes` path, since those could be affected by the refactor.
## Role Findings

The leak is real, fully wired, and has multiple silent-failure modes. The cleanest refactor is **removal**, not promotion or a new surface.

**The violation path (confirmed):**
- `open_browse_panel` (`src/notes/window/panels.rs:399-413`) appends day-page rows from `load_day_page_switcher_rows` + `get_day_page_switcher_actions` to the note switcher every time Cmd+P opens.
- `execute_note_switcher_action` (`panels.rs:314-334`) handles `daypage_YYYY-MM-DD` by closing the browse panel and deferring `open_day_page_in_main(date, cx)`.
- That hook (`src/main_sections/day_page_view.rs:710-742`) dispatches `WindowEvent::ShowMain` + `show_day_page_view` — i.e. it **makes the main launcher window visible and binds its Day Page**. That is precisely the "surprise main-window activation from Notes-window search" the contract forbids.

**The main Day Page surface already owns past-day Cmd+P:** `src/main_sections/day_page_switcher.rs` (`toggle_day_switcher`, `bind_day`, `load_day_switcher_entries`) is the correct, complete switcher. The Notes-side listing is a **third duplicate** of the day-file read logic (Notes loader, main switcher loader, plus the row builder) and a second-class citizen: it caps at 90 rows (`DAY_PAGE_SWITCHER_ROW_LIMIT`) while the main switcher is uncapped, so older days are silently unreachable from Notes.

## Evidence And Assumptions

- Call graph is closed: `grep` shows `get_day_page_switcher_actions` / `load_day_page_switcher_rows` have exactly **one** integrator (`panels.rs`); `DayPageSwitcherInfo` is `pub`-re-exported (`src/actions/mod.rs:45,49`, `src/actions/builders.rs:53`) but unused elsewhere. Removing the re-exports makes the compiler the enforcer (AGENTS.md rung 1).
- The hook chain: `register_open_day_page_in_main_hook` (`app_impl/startup.rs:257`) → `open_day_page_in_main_window_hook` (`day_page_view.rs:710`). Both exist solely for this handoff.
- `src/notes/window/init.rs:110` calls `get_note_switcher_actions` for initial state but does **not** append day pages — so `panels.rs` is the sole integrator.
- Existing source-audit `tests/actions.rs:1564` pins `SearchNotes => notes::open_notes_search(cx)` — **unaffected**; `open_notes_search` still exists, only the row population inside changes.
- `tests.rs:593` pins the `replace_active_note_mention_with_note` line — **unaffected** by this refactor.

Assumption: no external/MCP/automation caller depends on `daypage_` ids appearing in the Notes switcher. Confirmed by grep: the `daypage_` prefix is produced and consumed only within `panels.rs` + `builders/notes.rs`.

## Failure Modes

1. **Silent no-op on parse failure** (`panels.rs:315,329-331`): a malformed `daypage_` id logs a warning and returns after the browse panel was already closed — user gets a no-op with zero feedback. Removing the branch eliminates this.
2. **Deferred-focus-steal race** (`panels.rs:319` `cx.defer`): if the Notes window closes between the pick and the deferred `open_day_page_in_main`, the main window still gets activated — a time-delayed focus steal from a window the user already dismissed. Removal eliminates the race.
3. **Silent failure when main window is gone** (`day_page_view.rs:714-716`, `day_page_rows.rs:97-104`): hook returns `false`, browse panel already closed, only a warn-log. User sees nothing happen.
4. **Synchronous FS I/O on every Cmd+P open** (`panels.rs:399` → `load_day_page_switcher_rows` does `read_dir` + `read_to_string` per file on the main thread). With a large `brain/days/` dir this stalls popup-open latency for rows the contract says shouldn't be there at all.
5. **Hidden bug fixed by removal**: with zero notes but existing day files, the switcher today shows `no_notes` placeholder **plus** day-page rows — a fresh user can pick a day page and get yanked to the main window. Post-refactor they get the clean `no_notes` state.
6. **Cap inconsistency**: 90-row limit on Notes side vs. uncapped on main side — another reason the Notes listing is a confusing second-class surface.

## Recommendation

**Option 1 (recommended): remove the Notes→main day-page handoff entirely.** Day pages stay reachable via the main Day Page's own Cmd+P (`day_page_switcher.rs`). This is the narrow, maintenance-friendly path and the only one that satisfies "never open or redirect into the main launcher."

**Why not Option 2 (make day pages real notes):** that creates a second save path for `brain/days/YYYY-MM-DD.md` against a SQLite-backed notes DB — the code comment at `panels.rs:310-313` already calls this out as the reason for the handoff. High data-integrity risk, large scope.

**Why not Option 3 (new explicit surface):** violates the AGENTS.md UI-consistency contract (a third switcher system next to the two existing ones). The main Day Page switcher is the strongly-justified host surface; no host-specific design is needed in Notes.

### Concrete changes

| File | Change |
|---|---|
| `src/notes/window/panels.rs` | `open_browse_panel`: delete the `day_page_rows` load + `extend(get_day_page_switcher_actions(...))` (lines ~397-413); simplify the `info!` log (416-420, references `day_page_rows.len()`). `execute_note_switcher_action`: delete the entire `daypage_` branch (308-334). |
| `src/notes/day_page_rows.rs` | **Delete file** (loader, hook, `day_page_rows_tests`). The main switcher's `load_day_switcher_entries` is the surviving duplicate. |
| `src/notes/mod.rs:38` | Remove `pub(crate) mod day_page_rows;`. |
| `src/actions/builders/notes.rs` | Delete `DayPageSwitcherInfo` (934-940), `get_day_page_switcher_actions` (944-963), and its test (375-398). |
| `src/actions/builders.rs:53` / `src/actions/mod.rs:45,49` | Remove `get_day_page_switcher_actions` + `DayPageSwitcherInfo` re-exports (compiler enforces no stragglers). |
| `src/app_impl/startup.rs:255-259` | Remove `register_open_day_page_in_main_hook(...)` registration block. |
| `src/main_sections/day_page_view.rs:706-742` | Remove `open_day_page_in_main_window_hook` method. |

### Tests (compiler/behavior first, per AGENTS.md ladder)

- **Rung 1 (compiler, strongest):** deleting the builder + re-exports + `daypage_` match arm makes it impossible to produce or dispatch a `daypage_` id or to call the main-window hook from Notes. No source-audit test needed — the import deletion *is* the enforcement.
- **Rung 3 (behavior, add):** in `src/actions/builders/notes.rs::tests`, add `note_switcher_actions_never_contain_day_page_section_or_ids` — calls the real `get_note_switcher_actions` with sample notes and asserts no action has `section == Some("Day Pages")` and no `id.starts_with("daypage_")`. Regression guard against re-introduction; exercises the actual builder output, not source text.
- **Delete:** `test_get_day_page_switcher_actions_builds_daypage_ids_in_day_pages_section` (builders/notes.rs:375) and the whole `day_page_rows_tests` module — both test deleted code.
- **Do NOT add** a source-audit asserting "open_browse_panel doesn't call get_day_page_switcher_actions" — that is exactly the discouraged pattern and the compiler already nails it.

### Runtime proof (rung 4)

DevTools probe under `scripts/agentic/` (or via the `agy-script-kit-devtools` skill): trigger the `SearchNotes` builtin → open Cmd+P → enumerate switcher rows through the automation element surface → assert (a) zero rows with `daypage_*` semantic ids / "Day Pages" section, and (b) `is_main_window_visible()` stays `false` after picking any note. (b) is the real contract proof — no Notes-Cmd+P result ever activates the main window.

### Verification commands

```
./scripts/agentic/agent-cargo.sh check --lib
./scripts/agentic/agent-cargo.sh test --lib actions::builders::notes::tests
./scripts/agentic/agent-cargo.sh test --test actions
```

## Self Score

**8.5/10.** I grounded every claim in the actual call graph (read panels.rs, day_page_rows.rs, day_page_switcher.rs, day_page_view.rs hook, startup registration, actions builder, keyboard router), identified six distinct failure modes beyond the core violation (including a latency hit and a hidden empty-state bug), and picked the compiler as the primary enforcer rather than minting a source-audit. Deductions: I did not run a `#[gpui::test]` to confirm `open_browse_panel`'s post-refactor action set at runtime (deferred to the devtools probe recommendation), and I did not exhaustively rule out a config flag gating day-page rows (none found in `config/types.rs` grep, but I didn't open the full config walkthrough).
