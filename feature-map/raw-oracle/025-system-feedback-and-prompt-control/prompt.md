# 025 System Feedback and Prompt Control APIs

```text
[system-feedback-prompt-control-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 025: System Feedback and Prompt Control APIs / `beep()` / `say()` / `notify()` / `hud()` / `setStatus()` / `menu()` / `setActions()` / `setInput()`.

This atlas must cover:

- SDK API contracts, argument shapes, return values, fire-and-forget semantics, warnings/stub comments, and message payloads for each function.
- Current implementation truth: distinguish APIs with real Rust behavior (`hud`, `setActions`, `setInput`) from SDK/protocol-visible but unimplemented or logging-only APIs (`beep`, `say`, `notify`, `setStatus`, `menu`) based on source proof.
- HUD behavior: `PromptMessage::ShowHud`, `show_hud`, HUD manager windows/slots/duration/stacking, hide-visibility decoupling, auto-dismiss behavior, and tests/smokes.
- Prompt actions behavior: SDK action normalization/serialization, `__kitActionsMap`, `setActions`, `ProtocolAction`, `ActionTriggered`, actions dialog refresh/resizing, shortcuts, handler vs submit-value actions, visible/close flags, and current prompt boundaries.
- Prompt input behavior: SDK `setInput`, Rust `PromptMessage::SetInput`, `set_prompt_input`, batch `setInput`, target-specific setInput for main, detached ACP, Notes, and ActionsDialog where relevant.
- Status/menu/notification/speech/beep boundaries: what is typed and serialized, what is logged or warned, what backend side effects are absent, and how agents should verify gaps without assuming native behavior.
- Relationship to adjacent features: Actions Dialog, prompt runtime, protocol automation, HUD manager, tray/menu, platform/macOS notification/speech, ACP/Notes target setInput, and built-in HUD uses.
- Automation/protocol surface: stdin JSON messages, batch `setInput`, `getState` changes after `setInput`, action-trigger callbacks, response messages, transaction traces, target identity, and failure paths.
- Data/privacy boundaries: text entered into prompts, HUD/status/notification/body text, action names/descriptions/values/shortcuts, logs, automation receipts, and no persistence unless caller or prompt state persists.
- Error, empty, disabled, unsupported, race, and lifecycle states: empty actions, duplicate action names, invisible actions, no handler/no value actions, setInput before prompt exists, setInput on non-editable prompts, HUD while main is hidden, script exit/cancel, unsupported native APIs, and action dialog open/closed update parity.

Important boundaries:

- Feature 016 covers core `arg()`, `select()`, `div()`, and `md()` prompt lifecycle; this feature should focus on utility/control calls that mutate prompt/UI feedback after or outside initial prompt creation.
- Feature 011 covers root result actions and actions dialog behavior broadly; include `setActions()` only as the SDK/script-facing way to populate prompt actions.
- Tray/menu bar observation/action APIs are separate later features; include `menu()` only as the SDK fire-and-forget message for tray icon/scripts if source proves it.
- Clipboard, selected text, keyboard/mouse, windowing, media, chat, and AI APIs are separate later features. Mention only boundary notes.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, actions-popups, protocol-automation, agentic-testing, platform-windowing-macos, theme-config-preferences
- `lat.md`: protocol, verification, surfaces, design, windowing, tray-menu if present, acp-chat for HUD label boundary
- Source: SDK system/control functions and message interfaces, protocol variants/constructors, prompt handler arms, HUD manager, action dialog/action handling, prompt input helpers, batch setInput, tray/menu module, relevant tests/smokes

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 025 System Feedback and Prompt Control APIs

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
