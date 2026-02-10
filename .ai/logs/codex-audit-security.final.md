**Security Findings**
1. **High (fixed): JS injection surface in config extraction**
- `src/config/loader.rs:301` previously built a `bun -e` script by interpolating a path into JavaScript source (`require('...')`), which could break out if the temp path contained quotes.
- Fixed by passing the module path as a separate process argument and reading `process.argv[1]` in a static snippet (`src/config/loader.rs:14`, `src/config/loader.rs:27`, `src/config/loader.rs:302`).
- Regression test: `src/config/loader.rs:497`.

2. **Medium (fixed): predictable temp-file TOCTOU in config writes/recovery**
- `write_config_safely` / `recover_from_backup` used a deterministic `config.ts.tmp` path, enabling race/symlink attacks.
- Fixed with randomized secure tempfiles in the same directory and atomic persist (`src/config/editor/part_02.rs:88`, `src/config/editor/part_02.rs:211`, `src/config/editor/part_02.rs:264`).
- Regression tests: `src/config/editor/tests/part_02.rs:220`, `src/config/editor/tests/part_02.rs:286`.

3. **Audit result (no additional direct shell injection found)**
- In scoped paths, `Command::new` usage is confined to config code and uses argumentized process invocation (no `sh -c`/`cmd /C` patterns found).

**Changed Files**
- `src/config/loader.rs`
- `src/config/editor/part_02.rs`
- `src/config/editor/tests/part_02.rs`

**How To Test**
1. `cargo test test_build_bun_extract_config_command_passes_module_path_as_argument`
2. `cargo test test_write_config_safely_does_not_touch_predictable_tmp_path`
3. `cargo test test_recover_from_backup_does_not_touch_predictable_tmp_path`

**Risks / Known Gaps**
1. I ran scoped verification only (parallel-agent constraint), not full-workspace `cargo check/clippy/test`.
2. `config.ts` execution itself is still a trusted-code boundary by design (user config code is executed via Bun).

**Commits**
1. `eac946a` fix(config): harden config load/write against injection and temp races