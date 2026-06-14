**Recommendation**

For the current checkout, this looks already implemented at the right seam.

Likely owner is [src/notes/window/panels.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/panels.rs), specifically:

- `NotesApp::open_browse_panel`: should populate the Notes Cmd+P picker only from `get_note_switcher_actions(...)` over `self.notes`.
- `NotesApp::execute_note_switcher_action`: should be the fail-closed dispatcher guard.

The important guard is the `daypage_` branch before normal `note_` handling. In current source, it reportedly closes the browse panel, logs `notes_note_switcher_day_page_action_ignored`, and returns. That is the right behavior: Notes should not redirect into the main-window `day` view, even if a stale or injected action ID reaches the dispatcher.

Keyboard Enter is also covered if it routes through `execute_note_switcher_action`, so this seam protects both mouse/activation callbacks and selected-row keyboard activation.

**If Targeting An Older Branch**

Make only these narrow edits:

1. In `open_browse_panel`, do not add Day Page rows to the Notes picker. It should only show real note switcher actions from the Notes window’s note set.
2. In `execute_note_switcher_action`, keep/add a fail-closed branch before `note_` handling:

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

Do not touch the main Day Page switcher unless verification proves it is involved. Also avoid stale cleanup advice around `src/notes/day_page_rows.rs` or `get_day_page_switcher_actions`; those claims do not match the current tree.

**Verification**

Use the existing runtime proof first, not a new source-audit test:

```bash
SCRIPT_KIT_AGENT_ARTIFACT_NAME=notes-day-guard \
  ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
```

Then:

```bash
PROBE_BINARY=target-agent/artifacts/notes-day-guard/script-kit-gpui \
  bun scripts/agentic/notes-day-page-switcher-probe.ts
```

Expected proof from the probe:

- Notes Cmd+P opens the Notes switcher.
- No visible rows/action IDs start with `daypage_`.
- No `Day Pages` section appears.
- Seeded day files do not leak into the Notes picker.
- No day-page handoff log appears.
- Main window `promptType` remains not `dayPage`.

A source-structural test is only a fallback if the runtime probe cannot express the invariant cheaply; AGENTS.md makes source audits the last rung here.
