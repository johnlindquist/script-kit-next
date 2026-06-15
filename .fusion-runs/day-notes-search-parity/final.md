**Recommended Shape**

Treat this as convergence around the existing shared Notes-style switcher, not a fresh Day Cmd+P implementation. Current source apparently already has partial convergence: Day Page has `note_switcher` / `open_note_switcher` wiring and `day_page:open_past_day` already opens the shared note switcher, while the old bespoke `DaySwitcherState` render/key path still exists. So the safe move is to finish the convergence, remove stale paths only after checking references, and make host-local routing explicit.

1. Share the search rows and action builder around day notes.
   Reconcile the two existing day-note modeling paths:

   - `src/actions/builders/notes.rs` / `get_day_note_switcher_actions`
   - `src/notes/day_switcher.rs` with `day:` IDs

   Keep one neutral day-note model, for example `DayNoteSwitcherInfo { date, title, preview, is_today, ... }`, and have both Notes and Day Page use the same loader and same row/action language. Do not claim `daynote_` is safer by default: current source already uses `daypage_` in shared actions and Day Page routing, so either keep `daypage_{date}` if it is already wired, or migrate to `daynote_{date}` only with a deliberate source-backed cleanup of all old guards, probes, logs, and tests.

2. Use one container/component.
   Notes should keep its existing `CommandBar` path in `src/notes/window/panels.rs::open_browse_panel`.

   Day Page should use that same `CommandBar` style and action/result language. Since current source reportedly already has `DayPageView.note_switcher`, `open_note_switcher`, and `day_page:open_past_day` opening the shared switcher, do not implement a new CommandBar from scratch. Instead:

   - make `day_page:open_past_day` definitively use the shared note switcher;
   - stop rendering or invoking the bespoke `src/main_sections/day_page_switcher.rs` overlay once all references are gone;
   - only delete `DaySwitcherState`, `render_day_page_day_switcher_panel`, custom key handling, etc. after `rg` confirms no active fallback/reference remains.

3. Route selection by host.
   This is the core invariant:

   - Notes selecting a day note opens that day file in the Notes editor/window.
   - Day Page selecting the same day note calls `bind_day(date)` and rebinds the Day Page editor.
   - Day Page opens a note in the Notes window only through an explicit action such as `Open in Notes Window`.

   The stale Notes branch that currently ignores `daypage_` rows must flip from “forbidden crossover” to positive local Notes handling. Important: verify Notes persistence through `active_day_binding`, `select_day_note`, and `save_current_note` before assuming edits to `brain/days/YYYY-MM-DD.md` are safely saved from the Notes editor.

4. Keep Day Page search scoped unless source proves more.
   The request is clear about all day notes being visible in both surfaces. It does not require the Day Page editor to host arbitrary regular notes. So the safest first implementation is shared day-note search in both hosts. If regular `note_{uuid}` rows appear in Day Page search today, route them only if the Day Page editor already has a supported binding model for them; otherwise omit them from Day Page search or expose an explicit Notes-window action.

5. Remove/rewrite stale artifacts.
   Rewrite or remove anything enforcing the old no-crossover contract:

   - `src/notes/window/panels.rs` branch/comment that logs or ignores `daypage_`
   - `scripts/agentic/notes-day-page-switcher-probe.ts`, which should now assert day rows are present and route locally
   - any docs/comments mentioning Notes Cmd+P must not expose day rows
   - `GLOSSARY.md` should document the new paradigm: shared note search UI/container/result language; only default binding differs between Day Page and Notes

   Search before editing: `daypage_`, `day:`, `notes-day-page-switcher`, `no crossover`, `open_past_day`, `DaySwitcherState`.

**Verification Plan**

Smallest useful proof:

- Unit test shared day-note action building: same row/action language emitted for day notes.
- Focused routing tests:
  - Notes action for day note selects/opens in Notes editor and sets/verifies day-file binding.
  - Day Page action for day note calls/reaches `bind_day(date)` / bound date changes.
- Run through cargo wrapper only:
  ```bash
  ./scripts/agentic/agent-cargo.sh test --lib actions::builders::notes
  ```
  plus the narrow module tests touched by Notes/Day routing.
- Rewrite the runtime probe to seed a day file, open both Cmd+P surfaces, verify the same day row appears, select it in each host, and confirm:
  - Notes selection stays in Notes editor/window.
  - Day Page selection rebinds the Day Page editor.
  - Notes window opens from Day Page only through explicit `Open in Notes Window`.

Confidence is medium because the panel evidence shows partial current-source convergence, but Notes day-file persistence still needs direct source verification before editing.
