# 014 Quick Terminal PTY Prompt

```text
[quick-terminal-pty-atlas]




- Opening Quick Terminal from the launcher and from path actions.
- PTY lifecycle, cold spawn, warm PTY pool, stale/dead handle behavior, cleanup, and shutdown.
- Quick Terminal sizing and launcher height contract.
- Terminal rendering, Alacritty content, theme adaptation, edge inset, scroll/mouse behavior.
- Path actions that open Quick Terminal at a directory or file parent and write a quoted `cd` command.
- Zsh prompt `%` / blank-row suppression through `PROMPT_EOL_MARK`, `ZDOTDIR`, and the Script Kit zsh shim.
- Automation and verification recipes for state-first proof and source-contract tests.


- Quick Terminal is PTY-backed and is not ACP Chat.
- `Tab` and `Shift+Tab` inside Quick Terminal belong to the PTY, not ACP focus navigation.
- Plain `Escape` is forwarded to the PTY.
- `Cmd+W`, protocol `simulateKey` Cmd+W, and native footer Close tear down the PTY harness and close the main window state-first.
- `Cmd+Enter` apply-back is terminal-specific and only runs when `quick_terminal_can_apply_back()` is true.
- Quick Terminal opened from the launcher must stay at compact launcher height; it must not call the SDK TermPrompt resize path.
- Warm PTYs and cold PTYs must use the active theme, and attached warm PTYs must be rethemed.
- Quick Terminal uses a native footer surface `quick_terminal`; SDK `TermPrompt` does not.
- Quick Terminal footer buttons are `[Close]` or `[Apply, Close]`; Run / AI / Actions are not copied from the launcher footer.
- Zsh prompt marker suppression must happen at spawn time, not by attach-time clear bytes.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


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
