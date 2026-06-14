• ## Role Findings

  The "cross-over" is **already prevented** in the current tree, but only by a single silent guard with no regression test or user feedback.

  - **Owner file:** `src/notes/window/panels.rs`
  - **Implementation seam:** `NotesApp::execute_note_switcher_action` (the routing sink for the Notes window Cmd+P `note_switcher`)
  - **Existing guard:** lines 311-319 ignore any `daypage_YYYY-MM-DD` action id, close the browse panel, log `notes_note_switcher_day_page_action_ignored`, and return.
  - **Current picker contents:** `open_browse_panel` in the same file only feeds `get_note_switcher_actions(&notes)` (UUID-based notes). No `Day Pages` section or `daypage_` rows are currently generated anywhere in `src/` for the Notes switcher. The guard is therefore a fail-safe against stale state or future regressions rather than active daily traffic.
  - **No tests:** No checked-in test covers the guard, the absence of day rows in the Notes switcher, or the fact that selecting a `daypage_` action never reaches the main-window `DayPageView`.

  ## Evidence And Assumptions

  Evidence from the repo:

  - `src/notes/window/panels.rs:311-319` — the only `daypage_` reference in source:
    ```rust
    if let Some(date_str) = action_id.strip_prefix("daypage_") {
        self.close_browse_panel(window, cx);
        tracing::info!(
            target: "script_kit::notes",
            event = "notes_note_switcher_day_page_action_ignored",
            date = %date_str,
        );
        return;
    }
    ```
  - `src/notes/window/init.rs:110-127` and `src/notes/window/panels.rs:362-388` — `note_switcher` is initialized and refreshed exclusively from `get_note_switcher_actions(notes)`, which emits only `note_{uuid}` and `no_notes` ids.
  - `src/actions/builders/notes.rs` — no `daypage_`/`DayPage` row builder is called for the Notes switcher.
  - `src/notes/window/keyboard.rs:491-496` — Enter in the switcher routes through `execute_note_switcher_action`, so the guard covers both mouse/activation-callback and keyboard paths.
  - `src/notes/day_page_rows.rs` does not exist, and `register_open_day_page_in_main_hook`/`open_day_page_in_main` are not referenced anywhere in `src/`.

  Assumptions:

  - "Day note" means the main-window `DayPageView` (`AppView::DayPage`), not a markdown file physically stored in the notes directory.
  - The desired behavior is: Notes window stays the Notes window; Cmd+P must not cause the main launcher to switch to the day-page surface.
  - The project wants the narrowest change, not a larger refactor of note/day storage.

  ## Failure Modes

  1. **Silent UX failure.** If a `daypage_` row ever appears (stale cache, future feature, protocol injection), selecting it does nothing and gives no feedback. The user may think the app froze or the shortcut is broken.
  2. **Regression by removal.** The guard is unguarded by tests. A future refactor that deletes it or moves routing into `handle_action` would silently re-enable cross-over.
  3. **Prefix drift.** The check is exactly `strip_prefix("daypage_")`. If a future builder uses `day_`, `today_`, or a different casing, the guard misses it.
  4. **Action-id collision (theoretical).** A regular note UUID could theoretically start with `daypage_` (UUIDs are hex, so impossible in practice), but the `note_`/`daypage_` namespaces are distinct by convention, not by a type system.
  5. **Mention-portal path.** The guard runs before `replace_active_note_mention_with_note`. That is correct (a day page should not be inserted as a note mention), but if the two paths are reordered during a refactor, day content could be pasted into a note body.
  6. **Protocol/automation bypass.** A DevTools or MCP command could send a `daypage_` action directly to the Notes entity. The guard catches it, but without a test there is no proof it stays caught.
  7. **Stale popup state across rebuilds.** `CommandBar` caches actions; `open_browse_panel` calls `set_actions`, which replaces them. This is fine today, but any change that merges actions instead of replacing them could resurrect day rows.

  ## Recommendation

  Keep the existing seam and add a regression lock plus a DevTools runtime proof. Do not widen the change into the day-page storage model or the main launcher routing.

  1. **Keep the guard** at `src/notes/window/panels.rs::execute_note_switcher_action`. It is the correct last-line-of-defense seam.
  2. **Add a focused regression test.** Direct `Window`/`Context` unit testing is not present in `src/notes`, so add a source-structural test in `src/notes/window/tests.rs` (consistent with the existing keyboard-branch source audits in that file) that asserts the body of `execute_note_switcher_action` contains the `daypage_` strip-prefix check and returns before any note-selection or main-window handoff logic. This is a justified last-resort source audit because the invariant is "cross-window navigation must not occur," which no compiler or unit test in this codebase can currently express.
  3. **(Optional UX)** When the guard fires, show a brief `show_action_feedback("Day notes open in the main window", false)` so the silent no-op is not silent. This is a one-line addition inside the existing guard.
  4. **DevTools verification** (red/green):
     - Build: `SCRIPT_KIT_AGENT_ARTIFACT_NAME=notes-day-guard ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`
     - Run a small driver probe that:
       - `Driver.launch({ binary: "target-agent/artifacts/notes-day-guard/script-kit-gpui", sandboxHome: true })`
       - Opens the Notes window via protocol.
       - Sends `simulateKey("p", ["cmd"])` to open the note switcher.
       - Calls `getElements(target notes)` and asserts no semantic id starts with `daypage_` and no section label is `"Day Pages"`.
       - For the guard itself, the source-structural test plus a one-off injected-action check (if you have a test build that temporarily adds a `daypage_` row) proves selecting it does not switch the main window to `AppView::DayPage`.
     - Existing test suite: `./scripts/agentic/agent-cargo.sh test --lib notes::window::tests`

  Why this is narrow: the change stays inside `src/notes/window/panels.rs` (plus one test file) and does not touch the day-page model, the main launcher, or the shared `NotesEditor`.

  ## Self Score

  8/10. The seam, owner files, current state, and failure modes are clearly identified, and the verification path uses the project's own `script-kit-devtools` driver. I did not run the actual DevTools probe or compile a test build, so the score stops short of a fully executed red/green proof.

