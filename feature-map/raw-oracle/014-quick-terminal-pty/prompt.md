# 014 Quick Terminal PTY Prompt

```text
[quick-terminal-pty-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 014: Quick Terminal PTY / TermPrompt / warm pool / native footer / apply-back.

This atlas must cover the full Quick Terminal user and agent behavior after the `>` ScriptList handoff or built-in Quick Terminal route:

- Opening Quick Terminal from the launcher and from path actions.
- `AppView::QuickTerminalView` versus SDK-spawned `TermPrompt` and versus ACP Chat.
- PTY lifecycle, cold spawn, warm PTY pool, stale/dead handle behavior, cleanup, and shutdown.
- Terminal input semantics: printable keys, Ctrl keys, special keys, Tab, Shift+Tab, Escape, Cmd+W, Cmd+Enter, Cmd+K.
- Quick Terminal sizing and launcher height contract.
- Terminal rendering, Alacritty content, theme adaptation, edge inset, scroll/mouse behavior.
- Native footer behavior: Close, Apply, `footer:native:close`, footer ownership, and active footer receipts.
- Apply-back routing: when Apply is visible, what Cmd+Enter does, what it must not do, and how close clears apply-back state.
- Path actions that open Quick Terminal at a directory or file parent and write a quoted `cd` command.
- Zsh prompt `%` / blank-row suppression through `PROMPT_EOL_MARK`, `ZDOTDIR`, and the Script Kit zsh shim.
- Automation and verification recipes for state-first proof and source-contract tests.

Important known requirements from current docs:

- Quick Terminal is PTY-backed and is not ACP Chat.
- `Tab` and `Shift+Tab` inside Quick Terminal belong to the PTY, not ACP focus navigation.
- Plain `Escape` is forwarded to the PTY.
- `Cmd+W`, protocol `simulateKey` Cmd+W, and native footer Close tear down the PTY harness and close the main window state-first.
- `Cmd+Enter` apply-back is terminal-specific and only runs when `quick_terminal_can_apply_back()` is true.
- Quick Terminal opened from the launcher must stay at compact launcher height; it must not call the SDK TermPrompt resize path.
- Warm PTY reuse must fail open: stale/dead/missing/inflight/spawn-failed warm handles fall back to cold spawn.
- Warm PTYs and cold PTYs must use the active theme, and attached warm PTYs must be rethemed.
- Quick Terminal uses a native footer surface `quick_terminal`; SDK `TermPrompt` does not.
- Quick Terminal footer buttons are `[Close]` or `[Apply, Close]`; Run / AI / Actions are not copied from the launcher footer.
- Zsh prompt marker suppression must happen at spawn time, not by attach-time clear bytes.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: quick-terminal-pty, protocol-automation, agentic-testing
- `lat.md`: acp-chat, surfaces, automation, protocol, verification
- Source: Quick Terminal openers, warm PTY pool, native footer, tab AI harness close/apply-back, TermPrompt rendering/input, terminal/PTY lifecycle, window sizing, simulateKey dispatch, path action routing
- Tests/scripts: Quick Terminal contracts, tab AI routing/apply-back contracts, footer ownership contracts, SDK term smoke, footer ownership matrix

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 014 Quick Terminal PTY / TermPrompt / Warm Pool / Apply-back

### Executive Summary

### What Users Can Do

### Core Concepts

### Entry Points

### User Workflows

### Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|

### State Machine

### Visual And Focus States

### Keystrokes And Commands

### Actions And Menus

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
```
