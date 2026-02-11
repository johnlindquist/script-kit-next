# Stdin Protocol: Add A Command

## When to use

- You need a new external JSONL command sent to app stdin for
  testing or automation.
- You need to route a new `"type"` payload to a `ScriptListApp`
  state update.
- You need deterministic UI updates after protocol-driven mutations.

## Do not do

- Do not add CLI args for this flow; this app is stdin JSONL-driven.
- Do not add a new enum variant without wiring a runtime `match` arm.
- Do not mutate render-affecting state without ending in `ctx.notify()`.
- Do not skip deserialization tests for the new command payload.

## Full path (JSONL sender -> parse -> state update -> render)

1. Sender writes one JSON object per line to app stdin (JSONL), for
   example `{"type":"setFilter","text":"notes"}`.
2. Rust listener reads lines in `src/stdin_commands/part_001.rs:14`
   (`start_stdin_listener`).
3. Listener parses with serde at `src/stdin_commands/part_001.rs:43`
   (`serde_json::from_str::<ExternalCommand>(...)`).
4. Parsed commands are sent as `ExternalCommandEnvelope` over channel
   at `src/stdin_commands/part_001.rs:63`.
5. Main runtime receives commands in
   `src/main_entry/app_run_setup.rs:1262`, then dispatches in
   `match cmd` at `src/main_entry/app_run_setup.rs:1282`.
6. Each branch mutates app/window state (`view.*`, `current_view`,
   focus, caches, etc.).
7. Dispatch ends with `ctx.notify();` at
   `src/main_entry/app_run_setup.rs:1975`, which triggers GPUI
   re-render on the next event loop turn.

## Files to edit when adding a new stdin command type

- `src/stdin_commands/part_000.rs:306`: Add the new `ExternalCommand`
  enum variant (`#[serde(tag = "type", rename_all = "camelCase",
  deny_unknown_fields)]`).
- `src/stdin_commands/part_000.rs:424`: Extend `command_type()` so
  logs/reporting include the new command name.
- `src/stdin_commands/part_000.rs:412` (if needed): If the command
  supports `requestId`, update `request_id()` accessor.
- `src/main_entry/app_run_setup.rs:1282`: Add a `match` arm for the
  new variant and perform state update logic.
- `src/main_entry/app_run_setup.rs:1975`: Keep the trailing
  `ctx.notify()` intact (or notify inside an extracted helper).
- `src/stdin_commands/part_002.rs:46`: Add deserialization tests for
  valid payloads and relevant failure cases.

## Canonical files

- `src/stdin_commands/part_000.rs:304`: command schema and serde tag.
- `src/stdin_commands/part_001.rs:14`: stdin listener parse pipeline.
- `src/main.rs:309`: runtime entry includes
  `main_entry/app_run_setup.rs`.
- `src/main_entry/app_run_setup.rs:1204`: command receiver wiring.
- `src/main_entry/app_run_setup.rs:1282`: command dispatch `match`.
- `src/main_entry/app_run_setup.rs:1975`: final `ctx.notify()`.
- `src/stdin_commands/part_002.rs:45`: protocol unit tests for JSON.

## Minimal snippet

```rust
// src/stdin_commands/part_000.rs
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ExternalCommand {
    // ...
    SetBadge { text: String },
}

impl ExternalCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            // ...
            Self::SetBadge { .. } => "setBadge",
        }
    }
}

// src/main_entry/app_run_setup.rs (inside match cmd)
ExternalCommand::SetBadge { text } => {
    view.filter_text = text;
}
// keep trailing ctx.notify();
```
