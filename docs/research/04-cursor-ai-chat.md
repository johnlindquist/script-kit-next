# Cursor AI Chat Integration Patterns

Research goal: capture Cursor's chat, inline edit, and Composer patterns to inform Script Kit's AI chat window.

## Inline chat / in-editor prompt bar (Cmd+K)

- Inline Edit opens an in-editor prompt bar with Cmd/Ctrl+K. With a selection, it edits the selected code in place; without a selection, it generates code at the cursor. Default context can include related files and recent code.
- The prompt bar supports @-symbol context references and follow-up instructions to refine changes.
- Inline Edit supports quick questions about selected code (Alt/Option+Enter) and full-file edits (Ctrl/Cmd+Shift+Enter).
- Selected code can be sent to Chat for multi-file edits or advanced workflows (Ctrl/Cmd+L).

## Composer (multi-file AI workspace)

- Composer is Cursor's in-editor AI assistant for exploring code, writing features, and making multi-file edits. Open with Cmd+I; create a new Composer with Cmd+N.
- Agent mode (toggle Cmd+.) can pull context, run terminal commands, create/modify files, and search code semantically.
- Composer emphasizes reviewability: diff view, accept/reject controls, and checkpoints for undoing changes.
- Context controls are surfaced as pills; @ and # are used to add or focus context.
- Composer includes history and multiple layout modes (pane vs editor).

## Code context awareness patterns

- Cursor indexes codebases with embeddings, keeps them in sync, supports multi-root workspaces, and respects ignore lists to improve relevance.
- Chat can force codebase search via Cmd/Ctrl+Enter or @codebase for targeted context injection.
- Cursor automatically pulls relevant context (current file, semantically similar patterns, session info) and recommends explicit @ references for precision.
- Chat shows context pills above the input and defaults to the current file unless removed.
- Rules and memories provide persistent guidance that applies to both Chat and Inline Edit.

## Keyboard shortcuts (high-signal subset)

- Chat: `Cmd/Ctrl+L` to open chat; `Cmd/Ctrl+Enter` to submit with codebase.
- Inline Edit: `Cmd/Ctrl+K` to open; `Alt/Option+Enter` for quick question; `Cmd/Ctrl+Shift+Enter` for full-file edits.
- Composer: `Cmd+I` to open; `Cmd+N` new Composer; `Cmd+.` toggle Agent; `Cmd+Alt+L` history.
- Context references: `@` and `#` for explicit context and file selection.

## Suggestions for Script Kit AI chat window

1. Provide two distinct surfaces:
   - Inline edit bar (Cmd+K) for quick, in-file changes.
   - Full chat/Composer window (Cmd+L / Cmd+I) for multi-file work and longer reasoning.
2. Add a context control UI patterned after Cursor:
   - Show "context pills" with file/symbol sources.
   - Provide explicit @ and # entry points to add context.
   - Make codebase search an explicit action (e.g., Cmd+Enter) and show indexing status.
3. Emphasize reviewability for edits:
   - Diff view with accept/reject.
   - Checkpoints for revert to prior states.
4. Use persistent instruction channels:
   - Project/user rules in a visible, editable location.
   - Surface which rules are active per conversation.
5. History and multi-conversation UX:
   - Tabs or a history panel for quick revisit.
   - Rename, archive, or pin conversations for recurring workflows.
6. Keyboard-first UX:
   - Consistent shortcuts across chat/inline/composer.
   - Clear in-UI hinting for key actions (submit, apply, cancel, toggle mode).
7. Make context transparency obvious:
   - Display auto-included context (current file, relevant matches).
   - Offer a "show exact context" view for debugging.

## Sources

- https://docs.cursor.com/get-started/concepts
- https://docs.cursor.com/en/inline-edit/overview
- https://docs.cursor.com/chat/overview
- https://docs.cursor.com/context/codebase-indexing
- https://docs.cursor.com/en/guides/working-with-context
- https://docs.cursor.com/context/rules
- https://cursordocs.com/en/docs/cmdk/overview
- https://cursordocs.com/en/docs/composer/overview
- https://cursordocs.com/en/docs/chat/overview
- https://cursordocs.com/en/docs/context/codebase-indexing
- https://cursordocs.com/en/docs/advanced/keyboard-shortcuts
