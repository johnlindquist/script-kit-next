Implemented the logging/observability changes across logging-related Rust code only.

Changed files:
- `src/logging.rs`
- `src/stdin_commands.rs`
- `src/hotkeys.rs`
- `src/main.rs`
- `src/hotkey_pollers.rs`
- `src/execute_script.rs`
- `src/mcp_server.rs`

What changed:
- Added/expanded structured compact-category mapping and target inference for AI compact mode.
- Added legacy category→severity mapping and made `log(...)` emit correct `tracing` level instead of flattening to info.
- Added correlation-aware stdin command envelope and command metadata helpers.
- Replaced raw payload/debug dumps with summarized structured logging fields for stdin/protocol/MCP request paths.
- Propagated correlation IDs through hotkey event channels and restored correlation context at event consumers.
- Updated affected tests in touched modules to reflect new behavior.

Verification run:
- Ran `cargo check` in the shared tree.
- Current failure is unrelated to logging changes and in another agent’s scope:
  - `src/components/unified_list_item/render.rs:374`
  - `error[E0027]: pattern does not mention field 'fragments'`

How to test after that blocker is fixed:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Runtime compact-log smoke test:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
5. JSONL verification:
   - `tail -n 100 ~/.scriptkit/logs/script-kit-gpui.jsonl | rg 'correlation_id|stdin|hotkey|mcp'`

Risks / known gaps:
- Shared-tree compile blocker prevented full end-to-end verification in this run.
- No additional commit was made from this agent in this turn.

Commits:
- None.