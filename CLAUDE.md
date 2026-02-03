# Script Kit GPUI

Script Kit GPUI is a rewrite of Script Kit using **GPUI** (app shell) + **bun** (script runner) + a new SDK. Goal: **backwards compatibility** for Script Kit scripts with a new architecture.

---

## Quick-Start Checklist (MANDATORY)

1. Read this file before changing code
2. Check `.hive/issues.jsonl` for tasks/context
3. **TDD**: write failing test → implement → refactor
4. Update bead status when starting/completing work
5. Include `correlation_id` in all log entries/spans
6. **UI changes**: test via stdin JSON protocol (never CLI args)
7. Before every commit: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

---

## Skills

This project uses modular skills for detailed guidance. Skills are loaded on-demand to keep context focused.

| Skill | When to Use |
|-------|-------------|
| `script-kit-agent-workflow` | Fixing bugs, completing tasks, session workflow |
| `script-kit-ui-testing` | Testing UI changes, screenshots, layout debugging |
| `script-kit-scripting` | Writing scripts, metadata, scriptlet bundles |
| `gpui-patterns` | UI code, keyboard events, layouts, themes |
| `script-kit-logging` | Adding logs, debugging, observability |
| `script-kit-architecture` | Navigating codebase, understanding modules |
| `script-kit-testing` | Writing tests, running test suites |
| `script-kit-hive` | Task management, beads, file reservations |
| `visual-test` | Visual iteration workflow, named-pipe testing, captureWindow screenshots |

Skills are located in `.claude/skills/`.

---

## Critical Rules (Always Apply)

### UI Testing Protocol
- **Never** pass scripts as CLI args; use stdin JSON protocol
- Always use `SCRIPT_KIT_AI_LOG=1` for compact logs
- After screenshots, **read the PNG file** to verify

### Verification Gate
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Keyboard Keys
Match both variants:
```rust
match key.as_str() {
  "up" | "arrowup" => ...,
  "down" | "arrowdown" => ...,
  "enter" | "Enter" => ...,
  "escape" | "Escape" => ...,
  _ => {}
}
```

### State Updates
After any render-affecting mutation: `cx.notify()`

### Theme Colors
Use `theme.colors.*` — never hardcode `rgb(0x...)`

### Session Completion
Work is not done until `git push` succeeds. Never stop before pushing.

---

## References

- GPUI docs: https://docs.rs/gpui/latest/gpui/
- Zed source: https://github.com/zed-industries/zed/tree/main/crates/gpui
- Protocol reference: `docs/PROTOCOL.md`
- Roadmap: `docs/ROADMAP.md`
