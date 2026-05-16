# 015 SDK TermPrompt Prompt

```text
[sdk-term-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 015: SDK TermPrompt / term() prompt runtime / terminal actions / full-height terminal.

This atlas must cover the full SDK-spawned terminal prompt user and agent behavior:

- The TypeScript SDK `term(command?, actions?)` API and the JSON/protocol request it sends.
- The Rust prompt handler route that creates `AppView::TermPrompt { id, entity }`.
- How SDK TermPrompt differs from Quick Terminal (`AppView::QuickTerminalView`) and ACP Chat.
- Full-height terminal sizing, `ViewType::TermPrompt`, and why SDK terminal prompts must not inherit compact Quick Terminal sizing.
- Terminal rendering, Alacritty/PTY lifecycle, shell command execution, interactive shell without a command, terminal theme adaptation, scrollback, selection, copy/paste, and mouse behavior.
- SDK terminal actions: SDK-provided actions versus built-in terminal commands, clear, scroll, reset, actions toggle, action host ownership, shortcut behavior, and footer/hint-strip behavior.
- Keyboard semantics: printable text, Ctrl keys, Enter, Tab, Shift+Tab, Escape, Cmd+C, Cmd+V, Cmd+K, Cmd+Shift+K, Cmd+W, and any known physical/protocol differences.
- Return value semantics: terminal output string, close/submit behavior, command exit code behavior, ANSI output handling, multi-line output, and non-command interactive session behavior.
- Automation and agentic receipts: getState, getElements, element collection, semantic ids, target identity, footer ownership matrix, SDK tests, smoke tests, and source-contract tests.
- Data/privacy boundaries: terminal output versus prompt text, context extraction, clipboard use, local shell environment, PTY subprocess state, and command output capture.
- Error, loading, disabled, and edge states.

Important known requirements from current docs and the previous Quick Terminal pass:

- SDK `term()` creates `AppView::TermPrompt { id, entity }`; launcher Quick Terminal creates `AppView::QuickTerminalView`.
- SDK TermPrompt keeps full terminal prompt height through `ViewType::TermPrompt`.
- SDK TermPrompt does not register the native `quick_terminal` footer surface; it keeps the GPUI terminal hint strip or prompt-owned footer behavior.
- Quick Terminal uses compact sizing and native footer; do not collapse these two terminal surfaces.
- `TermPrompt` is shared implementation, so terminal input/rendering/theming behavior may apply to both surfaces, but route identity, sizing, footer ownership, and close/apply-back behavior differ.
- Source-contracts and smoke tests should be named when they prove behavior; recommended recipes should be marked as recommended if not actually run.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: quick-terminal-pty, sdk-script-execution, prompt-runtime, protocol-automation, agentic-testing
- `lat.md`: design, surfaces, acp-chat, protocol, scripting, verification
- Source: SDK `term()` definition, prompt handler route, TermPrompt renderer/input, terminal/PTY lifecycle, terminal actions/dialog, footer/window dispatch, layout/element collection, surface state, window resize, simulateKey dispatch
- Tests/scripts: SDK term test, smoke term tests, Quick Terminal/TermPrompt contracts, footer ownership contract, resize presentation contract, footer ownership matrix

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 015 SDK TermPrompt / term() / Terminal Actions / Full-height Terminal

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
