# 026 Clipboard, Selected Text, and Accessibility APIs

```text
[clipboard-selected-text-accessibility-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 026: Clipboard, Selected Text, and Accessibility APIs / `copy()` / `paste()` / `clipboard.readText()` / `clipboard.writeText()` / `clipboard.readImage()` / `clipboard.writeImage()` / `getSelectedText()` / `setSelectedText()` / `hasAccessibilityPermission()` / `requestAccessibilityPermission()`.

This atlas must cover:

- SDK contracts, payload shapes, request ids, return values, auto-submit fallbacks, aliases, and error handling for clipboard and selected-text/accessibility APIs.
- Clipboard read/write implementation: executor-side `Message::Clipboard`, `ClipboardAction`, `ClipboardFormat`, arboard text/image behavior, `Submit` responses, empty-string failure fallback, no-request-id write behavior, image base64 limitations, and current gaps.
- `copy()` / `paste()` aliases and their relationship to `clipboard.writeText` / `clipboard.readText`.
- Selected text workflow: SDK hides the main window, waits 20ms, sends request, reads/writes focused app selection, and resolves/rejects from response values.
- Accessibility permissions: `checkAccessibility` vs `requestAccessibility`, macOS prompt behavior, read-only vs prompting boundaries, response shape, and non-macOS behavior.
- Rust selected text module: AX-first selected text, clipboard simulation fallback, clipboard restore, Core Graphics Cmd+V paste simulation, timing delays, permission gates, privacy logging, and platform guards.
- Stdin protocol and automation receipts: typed `SelectedText`, `TextSet`, `AccessibilityStatus` responses, request-id correlation, tracing event fields, no raw selected-text logging, source-audit tests, and session.sh RPC expectations.
- Data/privacy/security boundaries: clipboard contents, selected app text, passwords/private notes risk, base64 image payloads, app.log redaction via `text_len`, clipboard restoration best effort, and permission-prompt side effects.
- Relationship to adjacent features: clipboard history/root search, sharing clipboard trust watcher, emoji/clipboard-history paste flows, AI/chat copy/export, keyboard/mouse APIs, permission assistant, and MCP computer permission tools.
- Error, empty, unsupported, focus, lifecycle, and race states: no selected text, permission denied, frontmost app not restored, clipboard unavailable, image conversion issues, failed paste, clipboard restore failure, unsupported platform, response sender absent, and SDK auto-submit masking.

Important boundaries:

- Feature 008 covers root unified clipboard history; do not remap clipboard history storage/list UI here except as a related feature.
- Feature 025 covers `hud`, `setActions`, and `setInput`; include only the shared fire-and-forget/receipt lessons if needed.
- Keyboard/mouse/window APIs are separate later features; include only the Cmd+V Core Graphics boundary used by `setSelectedText`.
- Permission Assistant is an adjacent setup workflow; include its permission context but not its full UI.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: platform-windowing-macos, storage-cache-security, sdk-script-execution, protocol-automation, agentic-testing
- `lat.md`: protocol, builtins, sharing, permissions, verification
- Source: SDK clipboard/selected-text functions and message interfaces, protocol system-control variants and constructors, primitive clipboard enums, executor clipboard handling, prompt-handler stdin arms, selected_text module, permission wizard, source-audit tests, generated API tests, clipboard-related smokes

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 026 Clipboard, Selected Text, and Accessibility APIs

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
