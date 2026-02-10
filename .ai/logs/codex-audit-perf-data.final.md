**Audit Findings**
1. Fixed: `parse_message_graceful` now uses a fast path (`serde_json::from_str::<Message>`) and only falls back to `Value` parsing on errors, reducing hot-path deserialize overhead in protocol handling (`src/protocol/io/parsing.rs:186`).
2. Fixed: `JsonlReader` now decodes into a reusable buffer and avoids the extra `String` copy between read and parse stages (`src/protocol/io/reader.rs:20`, `src/protocol/io/reader.rs:84`, `src/protocol/io/reader.rs:104`).
3. Fixed: protocol message/response debug summaries in execute loop are now behind `tracing::enabled!(DEBUG)`, so normal runs avoid unnecessary `serde_json::to_string` work (`src/execute_script/part_001_body/execute_interactive_merged.rs:296`, `src/execute_script/part_001_body/execute_interactive_merged.rs:312`).
4. Remaining risk (not changed): app/window fuzzy search still clones full structs per match (`src/scripts/search/apps.rs:20`, `src/scripts/search/apps.rs:89`, `src/scripts/search/windows.rs:25`, `src/scripts/search/windows.rs:86`).
5. Remaining risk (not changed in-scope): script/scriptlet file loads are synchronous and can block when triggered from main-thread refresh paths (`src/scripts/loader.rs:72`, `src/scripts/scriptlet_loader/loading.rs:137`, callsite outside scope: `src/app_impl/refresh_scriptlets.rs:4`).

**Changed Files**
- `src/protocol/io/parsing.rs`
- `src/protocol/io/reader.rs`
- `src/execute_script/part_001_body/execute_interactive_merged.rs`

**How To Test**
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --lib --tests -- -D warnings`
3. `cargo test -p script-kit-gpui protocol::io::tests::parsing::`
4. `cargo test -p script-kit-gpui protocol::io::tests::reader::`

All passed.

**Commit**
- `e508c5f` `perf(protocol): reduce hot-path serde and buffer overhead`

**Risks / Known Gaps**
- I did not run full-workspace `cargo check/clippy/test` because this repo is under parallel-agent modification; I ran scoped validation for the changed data/protocol paths.
- The two remaining findings above are audit-only and not implemented in this patch.