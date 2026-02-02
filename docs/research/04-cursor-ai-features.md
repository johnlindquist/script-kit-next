# Cursor AI editor chat features and UX patterns

Research focus: Cmd+K inline editing, chat panel UX, context awareness, codebase indexing, keyboard shortcuts. This doc summarizes observed behavior and translates it into applicable patterns for Script Kit GPUI.

## Observed features & UX patterns (Cursor)

### Cmd+K inline edit (Prompt Bar)
- Inline Edit opens a prompt bar with `Ctrl/Cmd+K`, allowing edits or questions directly in the editor. The input uses the selected code + your instruction as the request. Without a selection, it generates new code at the cursor position. The model includes relevant surrounding code for context (e.g., triggering on a function name can include the whole function). ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Quick Question: `Alt/Option+Enter` in the inline editor asks about the selection, then you can type "do it" (or similar) to convert the answer into changes. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Full-file edits use `Ctrl/Cmd+Shift+Enter` for whole-file changes. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Send to Chat: `Ctrl/Cmd+L` sends selected code into Chat for multi-file edits or advanced workflows. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Follow-up instructions: after a generation, you can keep iterating by adding instructions and hitting `Enter`. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Default context: Inline Edit automatically adds related files, recently viewed code, and other relevant info beyond explicit @ symbols; it ranks and keeps the most relevant items. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))

### Chat panel + thread management
- Chat lives in an AI pane opposite the primary sidebar and can be toggled with `Ctrl/Cmd+L`, which focuses the chat input. ([Cursor Chat Overview](https://cursordocs.com/en/docs/chat/overview))
- Cursor Chat emphasizes context: it can include codebase context, web search, indexed docs, and user-selected code blocks. ([Cursor Chat Overview](https://cursordocs.com/en/docs/chat/overview))
- Threads: user messages can be edited and rerun (overwriting later messages). Chats are saved in history, which opens via a "Previous Chats" button or `Ctrl/Cmd+Alt/Option+L`. ([Cursor Chat Overview](https://cursordocs.com/en/docs/chat/overview))
- Default context includes the current file; context appears as "pills" above the input and can be removed per message. ([Cursor Chat Overview](https://cursordocs.com/en/docs/chat/overview))

### Context awareness & @-style reference
- Inline Edit and Chat both use a shared context system: manual references plus automatic context gathering (recent/related files) are ranked and attached. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Chat's context system explicitly mentions codebase context, web search, documentation indexing, and user-specified code blocks. ([Cursor Chat Overview](https://cursordocs.com/en/docs/chat/overview))

### Codebase indexing
- Cursor computes embeddings for each file in a codebase to improve AI answers; indexing starts automatically when a project is opened and new files are indexed incrementally. Status and included files are viewable in settings. Ignore rules use `.gitignore` and `.cursorignore`. ([Codebase Indexing](https://docs.cursor.com/context/codebase-indexing))
- Security details: indexing uses a Merkle tree of file hashes, syncs to the server, and periodically uploads only changed files. Files are chunked and embedded server-side; embeddings are stored with obfuscated file paths and line ranges. During inference, results return to the client, which reads local chunks and sends them for answers; privacy mode avoids plaintext storage server-side. ([Cursor Security](https://cursor.com/security))

### Keyboard shortcuts (selected)
- `Cmd/Ctrl+K` opens Inline Edit (Prompt Bar). ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- `Alt/Option+Enter` asks a Quick Question in Inline Edit. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- `Cmd/Ctrl+L` opens Chat and also sends selected code to Chat from Inline Edit. ([Cursor Chat Overview](https://cursordocs.com/en/docs/chat/overview); [Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- `Cmd/Ctrl+Shift+Enter` is used for full-file edits in Inline Edit. ([Cursor Inline Edit](https://docs.cursor.com/en/inline-edit/overview))
- Other Cursor shortcuts (for reference) include Composer, history, and context keybindings. ([Keyboard Shortcuts](https://cursordocs.com/en/docs/advanced/keyboard-shortcuts))

## Applicable patterns for Script Kit GPUI (recommendations)

### Cmd+K inline edit equivalent
- Provide an **inline prompt bar** attached to the current editor caret/selection. If there is no selection, default to **insert/generate at caret**; if there is a selection, perform **in-place edits**.
- Support a **Quick Question** mode (e.g., `Alt/Option+Enter`) to get an explanation before applying changes. Reuse the response as follow-up context when the user says "do it" (or taps "Apply").
- Allow **follow-up instructions** without leaving the inline bar, preserving the last edit context (selection + auto-context). (Inference based on Cursor's iteration flow.)
- Provide a **one-step escape hatch to Chat** for multi-file or longer tasks (e.g., `Cmd/Ctrl+L` from inline prompt).

### Chat panel UX
- Place AI chat in a **dedicated side panel** that can be toggled and focused with a single shortcut. This is the hub for multi-file reasoning and history.
- Expose a **history view** of chat threads, with the ability to revisit and rerun previous prompts (with a warning that future messages will be replaced). (Inference based on Cursor's "edit previous messages" behavior.)
- Show **context pills** above the chat input for transparency and easy removal per message.

### Context awareness model
- Use **auto-context selection** (recent files, related symbols, open buffers) that can be **ranked and capped**. Keep the current file as a default context item, with a simple way to remove it per prompt.
- Support **explicit context references** (e.g., `@file`, `@symbol`, `@codebase`, `@docs`, `@web`) that appear as pills in both inline edit and chat.

### Codebase indexing (architecture & UX)
- Start indexing **automatically** when a project is opened; update **incrementally** when files change; surface status in settings/UI.
- Respect **ignore files** (`.gitignore` + tool-specific ignore) and offer a **view of included files** for transparency.
- If using server-side embeddings, consider **hash-based incremental updates**, **obfuscated path metadata**, and **client-side chunk retrieval** to reduce plaintext storage (pattern mirrored from Cursor's security description). (Inference based on Cursor's design.)

### Shortcut strategy
- Maintain **muscle-memory parity** with Cursor where it helps adoption:
  - `Cmd/Ctrl+K`: inline edit
  - `Alt/Option+Enter`: quick question
  - `Cmd/Ctrl+L`: open chat
  - `Cmd/Ctrl+Shift+Enter`: full-file edit (or "edit file")
- Keep shortcuts consistent across prompt bars (inline edit, chat input, terminal command bar).

## Notes & caveats
- Some UX details are sourced from Cursor documentation mirrors (e.g., cursordocs.com). Where possible, prioritize Cursor's official domain; mirrors are used here due to access constraints in tooling. Validate against the live Cursor docs in a browser if needed before final product decisions.

## Sources
- https://docs.cursor.com/en/inline-edit/overview
- https://cursordocs.com/en/docs/chat/overview
- https://cursordocs.com/en/docs/advanced/keyboard-shortcuts
- https://docs.cursor.com/context/codebase-indexing
- https://cursor.com/security
