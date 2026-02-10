**Result**
Split all oversized files in `src/config/` and `src/theme/` into sub-files under 500 lines, with wrapper modules and updated test module wiring in `mod.rs`.

**Changed Files**
- `src/config/types.rs`
- `src/config/types/part_01.rs`
- `src/config/types/part_02.rs`
- `src/config/types/part_03.rs`
- `src/config/types/part_04.rs`
- `src/config/editor.rs`
- `src/config/editor/part_01.rs`
- `src/config/editor/part_02.rs`
- `src/config/editor/tests.rs`
- `src/config/editor/tests/part_01.rs`
- `src/config/editor/tests/part_02.rs`
- `src/config/config_tests/mod.rs`
- `src/config/config_tests/part_01.rs`
- `src/config/config_tests/part_02.rs`
- `src/config/config_tests/part_03.rs`
- `src/config/config_tests/part_04.rs`
- `src/config/mod.rs`
- `src/config/config_tests.rs` (deleted)
- `src/theme/types.rs`
- `src/theme/types/part_01.rs`
- `src/theme/types/part_02.rs`
- `src/theme/types/part_03.rs`
- `src/theme/types/part_04.rs`
- `src/theme/types/part_05.rs`
- `src/theme/types/part_06.rs`
- `src/theme/presets.rs`
- `src/theme/presets/part_01.rs`
- `src/theme/presets/part_02.rs`
- `src/theme/presets/part_03.rs`
- `src/theme/presets/part_04.rs`
- `src/theme/theme_tests/mod.rs`
- `src/theme/theme_tests/part_01.rs`
- `src/theme/theme_tests/part_02.rs`
- `src/theme/theme_tests/part_03.rs`
- `src/theme/mod.rs`
- `src/theme/theme_tests.rs` (deleted)

**How To Test**
- `cargo check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

**Verification Outcome**
- Ran all three required commands.
- All three currently fail due unrelated parallel-agent breakages outside `src/config/*` and `src/theme/*` (examples: `src/terminal/alacritty.rs` missing modules, many `src/designs/*` split errors, `src/app_launcher/part_01.rs` unclosed delimiter).
- Confirmed no current diagnostics from `src/config/*` or `src/theme/*` in `cargo check` output.
- Confirmed all new split part files are `< 500` lines.

**Risks / Known Gaps**
- Global verification gate is blocked by external compile failures, so full repo-level green status is not achievable in current tree state.
- No commit/push was made because the verification gate is red.

**Commits**
- None.