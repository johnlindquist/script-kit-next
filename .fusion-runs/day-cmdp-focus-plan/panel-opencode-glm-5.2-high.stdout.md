I'll explore the relevant source files in parallel to ground my plan in the current code before producing the pragmatist analysis.
Now let me examine the key entry paths in `day_page_view.rs` and the actions dialog close path for the Escape bug.
Let me read the key routing in `day_page_view.rs`, the `command_bar_key_intent` helper, and the `close_actions_popup` function.
I need to understand the critical architectural question: is the Day switcher an inline overlay or a detached `CommandBar` window? Let me check the field wiring and render path.
Let me confirm the critical architecture question — whether the `CommandBar` detached dialog handles keys internally or relies on host routing — by reading `open_centered`, the dialog key handler, and how Notes routes keys to its switcher.
The architecture is now clear: Notes ALSO routes keys through its host `handle_key_down` (not just the detached window). Let me read the Notes keyboard switcher routing to compare directly with Day's `handle_day_switcher_key`, plus the `ActionsWindow` key handler.
I have enough evidence to produce the plan. Let me verify one final claim — that `MainList` focus restore hits `main_filter()` (wrong target for Day) and confirm there's no Day-specific branch in the actions close path.
## Role Findings

The smallest fix touches **two narrow spots**; everything else is convergence/risk-flagging.

**Bug 1 — Day Cmd+P keys are swallowed by the editor (typing/arrows/escape all dead).**
`open_note_switcher` (`src/main_sections/day_page_switcher.rs:113`) opens the detached `CommandBar` via `open_centered` but **never moves focus off the Day Page editor**. The file's own header comment (`day_page_switcher.rs:6-7`) states the design contract: *"While open, focus moves from the editor to the Day Page root focus handle so typed characters filter the switcher."* That move is missing. Result: after Cmd+P (a global binding, so it still fires), the embedded editor retains focus and eats every subsequent keystroke before the root `on_key_down` (`day_page_view.rs:581`) can route to `handle_day_switcher_key`. This single omission explains why typing, arrows, **and** escape all fail simultaneously — and why focus isn't "restored" (it never left the editor). Notes avoids this because `open_browse_panel` calls `request_focus_surface(NotesFocusSurface::BrowsePanel, …)` (`src/notes/window/panels.rs`).

**Bug 2 — Day Actions Escape focuses the wrong target.**
`actions_host_for_view` maps `AppView::DayPage` → `ActionsDialogHost::MainList` (`src/app_impl/actions_dialog.rs:39`). `request_focus_restore_for_actions_host` then maps `MainList` → `FocusRequest::main_filter()` (`actions_dialog.rs:1203-1213`) — the **script-list search input**, not the Day editor. So Escape closes Actions but dumps focus into the main search box.

## Evidence And Assumptions

- `day_switcher: Option<DaySwitcherState>` is initialized `None` (`day_page_view.rs:58`) and only ever reset to `None` (`day_page_switcher.rs:255`); it is **never set to `Some`**. So `render_day_page_day_switcher_panel` (gated on `self.day_switcher.clone()?`, line 376) is dead for rendering, and `day_page_spine.rs:60` reads an always-`None` field (secondary spine/automation bug — out of scope). The only visible popup is the **detached `ActionsWindow`** from `CommandBar::open_centered`.
- Both Notes and Day use the identical `CommandBar` + detached `ActionsWindow`. The detached window self-handles keys (`src/actions/window.rs:932` → `command_bar_key_intent`), and Notes *additionally* host-routes (`src/notes/window/keyboard.rs:467`, a source-audited branch). Day's `handle_day_switcher_key` is a **parallel reimplementation** that never gets the chance to run because focus never reaches the root.
- `wire_note_switcher_activation`'s `on_close` already calls `view.focus_editor(window, cx)` (`day_page_switcher.rs:178`) — so the **restore path for the switcher's own close is already correct**; only the *open* focus move is missing.
- Assumption to confirm at runtime: the detached ActionsWindow for Day is **not** becoming the key window (main window retains key). If devtools shows the detached window *is* key and keys still fail, the fix shifts to the dialog's `handle_key_event` — but the editor-swallow hypothesis is far more consistent with "all three keys dead."

## Failure Modes

- **Editor retains focus** → keys edit the doc / are swallowed; router never fires (the actual Bug 1).
- **MainList→main_filter mapping** → Actions Escape lands on script search, not Day editor (Bug 2).
- **Dual routing drift**: detached-window self-handle + host `handle_day_switcher_key` can diverge; any future edit that wires the dead inline panel (`render_day_page_day_switcher_panel`) would double-render.
- `close_day_switcher` sets `self.day_switcher = None` (already None) — harmless today, but signals the field is vestigial and misleading.
- `on_close` runs inside `cx.defer` + `day_window.update`; if the window handle is stale the restore silently no-ops (`day_page_switcher.rs:172-181`) — low risk, matches Notes.

## Recommendation

**1. Owner functions to patch (2 edits):**
- `DayPageView::open_note_switcher` — `src/main_sections/day_page_switcher.rs:113`: after `self.note_switcher.open_centered(window, cx)`, **focus the Day Page root handle** (`window.focus(&self.focus_handle, cx)`) so the editor stops swallowing keys, satisfying the documented contract at lines 6-7. Do *not* touch `wire_note_switcher_activation`'s `on_close` (already restores editor).
- `ScriptListApp::request_focus_restore_for_actions_host` (or `close_actions_popup`) — `src/app_impl/actions_dialog.rs:1190`: when `host == MainList` **and** `self.current_view` is `AppView::DayPage`, focus the Day Page editor entity (mirror `theme_focus.rs:308` / `registries_state.rs:268` which already resolve the DayPage entity) instead of `FocusRequest::main_filter()`.

**2. Reuse `CommandBar` routing directly — do NOT keep expanding the local wrapper.**
Day already uses the same `CommandBar` + detached `ActionsWindow` as Notes. The local `handle_day_switcher_key` should remain only as the **host-side fallback** (mirroring Notes `keyboard.rs:467`) for the case where the main window stays key; it must not grow further. The pragma here: do **not** extract a new shared router mid-fix on a dirty tree — that's scope creep. Just (a) fix the open-time focus move so the existing router actually runs, and (b) verify it matches Notes' branch char-for-char for the printable path (Notes uses `printable_char(key_char)`; Day uses `key.chars().next()` — accept this gap for now unless devtools shows real chars failing).

**3. Focus-restoration path:**
- **Day Cmd+P close:** already correct via `on_close` → `mark_closed_externally` + `focus_editor` (`day_page_switcher.rs:169-181`). No change beyond the open-side fix.
- **Day Actions Escape:** add the DayPage branch in `request_focus_restore_for_actions_host` so it targets the Day editor entity, not `main_filter()`.

**4. Verification (script-kit-devtools, cheapest first):**
- *Probe A (Bug 1):* focus the main window, open Day Page, fire Cmd+P, dump `document.activeElement`/automation focus before typing; send `simulate-key "a"`, `"down"`, `"escape"`; assert query updates, selection moves, popup closes, and post-close focus === Day editor.
- *Probe B (Bug 2):* on Day Page, fire Cmd+K, then `simulate-key "escape"`; assert focused element is the Day editor entity (not the main search input).
- *Source/behavior test (one, narrowly scoped):* a behavior test asserting `request_focus_restore_for_actions_host(MainList)` with `current_view == DayPage` does **not** enqueue `FocusRequest::main_filter()` (assert the Day editor entity is targeted). Do not mint a formatted-line source-audit for the focus move — prefer the devtools runtime proof.

**5. Risks in the current design:**
- Vestigial `day_switcher` field + `render_day_page_day_switcher_panel` + `accept_day_switcher_selection` invite a future double-render; flag, don't remove now.
- `day_page_spine.rs:60` reads the never-set field → spine/devtools under-reports; separate bug.
- Two key routers (detached window + host) will re-break if they drift; the open-time focus move is what keeps the host router alive.
- Day Page doesn't participate in `NotesFocusSurface`; the main-window `FocusRequest` enum has no Day-editor variant, so the Actions-fix must special-case via `current_view` — acceptable, but note it as design debt.

## Self Score

**8/10.** High confidence on Bug 2 (mapping is unambiguous in source). Bug 1's editor-swallow hypothesis is the strongest fit for "all three keys dead + focus not restored" and is directly testable, but I have not yet runtime-confirmed that the detached window isn't key — a devtools Probe A run could shift the fix from "focus the root on open" to "ensure the detached window takes key." Plan stays valid either way; the restore side and Actions fix are certain. No code changed; this is plan-only, consistent with the dirty-tree constraint.
