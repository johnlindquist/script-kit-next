Changed file:
- `src/actions/command_bar/part_03.rs:109`  
  Renamed `mod tests` to `mod command_bar_config_tests` so it does not collide in the shared module scope created by `include!` in `src/actions/command_bar.rs`.

How to test:
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `rg -n "^mod tests\\b" src/actions/command_bar/part_*.rs` (should return no matches)

What I ran:
- `cargo check -p script-kit-gpui` (failed due unrelated existing errors in `clipboard_history/*`, not in command bar files)
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` (failed due unrelated existing workspace errors, including duplicate module/file issues outside command bar scope)
- `rg -n "^mod tests\\b" src/actions/command_bar/part_*.rs` (no matches)

Risks / known gaps:
- Full check/clippy gate is currently red due unrelated in-flight workspace issues, so end-to-end green validation is blocked outside this task’s scope.

Commit made:
- `0411638` `fix(command-bar): rename config test module to avoid collision`