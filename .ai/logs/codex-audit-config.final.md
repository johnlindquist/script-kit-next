**Audit Findings**
1. `High` `Fixed` Missing semantic validation on numeric config values could let invalid values propagate (e.g., `<=0`, `NaN`, `Inf`) for editor/terminal font size and UI scale.  
`src/config/types/part_03.rs:28` `src/config/types/part_03.rs:61` `src/config/types/part_03.rs:67` `src/config/types/part_03.rs:73`

2. `Medium` `Fixed` `processLimits.healthCheckIntervalMs` accepted `0`, which is not a sensible polling interval and can cause bad runtime behavior. Getter now falls back to default.  
`src/config/types/part_03.rs:35` `src/config/types/part_03.rs:92`

3. `Low` `Fixed` Inconsistent default docs for notes hotkey: comments claimed a default shortcut, but behavior is “no default unless configured.”  
`src/config/types/part_02.rs:191` `src/config/types/part_03.rs:15` `src/config/types/part_03.rs:101`

4. `Medium` `Open` `config.ts` `layout` is loaded into `Config` but not used for runtime sizing; runtime uses `settings.json` layout instead.  
`src/config/types/part_02.rs:226` `src/config/types/part_03.rs:155` `src/window_resize/part_000.rs:258`

5. `Medium` `Open` `processLimits` is loaded and exposed, but I found no non-test runtime consumers of `get_process_limits()`.  
`src/config/loader.rs:108` `src/config/types/part_03.rs:92`

6. `Medium` `Open` `watcher` config values are loaded/sanitized via `load_watcher_settings()`, but watch loop currently uses constants directly (`DEBOUNCE_MS`, `MAX_NOTIFY_ERRORS`), so runtime watcher tuning appears not applied.  
`src/watcher/part_000.rs:46` `src/watcher/part_000.rs:247` `src/watcher/part_000.rs:296`

7. `Info` `Reviewed` I did not find panic-prone paths in config parsing/loading: loader parse paths use fallbacks instead of unwrap/panic on malformed input.  
`src/config/loader.rs:125` `src/config/loader.rs:167`

**Changed Files**
- `src/config/types/part_03.rs`
- `src/config/types/part_02.rs`
- `src/config/config_tests/part_02.rs`
- `src/config/config_tests/part_04.rs`

**How To Test**
1. `cargo test test_config_get_editor_font_size_uses_default_for_invalid_values -- --nocapture`
2. `cargo test test_config_get_terminal_font_size_uses_default_for_invalid_values -- --nocapture`
3. `cargo test --lib test_config_get_ui_scale_uses_default_for_invalid_values -- --nocapture`
4. `cargo test test_config_get_process_limits_uses_default_interval_when_zero -- --nocapture`

**Risks / Known Gaps**
- I did not change watcher/runtime consumers outside config scope, so the “loaded but unused” findings above remain open.
- A broad run `cargo test config::tests:: -- --nocapture` also matched unrelated `ai::config` tests and failed outside this task’s scope.
- During parallel agent activity, one non-`--lib` test invocation hit an unrelated compile error in `src/terminal/alacritty/handle_creation.rs` (`missing field reader_thread`), so I validated the UI-scale test with `--lib` to isolate this config work.

**Commits**
- `e38ff288152d0ee466ff2efb789469c072e4fab9` `fix(config): sanitize invalid numeric getter values`