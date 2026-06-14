## Role Findings

Likely owner is [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:300). The current Notes Cmd+P path explicitly supports `daypage_YYYY-MM-DD` action IDs and opens the main Day Page via `open_day_page_in_main`.

The implementation seam is two-part:

1. Remove day-page row injection from `NotesApp::open_browse_panel` at [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:397). Cmd+P in Notes should only call `get_note_switcher_actions(...)`.
2. Remove or fail-close the `daypage_` activation branch in `execute_note_switcher_action` at [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs:308). This prevents stale popups, direct protocol actions, or old state from crossing into Day Page.

After that, delete the now-owned crossover machinery if no references remain: [src/notes/day_page_rows.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/day_page_rows.rs:1), exports from `src/notes/mod.rs`, `DayPageSwitcherInfo` / `get_day_page_switcher_actions` in [src/actions/builders/notes.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders/notes.rs:931), re-exports in `src/actions/{builders.rs,mod.rs}`, and the startup hook registration at [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs:255).

## Evidence And Assumptions

Current behavior is intentional in code but conflicts with the new product contract: Notes “lists” day pages and “hands a pick off to the main window” in [src/notes/day_page_rows.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/day_page_rows.rs:1).

Day Page already owns its own past-day switcher: [src/main_sections/day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:109) opens it, and [src/main_sections/day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:128) binds a selected day in-place. So removing Notes access does not remove day-note editing; it restores the boundary.

DevTools can verify this structurally because Notes exposes command bar state through [src/actions/command_bar.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/command_bar.rs:629), and visible action samples through [src/actions/dialog.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/dialog.rs:2675).

## Failure Modes

If only row construction is removed, a stale `daypage_...` action or direct automation activation can still open Day Page through the existing activation branch.

If only activation is removed, Notes Cmd+P still shows “Day Pages” rows that do nothing, which is worse UX and leaves the conceptual crossover visible.

If the hook/helper files remain, future code can accidentally reattach the bridge. Given `rg` shows these symbols are only used by this crossover, deleting them is the cleaner narrow change.

## Recommendation

Implement the fix as a boundary removal, not a redirect:

- In `open_browse_panel`, stop loading `notes_brain_days_dir()` and stop extending `note_switcher_actions` with day rows.
- In `execute_note_switcher_action`, remove the `daypage_` branch or change it to warn + close without opening main Day Page. I prefer deleting the branch and letting the unknown-action path close, unless you want a clearer log like `notes_note_switcher_day_page_action_ignored`.
- Delete the dedicated Notes day-page row module and action-builder helper after `rg` confirms no remaining uses.
- Leave `src/main_sections/day_page_switcher.rs` untouched. That is the correct owner for selecting past day notes.

Focused verification:

```bash
SCRIPT_KIT_AGENT_ARTIFACT_NAME=notes-no-day-crossover \
  ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
```

Then invert the existing proof in `scripts/agentic/notes-day-page-switcher-probe.ts`: seed sandbox `brain/days/*.md`, open Notes, send target-scoped Cmd+P, and assert:

- Notes `commandBars.noteSwitcher.configuredSections` has no `Day Pages`.
- Actions dialog visible/action samples contain no IDs starting with `daypage_`.
- Pressing/filtering cannot make the main state become `promptType === "dayPage"` from the Notes switcher path.
- Separately open Day Page with `scripts/agentic/day-page-open-helper.ts`, send Cmd+P there, and assert day switcher rows still appear under the Day Page surface.

Run it with:

```bash
PROBE_BINARY=target-agent/artifacts/notes-no-day-crossover/script-kit-gpui \
  bun scripts/agentic/notes-day-page-switcher-probe.ts
```

## Self Score

8/10. The owner seam is clear and the verification path maps to existing DevTools. The only uncertainty is whether you want to keep an explicit ignored `daypage_` log for stale activations or simply delete the branch and rely on the unknown-action path.
