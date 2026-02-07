Implemented prompt spacing normalization in your scoped files.

**Changed Files**
- `src/render_prompts/arg.rs`:4  
Added shared helper `prompt_actions_dialog_offsets(...)` and replaced hardcoded actions popup offsets (`52/8`) with token-based values in arg render (`src/render_prompts/arg.rs:116`, `src/render_prompts/arg.rs:557`).
- `src/render_prompts/div.rs`:17  
Switched div prompt actions popup positioning to shared token-based offsets (`src/render_prompts/div.rs:204`).
- `src/render_prompts/form.rs`:16  
Switched form prompt actions popup positioning to shared token-based offsets (`src/render_prompts/form.rs:241`).
- `src/render_prompts/editor.rs`:22  
Switched editor prompt actions popup positioning to shared token-based offsets (`src/render_prompts/editor.rs:293`).
- `src/render_prompts/term.rs`:13  
Switched term prompt actions popup positioning to shared token-based offsets (`src/render_prompts/term.rs:212`).
- `src/render_prompts/path.rs`:58  
Switched path prompt actions popup top/right padding to shared token-based offsets (`src/render_prompts/path.rs:322`).
- `src/prompts/div.rs`:133  
Replaced hardcoded default container padding `12.0` with design-token default (`spacing().padding_md`) via `default_container_padding(...)`, used in render (`src/prompts/div.rs:964`).  
Added regression test: `test_default_container_padding_follows_design_spacing` (`src/prompts/div.rs:1088`).

**Validation Run**
- `cargo check` passed.
- `cargo test` passed (full suite in current tree).
- `cargo clippy --all-targets -- -D warnings` failed due unrelated existing issues in `src/actions/dialog_builtin_action_validation_tests_*.rs` and one clippy issue in `src/ai/window.rs`.
- Runtime stdin JSON verification run completed:
  - `ShowArg` with actions confirmed in logs.
  - `simulateKey` Cmd+K confirmed in logs.
  - Screenshots captured and reviewed:
    - `test-screenshots/test-before-cmdk.png`
    - `test-screenshots/test-after-cmdk.png`
  - The “after” image shows the actions dialog opened at the expected top-right aligned position.

**How To Test**
1. `cargo check`
2. `cargo test`
3. `cargo clippy --all-targets -- -D warnings` (currently expected to fail on unrelated files above)
4. Visual/runtime check (stdin protocol):
```bash
timeout 40 bash -lc '
PIPE=$(mktemp -u); mkfifo "$PIPE"
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui < "$PIPE" 2>&1 | tee /tmp/padding-margins-arg-actions.log &
APP_PID=$!
exec 3>"$PIPE"
sleep 3
echo "{\"type\":\"run\",\"path\":\"$(pwd)/tests/smoke/test-arg-actions-full.ts\"}" >&3
sleep 1.1
echo "{\"type\":\"simulateKey\",\"key\":\"k\",\"modifiers\":[\"cmd\"]}" >&3
sleep 2
echo "{\"type\":\"simulateKey\",\"key\":\"escape\",\"modifiers\":[]}" >&3
sleep 0.4
echo "{\"type\":\"simulateKey\",\"key\":\"enter\",\"modifiers\":[]}" >&3
sleep 3
exec 3>&-; rm -f "$PIPE"; kill "$APP_PID" 2>/dev/null || true; wait "$APP_PID" 2>/dev/null || true
'
```

**Risks / Known Gaps**
- Repo-wide clippy is currently red for unrelated files, so full lint gate is not green yet.
- The helper test added in `src/render_prompts/arg.rs` is in an `include!`d render file; main actionable regression test is the library test in `src/prompts/div.rs`.