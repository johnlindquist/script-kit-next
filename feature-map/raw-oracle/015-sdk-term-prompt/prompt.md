# 015 SDK TermPrompt Prompt

```text
[sdk-term-prompt-atlas]




- The TypeScript SDK `term(command?, actions?)` API and the JSON/protocol request it sends.
- Terminal rendering, Alacritty/PTY lifecycle, shell command execution, interactive shell without a command, terminal theme adaptation, scrollback, selection, copy/paste, and mouse behavior.
- Error, loading, disabled, and edge states.


- SDK TermPrompt does not register the native `quick_terminal` footer surface; it keeps the GPUI terminal hint strip or prompt-owned footer behavior.
- Quick Terminal uses compact sizing and native footer; do not collapse these two terminal surfaces.
- `TermPrompt` is shared implementation, so terminal input/rendering/theming behavior may apply to both surfaces, but route identity, sizing, footer ownership, and close/apply-back behavior differ.
- Source-contracts and smoke tests should be named when they prove behavior; recommended recipes should be marked as recommended if not actually run.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


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
