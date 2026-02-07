# Script Kit GPUI

Rust (GPUI app shell) + TypeScript (bun script runner) + SDK. Backwards-compatible rewrite of Script Kit.

## Scope Rules

- Do ONLY what is explicitly requested. No unrequested changes, refactors, or "improvements."
- If you notice something worth improving, mention it at the end — do not implement it.
- Stay within the boundaries of the task. A docs request is not a code change.

## Verification Gate (Mandatory)

Every code change must pass before reporting success:

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

After the gate passes, verify the change actually works:
- **Logic changes**: check logs with `SCRIPT_KIT_AI_LOG=1`
- **UI changes**: capture screenshot AND read the PNG to confirm visually
- **Never** report success without running verification

## Build & Test

| Action | Command |
|--------|---------|
| Check | `cargo check` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Test | `cargo test` |
| Test (system) | `cargo test --features system-tests` |
| Test (slow) | `cargo test --features slow-tests` |
| Run | `echo '{"type":"show"}' \| SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1` |
| Bundle | `cargo bundle --release` |

## Coding Conventions

### Rust
- Use `?` or graceful error handling — never `unwrap()` in unsafe/ObjC code
- After any render-affecting mutation: `cx.notify()`
- Use `theme.colors.*` — never hardcode `rgb(0x...)`
- Keyboard keys — match both variants:
  ```rust
  match key.as_str() {
    "up" | "arrowup" => ...,
    "down" | "arrowdown" => ...,
    "enter" | "Enter" => ...,
    "escape" | "Escape" => ...,
    _ => {}
  }
  ```

### UI Testing
- **Never** pass scripts as CLI args — use stdin JSON protocol
- Always use `SCRIPT_KIT_AI_LOG=1` for compact log output
- After screenshots, **read the PNG file** to verify

## Architecture Quick Ref

- Built-in commands: `BuiltInFeature` enum → `get_builtin_entries()` → `execute_builtin()` → `AppView` variant → render
- Non-dismissable views: add to `is_dismissable_view()` in `app_impl.rs`
- Vibrancy: prompts should NOT set opaque bg — let vibrancy show through from Root
- Render wrappers: `render_prompts/other.rs` wraps prompt entities, `prompts/*.rs` are inner components
- Protocol: bidirectional JSONL over stdin/stdout between bun scripts and Rust app — see `docs/PROTOCOL.md`

## Session Completion

Work is not done until `git push` succeeds.

1. Run verification gate (check/clippy/test)
2. Commit with descriptive message
3. `git pull --rebase && git push && git status`
4. Never say "ready to push when you are" — just push

## Skills (Loaded On-Demand)

Detailed guidance lives in `.claude/skills/` — load only when relevant:

| Skill | When to Use |
|-------|-------------|
| `script-kit-agent-workflow` | Fix-verify loop, session completion |
| `script-kit-ui-testing` | Screenshots, stdin JSON protocol, layout debugging |
| `gpui-patterns` | UI code, keyboard events, layouts, themes |
| `visual-test` | Visual iteration, named-pipe testing, captureWindow |
| `dev-loop` | Background dev server, log monitoring, runtime verification |
| `script-kit-architecture` | Navigating codebase, understanding modules |
| `script-kit-logging` | Adding logs, observability, correlation IDs |
| `script-kit-testing` | Writing tests, test organization |
| `script-kit-scripting` | Script metadata, scriptlet bundles |
| `script-kit-hive` | Task management, beads, issue tracking |

## References

- GPUI docs: https://docs.rs/gpui/latest/gpui/
- Zed source: https://github.com/zed-industries/zed/tree/main/crates/gpui
- Protocol reference: `docs/PROTOCOL.md`
