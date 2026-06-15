## Role Findings

The safest shape is to make Notes Cmd+P the shared implementation and make Day Page a second host of that same `CommandBar`/action-builder contract. Do not restyle Day’s bespoke overlay. Delete or bypass it.

Concretely:

1. Add a shared pure data/action layer around [src/actions/builders/notes.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders/notes.rs:755):
   - Keep `NoteSwitcherNoteInfo` and `get_note_switcher_actions(&[NoteSwitcherNoteInfo])` for compatibility.
   - Add `NoteSwitcherDayInfo { date, title, char_count, preview, relative_time, is_current }`.
   - Add `get_shared_note_search_actions(notes, days, host_options)` or similar.
   - Encode action IDs distinctly:
     - `note_{uuid}` for normal notes.
     - `daynote_{YYYY-MM-DD}` for day-note selection.
     - `daynote_open_in_notes_{YYYY-MM-DD}` for the explicit Day Page escape hatch.

2. Move day-note discovery out of Day’s local UI path:
   - Reuse the filesystem logic currently in [src/main_sections/day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:24), but return shared `NoteSwitcherDayInfo`.
   - Put it in a neutral module, for example `src/notes/note_search_items.rs` or `src/brain/day_notes.rs`.
   - Both Notes and Day Page should call the same loader so Notes search sees `brain/days/YYYY-MM-DD.md`.

3. Host routing should be explicit and local:
   - Notes host:
     - `note_{uuid}` keeps current `select_note`.
     - `daynote_{date}` opens that day file inside the Notes editor/window.
   - Day Page host:
     - `daynote_{date}` calls `bind_day(date)`, preserving the main-window Day editor.
     - `note_{uuid}` only opens inside Day Page if the product truly wants non-day notes there; based on the request, I would include all day notes in both surfaces and leave regular Notes rows unchanged for Notes unless current product language says Day Cmd+P should search every note type.
     - `daynote_open_in_notes_{date}` is the only route from Day Page to the floating Notes window.

4. Replace Day’s local switcher state:
   - Remove `DaySwitcherState` from [src/main_sections/day_page_types.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_types.rs:59).
   - Remove `render_day_page_day_switcher_panel` usage from [src/main_sections/day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:417).
   - Replace `open_day_switcher` with `open_day_note_search` backed by `CommandBar::new(..., CommandBarConfig::notes_recent_style())`, mirroring Notes [open_browse_panel](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:360).

## Evidence And Assumptions

Notes already uses the desired shared container: `CommandBar`, `get_note_switcher_actions`, detached activation wiring, and `notes_recent_style` placeholder cleanup in [panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:388).

Day Page currently owns a separate UI/input/filter stack: `DaySwitcherState`, `load_day_switcher_entries`, `handle_day_switcher_key`, and `render_day_page_day_switcher_panel` in [day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:15). That is the divergence to remove.

The prior Notes guard is now obsolete: [execute_note_switcher_action](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:308) explicitly ignores `daypage_` actions. Remove or rewrite it into a positive `daynote_` route.

The current probe is inverted against the new requirement: [notes-day-page-switcher-probe.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/agentic/notes-day-page-switcher-probe.ts:1) asserts Notes Cmd+P must not expose day rows.

Assumption: “Notes search should have access to all day notes” means day-page files under `brain/days/*.md`, not necessarily all regular Notes plus all Day Page plus fragments. I would name the model “day note” rather than “day page” in result language.

## Failure Modes

- Reusing `daypage_` IDs will keep dragging the old no-crossover meaning through the code. Prefer new `daynote_` IDs and delete the stale guard.
- Adding fields to `NoteSwitcherNoteInfo` will churn many existing action-builder tests. Add a parallel day-info struct or wrapper function instead.
- Making Day Page call Notes window APIs on normal selection would violate the key behavioral requirement. Day Page selection should bind locally by default.
- Keeping Day’s overlay “temporarily” risks two UI contracts surviving. The request is specifically same UI/container/component/result language, so the bespoke Day overlay should go.
- Runtime probes that only check row presence are insufficient; they must also prove host-local routing after selection.

## Recommendation

Implement in three narrow slices:

1. Shared action/data model:
   - Add day-note info type and shared action builder in `src/actions/builders/notes.rs`.
   - Add pure tests proving regular notes and day notes share sections/icons/description language and action IDs.

2. Host integration:
   - Notes `open_browse_panel` should include `load_day_note_infos(...)`.
   - Notes `execute_note_switcher_action` should route `daynote_{date}` to a Notes-window day-file open path.
   - Day Page should hold a `CommandBar` for search instead of `DaySwitcherState`; `day_page:open_past_day` at [day_page_actions.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_actions.rs:271) should open the shared search.
   - Day Page activation for `daynote_{date}` calls existing `bind_day(date)`.

3. Cleanup/docs/probes:
   - Remove `scripts/agentic/notes-day-page-switcher-probe.ts` or rewrite it as `shared-day-note-search-probe.ts`.
   - Update `day-page-today-feature-probe.ts` where it expects `list:day-page-day-switch`.
   - Update `GLOSSARY.md` to document shared note search with host-local default binding.
   - Remove comments saying Notes must not expose day pages.

Smallest useful verification:
- `./scripts/agentic/agent-cargo.sh test --lib actions::builders::notes`
- A focused source/test check for no remaining `daypage_` stale guard in Notes.
- Runtime probe:
  - Seed `brain/days/2026-06-01.md`.
  - Open Notes Cmd+P, verify row appears, select it, verify Notes editor/window opens that file.
  - Open Day Page Cmd+P, verify same row language/container, select it, verify Day Page editor rebinds.
  - From Day Page, execute explicit “Open in Notes Window” action and verify only that path opens Notes.

## Self Score

8/10. The architecture is clear from current source, but I did not inspect the exact Notes data model for opening arbitrary markdown day files inside the Notes window, so that part may need a small adapter design after implementation source review.
